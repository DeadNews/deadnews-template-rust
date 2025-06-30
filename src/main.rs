use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, net::SocketAddr};
use tracing::info;

#[derive(Parser)]
#[command(name = "deadnews-template-rust")]
#[command(about = "A Rust web service template")]
struct Args {
    /// Perform a health check against the given URL and exit
    #[arg(long)]
    healthcheck: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Serialize, Deserialize)]
struct DatabaseInfo {
    database: String,
    version: String,
}

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt().json().init();

    let args = Args::parse();

    // Handle health check mode
    if let Some(url) = args.healthcheck {
        match health_check(&url).await {
            Ok(_) => {
                println!("Health check succeeded");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Health check failed: {e}");
                std::process::exit(1);
            }
        }
    }

    // Get port from environment
    let port: u16 = env::var("SERVICE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000);

    // Get database DSN from environment
    let database_url = env::var("SERVICE_DSN").unwrap_or_else(|_| {
        tracing::error!("SERVICE_DSN environment variable is required");
        std::process::exit(1);
    });

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create database pool: {}", e);
            std::process::exit(1);
        });

    let app_state = AppState { db: pool };

    // Build the application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health_check_handler))
        .route("/test", get(database_test_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting HTTP server at http://{}", addr);

    // Run the app with hyper
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        });

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    });
}

async fn health_check(url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Health check failed with status: {}",
            response.status()
        ))
    }
}

async fn index() -> &'static str {
    "Hello world!"
}

async fn health_check_handler() -> impl IntoResponse {
    Json(json!("Healthy!"))
}

async fn database_test_handler(State(state): State<AppState>) -> impl IntoResponse {
    match get_database_info(&state.db).await {
        Ok(db_info) => Json(db_info).into_response(),
        Err(e) => {
            tracing::error!("Failed to get database info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

async fn get_database_info(pool: &PgPool) -> anyhow::Result<DatabaseInfo> {
    let mut conn = pool.acquire().await?;

    // Get database name
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut *conn)
        .await?;

    // Get database version
    let version: String = sqlx::query_scalar("SELECT version()")
        .fetch_one(&mut *conn)
        .await?;

    Ok(DatabaseInfo { database, version })
}

#[cfg(test)]
mod test;
