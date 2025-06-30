use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;

use super::*;

struct TestContext {
    _container: ContainerAsync<Postgres>,
    pool: PgPool,
    app: Router,
}

impl TestContext {
    async fn new() -> anyhow::Result<Self> {
        // Start PostgreSQL container
        let container = Postgres::default()
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start container: {}", e))?;

        // Get host and port dynamically
        let host = container
            .get_host()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get host: {}", e))?;
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get port: {}", e))?;

        // Use the default credentials from the PostgreSQL module
        let connection_string = format!("postgres://postgres:postgres@{host}:{port}/postgres");

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        let app_state = AppState { db: pool.clone() };

        let app = Router::new()
            .route("/", get(index))
            .route("/health", get(health_check_handler))
            .route("/test", get(database_test_handler))
            .with_state(app_state);

        Ok(TestContext {
            _container: container,
            pool,
            app,
        })
    }
}

#[tokio::test]
async fn test_index() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    let response = ctx
        .app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let body_str = String::from_utf8(body.to_vec())?;
    assert_eq!(body_str, "Hello world!");

    Ok(())
}

#[tokio::test]
async fn test_health_check_handler() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    let response = ctx
        .app
        .clone()
        .oneshot(Request::builder().uri("/health").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;
    assert_eq!(json, "Healthy!");

    Ok(())
}

#[tokio::test]
async fn test_database_test_handler() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    let response = ctx
        .app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;

    // Verify expected fields are present
    assert!(json.get("database").is_some());
    assert!(json.get("version").is_some());

    // Verify database name is correct
    assert_eq!(json["database"], "postgres");

    // Verify version contains PostgreSQL
    let version = json["version"].as_str().unwrap();
    assert!(version.contains("PostgreSQL"));

    Ok(())
}

#[tokio::test]
async fn test_get_database_info() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    let db_info = get_database_info(&ctx.pool).await?;

    assert_eq!(db_info.database, "postgres");
    assert!(db_info.version.contains("PostgreSQL"));

    Ok(())
}
