mod handlers;
mod models;

use crate::handlers::{create::create_post_handler, retrieve::home_handler};
use crate::models::AppState;
use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::{fs, net::TcpListener};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize environment variables
    dotenvy::dotenv().ok();

    // Create upload directory if it doesn't exist
    let upload_dir = "app/uploads";
    fs::create_dir_all(upload_dir)
        .await
        .context("Failed to create upload directory")?;

    // Database connection
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to Postgres")?;

    // Initialize SQLx migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run SQLx migrations")?;

    // Application state
    let state = Arc::new(AppState {
        pool,
        upload_dir: upload_dir.to_string(),
    });

    // Create router
    let app = Router::new()
        .route("/home", get(home_handler))
        .route("/post", post(create_post_handler))
        .nest_service("/app/uploads", ServeDir::new(upload_dir))
        .with_state(state);

    // Start server

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
