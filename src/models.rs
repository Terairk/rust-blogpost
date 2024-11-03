use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPool, FromRow};

// Struct to hold application state
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub upload_dir: String,
}

// Database model for a blog post
#[derive(FromRow, Debug, Clone)]
pub struct BlogPost {
    pub id: i32,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub image_path: Option<String>,
    pub avatar_url: Option<String>,
}
// Used for templating
#[allow(dead_code)]
pub struct BlogPostView {
    pub id: i32,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub image_path: String,
    pub avatar_url: String,
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
