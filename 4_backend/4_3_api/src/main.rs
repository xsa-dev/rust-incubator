use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    Json, Router, async_trait,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{Method, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use clap::{Parser, Subcommand};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct User {
    id: Uuid,
    name: String,
    #[serde(skip_serializing)]
    password: String,
    friends: HashSet<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct PublicUser {
    id: Uuid,
    name: String,
    friends: Vec<Uuid>,
}

impl From<&User> for PublicUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            name: user.name.clone(),
            friends: user.friends.iter().copied().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct UserGraph {
    user: PublicUser,
    friends: Vec<PublicUser>,
}

#[derive(Debug, Default, Clone)]
struct AuthStore {
    tokens: HashMap<String, Uuid>,
}

#[derive(Debug, Clone, Default)]
struct AppState {
    users: HashMap<Uuid, User>,
    names: HashMap<String, Uuid>,
    auth: AuthStore,
}

#[derive(Clone, Default)]
struct SharedState(Arc<Mutex<AppState>>);

impl FromRef<SharedState> for Arc<Mutex<AppState>> {
    fn from_ref(state: &SharedState) -> Self {
        state.0.clone()
    }
}

#[derive(Debug, Deserialize, ToSchema)]
struct RegisterPayload {
    name: String,
    password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
struct LoginPayload {
    name: String,
    password: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct TokenResponse {
    token: String,
}

#[derive(Error, Debug)]
enum ApiError {
    #[error("user already exists")]
    UserExists,
    #[error("user not found")]
    UserNotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("not authorized")]
    Unauthorized,
    #[error("failed to parse identifier")]
    BadIdentifier,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::UserExists | ApiError::InvalidCredentials => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::UserNotFound => StatusCode::NOT_FOUND,
            ApiError::BadIdentifier => StatusCode::BAD_REQUEST,
        };
        (status, self.to_string()).into_response()
    }
}

struct AuthenticatedUser(Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<Mutex<AppState>>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?
            .to_string();

        let state: Arc<Mutex<AppState>> = Arc::from_ref(state);
        let guard = state.lock().await;
        let user = guard
            .auth
            .tokens
            .get(&token)
            .ok_or(ApiError::Unauthorized)?;
        Ok(Self(*user))
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(register_user, login_user, get_user_graph, add_friend, remove_friend),
    components(schemas(RegisterPayload, LoginPayload, TokenResponse, UserGraph, PublicUser)),
    tags((name = "api", description = "Simple REST API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Server { addr } => run_server(addr).await?,
        Command::Register {
            server,
            name,
            password,
        } => {
            let payload = RegisterPayload { name, password };
            let _ = reqwest::Client::new()
                .post(url(&server, "/register")?)
                .json(&payload)
                .send()
                .await?
                .error_for_status()?;
            println!("Registered successfully");
        }
        Command::Login {
            server,
            name,
            password,
        } => {
            let payload = LoginPayload { name, password };
            let token: TokenResponse = reqwest::Client::new()
                .post(url(&server, "/login")?)
                .json(&payload)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            println!("Token: {}", token.token);
        }
        Command::GetUser { server, token, id } => {
            let response = reqwest::Client::new()
                .get(url(&server, &format!("/users/{id}"))?)
                .bearer_auth(token)
                .send()
                .await?
                .error_for_status()?;
            let graph: UserGraph = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&graph)?);
        }
        Command::AddFriend {
            server,
            token,
            id,
            friend_id,
        } => {
            reqwest::Client::new()
                .post(url(&server, &format!("/users/{id}/friends/{friend_id}"))?)
                .bearer_auth(token)
                .send()
                .await?
                .error_for_status()?;
            println!("Friend added");
        }
        Command::RemoveFriend {
            server,
            token,
            id,
            friend_id,
        } => {
            reqwest::Client::new()
                .post(url(
                    &server,
                    &format!("/users/{id}/friends/{friend_id}/remove"),
                )?)
                .bearer_auth(token)
                .send()
                .await?
                .error_for_status()?;
            println!("Friend removed");
        }
    }

    Ok(())
}

fn url(base: &str, path: &str) -> Result<Url, anyhow::Error> {
    Ok(Url::parse(base)?.join(path)?)
}

async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
    let state = SharedState::default();
    let router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/users/:id", get(get_user_graph))
        .route("/users/:id/friends/:friend_id", post(add_friend))
        .route("/users/:id/friends/:friend_id/remove", post(remove_friend))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        );

    println!("Running server on {addr}");
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterPayload,
    responses(
        (status = 200, description = "User registered"),
        (status = 400, description = "User already exists"),
    )
)]
async fn register_user(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<RegisterPayload>,
) -> Result<StatusCode, ApiError> {
    let mut state = state.lock().await;
    if state.names.contains_key(&payload.name) {
        return Err(ApiError::UserExists);
    }

    let id = Uuid::new_v4();
    let user = User {
        id,
        name: payload.name.clone(),
        password: payload.password,
        friends: HashSet::new(),
    };
    state.names.insert(user.name.clone(), id);
    state.users.insert(id, user);
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginPayload,
    responses(
        (status = 200, body = TokenResponse, description = "Token issued"),
        (status = 400, description = "Invalid credentials"),
    )
)]
async fn login_user(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<TokenResponse>, ApiError> {
    let mut state = state.lock().await;
    let user_id = state
        .names
        .get(&payload.name)
        .and_then(|id| state.users.get(id))
        .filter(|user| user.password == payload.password)
        .map(|user| user.id)
        .ok_or(ApiError::InvalidCredentials)?;

    let token = Uuid::new_v4().to_string();
    state.auth.tokens.insert(token.clone(), user_id);
    Ok(Json(TokenResponse { token }))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (status = 200, body = UserGraph, description = "User with friends"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    security(("token" = []))
)]
async fn get_user_graph(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
    _auth: AuthenticatedUser,
) -> Result<Json<UserGraph>, ApiError> {
    let id = Uuid::parse_str(&id).map_err(|_| ApiError::BadIdentifier)?;
    let state = state.lock().await;
    let user = state.users.get(&id).ok_or(ApiError::UserNotFound)?;
    let user_friends: Vec<PublicUser> = user
        .friends
        .iter()
        .filter_map(|friend_id| state.users.get(friend_id))
        .map(PublicUser::from)
        .collect();
    let graph = UserGraph {
        user: PublicUser::from(user),
        friends: user_friends,
    };
    Ok(Json(graph))
}

#[utoipa::path(
    post,
    path = "/users/{id}/friends/{friend_id}",
    responses(
        (status = 200, description = "Friend added"),
        (status = 400, description = "Invalid identifiers"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    security(("token" = []))
)]
async fn add_friend(
    State(state): State<Arc<Mutex<AppState>>>,
    Path((id, friend_id)): Path<(String, String)>,
    _auth: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let id = Uuid::parse_str(&id).map_err(|_| ApiError::BadIdentifier)?;
    let friend_id = Uuid::parse_str(&friend_id).map_err(|_| ApiError::BadIdentifier)?;

    let mut state = state.lock().await;
    let user = state.users.get_mut(&id).ok_or(ApiError::UserNotFound)?;
    let friend = state.users.get(&friend_id).ok_or(ApiError::UserNotFound)?;
    user.friends.insert(friend.id);
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/users/{id}/friends/{friend_id}/remove",
    responses(
        (status = 200, description = "Friend removed"),
        (status = 400, description = "Invalid identifiers"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    security(("token" = []))
)]
async fn remove_friend(
    State(state): State<Arc<Mutex<AppState>>>,
    Path((id, friend_id)): Path<(String, String)>,
    _auth: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let id = Uuid::parse_str(&id).map_err(|_| ApiError::BadIdentifier)?;
    let friend_id = Uuid::parse_str(&friend_id).map_err(|_| ApiError::BadIdentifier)?;

    let mut state = state.lock().await;
    let user = state.users.get_mut(&id).ok_or(ApiError::UserNotFound)?;
    user.friends.remove(&friend_id);
    Ok(StatusCode::OK)
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Simple REST API server and client")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run HTTP server
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        addr: SocketAddr,
    },
    /// Register a user via API
    Register {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        password: String,
    },
    /// Login and get a token
    Login {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        password: String,
    },
    /// Fetch a user with friends
    GetUser {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        id: String,
    },
    /// Add a friend to a user
    AddFriend {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        friend_id: String,
    },
    /// Remove a friend from a user
    RemoveFriend {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        friend_id: String,
    },
}
