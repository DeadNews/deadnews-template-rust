use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, runners::AsyncRunner};
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

// Test health check function with a mock server
#[tokio::test]
async fn test_health_check_success() -> anyhow::Result<()> {
    use tokio::net::TcpListener;

    // Create a simple mock server that returns 200 OK
    let mock_app = Router::new().route("/health", get(|| async { "OK" }));

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // Start the mock server in background
    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let health_url = format!("http://{addr}/health");
    let result = health_check(&health_url).await;

    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_health_check_failure() -> anyhow::Result<()> {
    // Test with a non-existent URL
    let result = health_check("http://127.0.0.1:1/nonexistent").await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_health_check_http_error() -> anyhow::Result<()> {
    use tokio::net::TcpListener;

    // Create a mock server that returns 500 error
    let mock_app = Router::new().route(
        "/health",
        get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "Server Error") }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let health_url = format!("http://{addr}/health");
    let result = health_check(&health_url).await;

    assert!(result.is_err());
    Ok(())
}

// Test database error handling
#[tokio::test]
async fn test_database_test_handler_error() -> anyhow::Result<()> {
    // Create a test that simulates a database error
    // by closing the connection pool in the existing working test
    let ctx = TestContext::new().await?;

    // Close all connections in the pool to simulate database failure
    ctx.pool.close().await;

    let app_state = AppState { db: ctx.pool };
    let app = Router::new()
        .route("/test", get(database_test_handler))
        .with_state(app_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty())?)
        .await?;

    // Should return 500 error when database is unavailable
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;
    assert_eq!(json["error"], "Internal server error");

    Ok(())
}

// Test CLI argument parsing
#[test]
fn test_args_parsing() {
    use clap::Parser;

    // Test default args (no health check)
    let args = Args::try_parse_from(["test"]).unwrap();
    assert!(args.healthcheck.is_none());

    // Test with health check URL
    let args = Args::try_parse_from(["test", "--healthcheck", "http://example.com"]).unwrap();
    assert_eq!(args.healthcheck, Some("http://example.com".to_string()));
}

// Test struct serialization/deserialization
#[test]
fn test_database_info_serialization() -> anyhow::Result<()> {
    let db_info = DatabaseInfo {
        database: "test_db".to_string(),
        version: "PostgreSQL 13.0".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&db_info)?;
    assert!(json.contains("test_db"));
    assert!(json.contains("PostgreSQL 13.0"));

    // Test deserialization
    let deserialized: DatabaseInfo = serde_json::from_str(&json)?;
    assert_eq!(deserialized.database, "test_db");
    assert_eq!(deserialized.version, "PostgreSQL 13.0");

    Ok(())
}

// Test AppState clone
#[tokio::test]
async fn test_app_state_clone() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    let app_state = AppState {
        db: ctx.pool.clone(),
    };

    // Test that AppState can be cloned
    let cloned_state = app_state.clone();

    // Both should be able to get database info
    let db_info1 = get_database_info(&app_state.db).await?;
    let db_info2 = get_database_info(&cloned_state.db).await?;

    assert_eq!(db_info1.database, db_info2.database);
    Ok(())
}
