use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_graphql::{
    Context, EmptySubscription, ErrorExtensions, ID, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    response::Html,
    routing::{get, post},
};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Default)]
struct AppState {
    inner: Arc<Mutex<AppData>>,
}

#[derive(Default)]
struct AppData {
    users: HashMap<Uuid, UserRecord>,
    tokens: HashMap<String, Uuid>,
}

#[derive(Clone)]
struct UserRecord {
    id: Uuid,
    name: String,
    password_hash: String,
    friends: HashSet<Uuid>,
}

impl UserRecord {
    fn verify_password(&self, password: &str) -> bool {
        self.password_hash == hash_password(password)
    }
}

#[derive(Clone)]
struct AuthedUser {
    id: Uuid,
}

#[derive(SimpleObject)]
struct AuthPayload {
    token: String,
    user: User,
}

#[derive(Clone)]
struct User {
    id: Uuid,
}

#[Object]
impl User {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let state = ctx.data::<AppState>()?;
        let data = state.inner.lock().await;
        let user = data
            .users
            .get(&self.id)
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;
        Ok(user.name.clone())
    }

    async fn friends(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let state = ctx.data::<AppState>()?;
        let data = state.inner.lock().await;
        let user = data
            .users
            .get(&self.id)
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;

        Ok(user
            .friends
            .iter()
            .filter_map(|id| data.users.get(id).map(|_| User { id: *id }))
            .collect())
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "User id or name")] id: Option<ID>,
        name: Option<String>,
    ) -> async_graphql::Result<User> {
        let auth = ctx.data::<Option<AuthedUser>>()?;
        if auth.is_none() {
            return Err(async_graphql::Error::new("Authorization required")
                .extend_with(|_, e| e.set("code", "UNAUTHORIZED")));
        }

        let state = ctx.data::<AppState>()?;
        let data = state.inner.lock().await;

        if let Some(id) = id {
            let uuid = parse_uuid(&id)?;
            data.users
                .get(&uuid)
                .map(|u| User { id: u.id })
                .ok_or_else(|| async_graphql::Error::new("User not found"))
        } else if let Some(name) = name {
            data.users
                .values()
                .find(|u| u.name == name)
                .map(|u| User { id: u.id })
                .ok_or_else(|| async_graphql::Error::new("User not found"))
        } else {
            Err(async_graphql::Error::new("Specify id or name"))
        }
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        name: String,
        password: String,
    ) -> async_graphql::Result<User> {
        let state = ctx.data::<AppState>()?;
        let mut data = state.inner.lock().await;

        if data.users.values().any(|u| u.name == name) {
            return Err(async_graphql::Error::new("User name already taken"));
        }

        let user = UserRecord {
            id: Uuid::new_v4(),
            name,
            password_hash: hash_password(&password),
            friends: HashSet::new(),
        };

        let id = user.id;
        data.users.insert(id, user);
        Ok(User { id })
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        name: String,
        password: String,
    ) -> async_graphql::Result<AuthPayload> {
        let state = ctx.data::<AppState>()?;
        let mut data = state.inner.lock().await;
        let user_id = data
            .users
            .values()
            .find(|u| u.name == name && u.verify_password(&password))
            .map(|u| u.id)
            .ok_or_else(|| async_graphql::Error::new("Invalid credentials"))?;

        let token = Uuid::new_v4().to_string();
        data.tokens.insert(token.clone(), user_id);

        Ok(AuthPayload {
            token,
            user: User { id: user_id },
        })
    }

    async fn add_friend(&self, ctx: &Context<'_>, friend_id: ID) -> async_graphql::Result<User> {
        let user_id = ensure_authorized(ctx)?;
        let friend_uuid = parse_uuid(&friend_id)?;
        let state = ctx.data::<AppState>()?;
        let mut data = state.inner.lock().await;

        let friend_exists = data.users.contains_key(&friend_uuid);
        if !friend_exists {
            return Err(async_graphql::Error::new("Friend not found"));
        }

        let user = data
            .users
            .get_mut(&user_id)
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;
        user.friends.insert(friend_uuid);
        Ok(User { id: friend_uuid })
    }

    async fn remove_friend(&self, ctx: &Context<'_>, friend_id: ID) -> async_graphql::Result<User> {
        let user_id = ensure_authorized(ctx)?;
        let friend_uuid = parse_uuid(&friend_id)?;
        let state = ctx.data::<AppState>()?;
        let mut data = state.inner.lock().await;

        let user = data
            .users
            .get_mut(&user_id)
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;
        if !user.friends.remove(&friend_uuid) {
            return Err(async_graphql::Error::new("Friend not in list"));
        }

        Ok(User { id: friend_uuid })
    }
}

fn parse_uuid(id: &ID) -> async_graphql::Result<Uuid> {
    Uuid::parse_str(id.as_str()).map_err(|_| async_graphql::Error::new("Invalid identifier format"))
}

fn ensure_authorized(ctx: &Context<'_>) -> async_graphql::Result<Uuid> {
    ctx.data::<Option<AuthedUser>>()?
        .as_ref()
        .map(|u| u.id)
        .ok_or_else(|| async_graphql::Error::new("Authorization required"))
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn graphiql() -> Html<String> {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}

async fn graphql_handler(
    State(server_state): State<ServerState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner().data(server_state.state.clone());
    let auth = extract_auth(headers, &server_state.state).await;
    request = request.data(auth);
    server_state.schema.execute(request).await.into()
}

async fn extract_auth(headers: HeaderMap, state: &AppState) -> Option<AuthedUser> {
    if let Some(token_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(raw_value) = token_header.to_str() {
            if let Some(token) = raw_value.strip_prefix("Bearer ") {
                let data = state.inner.lock().await;
                if let Some(id) = data.tokens.get(token).copied() {
                    return Some(AuthedUser { id });
                }
            }
        }
    }

    None
}

type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Clone)]
struct ServerState {
    schema: AppSchema,
    state: AppState,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();
    let state = AppState::default();
    let server_state = ServerState { schema, state };

    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .with_state(server_state);

    println!("GraphQL server running at http://127.0.0.1:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Unable to bind to port");
    axum::serve(listener, app).await.unwrap();
}
