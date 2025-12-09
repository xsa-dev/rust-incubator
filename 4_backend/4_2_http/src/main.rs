use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::signal;

#[derive(Parser)]
#[command(author, version, about = "Thin client/server demo", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the HTTP server that processes commands.
    Server {
        /// Port to listen on.
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Send a raw command string to the server.
    Client {
        /// Server address, e.g. http://localhost:3000
        #[arg(short, long, default_value = "http://localhost:3000")]
        server: String,
        /// Command to execute on the server.
        #[arg(last = true)]
        command: Vec<String>,
    },
}

#[derive(Debug, Default)]
struct Store {
    users: HashMap<u64, User>,
    roles: HashMap<String, Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    roles: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Role {
    slug: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct CommandResponse {
    status: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => run_server(port).await?,
        Commands::Client { server, command } => run_client(&server, command).await?,
    }

    Ok(())
}

async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(Store::default()));

    let app = Router::new()
        .route("/command", post(handle_command))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("Shutting down server");
}

async fn run_client(server: &str, command: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if command.is_empty() {
        eprintln!("Please provide a command to send to the server");
        std::process::exit(1);
    }

    let body = command.join(" ");
    let url = format!("{server}/command");
    let response = reqwest::Client::new().post(&url).body(body).send().await?;

    let status = response.status();
    let text = response.text().await?;
    println!("{}\n{}", status, text);
    Ok(())
}

async fn handle_command(
    State(state): State<Arc<Mutex<Store>>>,
    body: String,
) -> (StatusCode, Json<CommandResponse>) {
    let mut store = state.lock().expect("store mutex poisoned");
    let result = execute_command(&mut store, body.trim());

    let status = if result.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };

    (status, Json(result))
}

fn execute_command(store: &mut Store, input: &str) -> CommandResponse {
    let mut parts = input.split_whitespace();
    let Some(cmd) = parts.next() else {
        return CommandResponse {
            status: "error".into(),
            message: "Empty command".into(),
            data: None,
        };
    };

    match cmd {
        "create_user" => {
            let id = match parts.next().and_then(|p| p.parse::<u64>().ok()) {
                Some(id) => id,
                None => {
                    return error_response("Usage: create_user <id> <name> [role_slug]");
                }
            };
            let name = match parts.next() {
                Some(name) => name.to_string(),
                None => return error_response("Usage: create_user <id> <name> [role_slug]"),
            };
            let role = parts.next();

            if store.users.contains_key(&id) {
                return error_response("User with provided id already exists");
            }

            let mut roles = HashSet::new();
            if let Some(role_slug) = role {
                if store.roles.contains_key(role_slug) {
                    roles.insert(role_slug.to_string());
                } else {
                    return error_response("Unknown role slug provided");
                }
            }

            store.users.insert(id, User { id, name, roles });

            CommandResponse {
                status: "ok".into(),
                message: "User created".into(),
                data: None,
            }
        }
        "delete_user" => {
            let Some(id) = parts.next().and_then(|p| p.parse::<u64>().ok()) else {
                return error_response("Usage: delete_user <id>");
            };
            if store.users.remove(&id).is_some() {
                CommandResponse {
                    status: "ok".into(),
                    message: format!("User {id} deleted"),
                    data: None,
                }
            } else {
                error_response("User not found")
            }
        }
        "create_role" => {
            let Some(slug) = parts.next() else {
                return error_response("Usage: create_role <slug> <name>");
            };
            let Some(name) = parts.next() else {
                return error_response("Usage: create_role <slug> <name>");
            };

            if store.roles.contains_key(slug) {
                return error_response("Role with provided slug already exists");
            }

            store.roles.insert(
                slug.to_string(),
                Role {
                    slug: slug.to_string(),
                    name: name.to_string(),
                },
            );

            CommandResponse {
                status: "ok".into(),
                message: "Role created".into(),
                data: None,
            }
        }
        "assign_role" => {
            let Some(id) = parts.next().and_then(|p| p.parse::<u64>().ok()) else {
                return error_response("Usage: assign_role <user_id> <role_slug>");
            };
            let Some(role_slug) = parts.next() else {
                return error_response("Usage: assign_role <user_id> <role_slug>");
            };

            let Some(user) = store.users.get_mut(&id) else {
                return error_response("User not found");
            };
            if store.roles.contains_key(role_slug) {
                user.roles.insert(role_slug.to_string());
                CommandResponse {
                    status: "ok".into(),
                    message: "Role assigned".into(),
                    data: None,
                }
            } else {
                error_response("Unknown role")
            }
        }
        "unassign_role" => {
            let Some(id) = parts.next().and_then(|p| p.parse::<u64>().ok()) else {
                return error_response("Usage: unassign_role <user_id> <role_slug>");
            };
            let Some(role_slug) = parts.next() else {
                return error_response("Usage: unassign_role <user_id> <role_slug>");
            };

            let Some(user) = store.users.get_mut(&id) else {
                return error_response("User not found");
            };

            if user.roles.remove(role_slug) {
                CommandResponse {
                    status: "ok".into(),
                    message: "Role unassigned".into(),
                    data: None,
                }
            } else {
                error_response("Role not assigned to user")
            }
        }
        "list_roles" => CommandResponse {
            status: "ok".into(),
            message: "Roles list".into(),
            data: Some(json!(store.roles.values().cloned().collect::<Vec<_>>())),
        },
        "list_users" => CommandResponse {
            status: "ok".into(),
            message: "Users list".into(),
            data: Some(json!(store.users.values().cloned().collect::<Vec<_>>())),
        },
        "show_user" => {
            let Some(id) = parts.next().and_then(|p| p.parse::<u64>().ok()) else {
                return error_response("Usage: show_user <id>");
            };

            if let Some(user) = store.users.get(&id) {
                CommandResponse {
                    status: "ok".into(),
                    message: "User details".into(),
                    data: Some(json!(user)),
                }
            } else {
                error_response("User not found")
            }
        }
        _ => error_response("Unknown command"),
    }
}

fn error_response(msg: &str) -> CommandResponse {
    CommandResponse {
        status: "error".into(),
        message: msg.into(),
        data: None,
    }
}
