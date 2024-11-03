use crate::handlers::BlogError;
use crate::models::{AppState, BlogPost, BlogPostView};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;
use std::sync::Arc;

// Template for the home page
#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    posts: Vec<BlogPostView>,
}

// Handler for the home page
pub async fn home_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, BlogError> {
    // Get all blog posts
    let posts = get_blog_posts(&state.pool).await?;

    // Render template
    let template = HomeTemplate {
        posts: posts.into_iter().map(BlogPostView::from).collect(),
    };
    let html = template
        .render()
        .map_err(|e| BlogError::Template(e.to_string()))?;

    Ok(Html(html))
}

// Helper function to get all blog posts
async fn get_blog_posts(pool: &PgPool) -> Result<Vec<BlogPost>, BlogError> {
    sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(BlogError::Database)
}
