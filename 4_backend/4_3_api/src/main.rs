use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use clap::{Parser, Subcommand};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct Task {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
struct CreateTask {
    title: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
struct UpdateTask {
    title: Option<String>,
    completed: Option<bool>,
}

#[derive(Default)]
struct TaskStore {
    tasks: Vec<Task>,
    next_id: u64,
}

impl TaskStore {
    fn insert(&mut self, title: String) -> Task {
        let task = Task {
            id: self.next_id,
            title,
            completed: false,
        };
        self.next_id += 1;
        self.tasks.push(task.clone());
        task
    }

    fn update(&mut self, id: u64, update: UpdateTask) -> Option<Task> {
        self.tasks
            .iter_mut()
            .find(|task| task.id == id)
            .map(|task| {
                if let Some(title) = update.title {
                    task.title = title;
                }
                if let Some(completed) = update.completed {
                    task.completed = completed;
                }
                task.clone()
            })
    }

    fn delete(&mut self, id: u64) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        before != self.tasks.len()
    }
}

#[derive(Clone, Default)]
struct AppState(Arc<RwLock<TaskStore>>);

#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, description = "List existing tasks", body = [Task])
    )
)]
async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let guard = state.0.read().await;
    Json(guard.tasks.clone())
}

#[utoipa::path(
    post,
    path = "/tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Create a new task", body = Task)
    )
)]
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> impl IntoResponse {
    let mut guard = state.0.write().await;
    let task = guard.insert(payload.title);
    (StatusCode::CREATED, Json(task))
}

#[utoipa::path(
    put,
    path = "/tasks/{id}",
    request_body = UpdateTask,
    params(("id" = u64, Path, description = "Task id")),
    responses(
        (status = 200, description = "Updated task", body = Task),
        (status = 404, description = "Task not found")
    )
)]
async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTask>,
) -> impl IntoResponse {
    let mut guard = state.0.write().await;
    match guard.update(id, payload) {
        Some(task) => (StatusCode::OK, Json(task)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    params(("id" = u64, Path, description = "Task id")),
    responses(
        (status = 204, description = "Task removed"),
        (status = 404, description = "Task not found")
    )
)]
async fn delete_task(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    let mut guard = state.0.write().await;
    if guard.delete(id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(list_tasks, create_task, update_task, delete_task),
    components(schemas(Task, CreateTask, UpdateTask)),
    tags((name = "tasks", description = "Simple task management"))
)]
struct ApiDoc;

#[derive(Parser)]
#[command(author, version, about = "REST API server with CLI client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the HTTP server
    Server {
        /// Address to bind the server to (e.g. 127.0.0.1:8080)
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: SocketAddr,
    },
    /// Interact with the server REST API as a thick client
    Client {
        /// Base URL of the server (e.g. http://127.0.0.1:8080)
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: Url,
        #[command(subcommand)]
        action: ClientCommand,
    },
}

#[derive(Subcommand)]
enum ClientCommand {
    /// List all tasks
    List,
    /// Create a new task
    Add { title: String },
    /// Update an existing task
    Update {
        id: u64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        completed: Option<bool>,
    },
    /// Delete a task
    Delete { id: u64 },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server { addr } => {
            let state = AppState::default();
            let api = Router::new()
                .route("/tasks", get(list_tasks).post(create_task))
                .route("/tasks/:id", put(update_task).delete(delete_task))
                .with_state(state.clone());

            let app = Router::new()
                .merge(api)
                .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
                .fallback(|| async { StatusCode::NOT_FOUND });

            println!("Server is running on http://{addr}");
            axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
        }
        Commands::Client { server, action } => match action {
            ClientCommand::List => {
                let tasks: Vec<Task> = reqwest::Client::new()
                    .get(server.join("tasks")?)
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                if tasks.is_empty() {
                    println!("No tasks found");
                } else {
                    for task in tasks {
                        println!(
                            "[{}] {} - {}",
                            task.id,
                            task.title,
                            if task.completed { "done" } else { "pending" }
                        );
                    }
                }
            }
            ClientCommand::Add { title } => {
                let task: Task = reqwest::Client::new()
                    .post(server.join("tasks")?)
                    .json(&CreateTask { title })
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                println!("Created task #{}: {}", task.id, task.title);
            }
            ClientCommand::Update {
                id,
                title,
                completed,
            } => {
                let task: Task = reqwest::Client::new()
                    .put(server.join(&format!("tasks/{id}"))?)
                    .json(&UpdateTask { title, completed })
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                println!(
                    "Updated task #{}: {} (completed: {})",
                    task.id, task.title, task.completed
                );
            }
            ClientCommand::Delete { id } => {
                reqwest::Client::new()
                    .delete(server.join(&format!("tasks/{id}"))?)
                    .send()
                    .await?
                    .error_for_status()?;
                println!("Deleted task #{id}");
            }
        },
    }
    Ok(())
}
