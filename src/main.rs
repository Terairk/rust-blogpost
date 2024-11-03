use askama_axum::Template;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::path::Path;
use std::sync::Arc;
use tokio::{fs, net::TcpListener};
use tower_http::services::ServeDir;
use uuid::Uuid;

// Struct to hold application state
#[derive(Clone)]
struct AppState {
    pool: PgPool,
    upload_dir: String,
}

// Database model for a blog post
#[derive(sqlx::FromRow, Debug, Clone)]
struct BlogPost {
    id: i32,
    username: String,
    content: String,
    created_at: chrono::DateTime<Utc>,
    image_path: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
struct BlogPostView {
    id: i32,
    username: String,
    content: String,
    created_at: chrono::DateTime<Utc>,
    image_path: String,
    avatar_url: String,
}

// Conversion implementation
impl From<BlogPost> for BlogPostView {
    fn from(post: BlogPost) -> Self {
        BlogPostView {
            id: post.id,
            username: post.username,
            content: post.content,
            created_at: post.created_at,
            image_path: post.image_path.unwrap_or_default(),
            avatar_url: post.avatar_url.unwrap_or_default(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment variables
    dotenvy::dotenv().ok();

    // Create upload directory if it doesn't exist
    let upload_dir = "app/uploads";
    fs::create_dir_all(upload_dir).await?;

    // Set proper permissions for the upload directory
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(upload_dir).await?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(upload_dir, permissions).await?;
    }

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Initialize SQLx migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Application state
    let state = Arc::new(AppState {
        pool,
        upload_dir: upload_dir.to_string(),
    });

    // Create router
    let app = Router::new()
        .route("/", get(home_handler))
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

mod filters {

    use chrono::{DateTime, Utc};
    pub fn date(date: &DateTime<Utc>) -> ::askama::Result<String> {
        Ok(date.format("%Y-%m-%d %H:%M").to_string())
    }
}

// Template for the home page
#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    posts: Vec<BlogPostView>,
}

// Handler for the home page
async fn home_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, CreatePostError> {
    // Get all blog posts
    let posts = get_blog_posts(&state.pool)
        .await
        .map_err(|e| CreatePostError(format!("{e}")))?;

    // Render template
    let template = HomeTemplate {
        posts: posts.into_iter().map(BlogPostView::from).collect(),
    };
    let html = template
        .render()
        .map_err(|e| CreatePostError(format!("Template error: {e}")))?;

    Ok(Html(html))
}

#[derive(Debug)]
struct CreatePostError(String);

impl IntoResponse for CreatePostError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error creating post: {}", self.0),
        )
            .into_response()
    }
}

impl std::fmt::Display for CreatePostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CreatePostError {}

// Helper function to save uploaded file
async fn save_file(
    data: Vec<u8>,
    file_extension: &str,
    upload_dir: &str,
) -> Result<String, CreatePostError> {
    let file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
    let path = Path::new(upload_dir).join(&file_name);

    tokio::fs::write(&path, data)
        .await
        .map_err(|e| CreatePostError(format!("Failed to save file: {e}")))?;

    Ok(file_name)
}

// Helper function to download avatar
async fn download_avatar(avatar_url: &str, upload_dir: &str) -> Result<String, CreatePostError> {
    let response = reqwest::get(avatar_url)
        .await
        .map_err(|e| CreatePostError(format!("Failed to download avatar: {}", e)))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| CreatePostError(format!("Failed to read avatar bytes: {}", e)))?;

    let file_name = format!("{}.png", Uuid::new_v4());
    let path = Path::new(upload_dir).join(&file_name);

    tokio::fs::write(&path, bytes)
        .await
        .map_err(|e| CreatePostError(format!("Failed to save avatar: {}", e)))?;

    Ok(file_name)
}

// Handler for creating new posts
async fn create_post_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Redirect, CreatePostError> {
    let mut username = None;
    let mut content = None;
    let mut avatar_url = None;
    let mut image_path = None;

    // Process multipart form
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "username" => {
                username = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| CreatePostError(format!("Failed to read username: {e}")))?,
                )
            }
            "post_content" => {
                content = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| CreatePostError(format!("Failed to read content: {e}")))?,
                )
            }
            "avatar_url" => {
                let url = field
                    .text()
                    .await
                    .map_err(|e| CreatePostError(format!("Failed to read avatar URL: {e}")))?;
                if !url.is_empty() {
                    // Download and save avatar if URL is provided
                    let avatar_file_name = download_avatar(&url, &state.upload_dir).await?;
                    avatar_url = Some(format!("/app/uploads/{avatar_file_name}"));
                }
            }
            "post_image" => {
                if let Some(file_name) = field.file_name() {
                    if file_name.ends_with(".png") {
                        let data = field.bytes().await.map_err(|e| {
                            CreatePostError(format!("Failed to read image bytes: {e}"))
                        })?;

                        if !data.is_empty() {
                            // Save uploaded image
                            let image_file_name =
                                save_file(data.to_vec(), "png", &state.upload_dir).await?;
                            image_path = Some(format!("/app/uploads/{image_file_name}"));
                        }
                    }
                }
            }
            _ => (),
        }
    }

    // Validate required fields
    let username = username.ok_or_else(|| CreatePostError("Username is required".into()))?;
    let content = content.ok_or_else(|| CreatePostError("Content is required".into()))?;

    // Insert into database
    // Replace the existing create_post_handler query with this:
    sqlx::query(
        r#"
    INSERT INTO blog_posts (username, content, image_path, avatar_url)
    VALUES ($1, $2, $3, $4)
    "#,
    )
    .bind(&username)
    .bind(&content)
    .bind(image_path)
    .bind(avatar_url)
    .execute(&state.pool)
    .await
    .map_err(|e| CreatePostError(format!("Database error: {e}")))?;
    Ok(Redirect::to("/"))
}

// Helper function to get all blog posts
async fn get_blog_posts(pool: &PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
    sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}
