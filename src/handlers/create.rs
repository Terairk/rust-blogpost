use crate::handlers::BlogError;
use crate::models::AppState;
use axum::{
    extract::{Multipart, State},
    response::Redirect,
};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

// Helper function to save uploaded file
async fn save_file(
    data: Vec<u8>,
    file_extension: &str,
    upload_dir: &str,
) -> Result<String, BlogError> {
    let file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
    let path = Path::new(upload_dir).join(&file_name);

    tokio::fs::write(&path, data)
        .await
        .map_err(|e| BlogError::FileOperation(format!("Failed to save file: {e}")))?;

    Ok(file_name)
}

// Helper function to download avatar
async fn download_avatar(avatar_url: &str, upload_dir: &str) -> Result<String, BlogError> {
    let response = reqwest::get(avatar_url)
        .await
        .map_err(|e| BlogError::Network(format!("Failed to save avatar: {e}")))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| BlogError::FileOperation(format!("Failed to read avatar bytes: {e}")))?;

    let file_name = format!("{}.png", Uuid::new_v4());
    let path = Path::new(upload_dir).join(&file_name);

    tokio::fs::write(&path, bytes)
        .await
        .map_err(|e| BlogError::FileOperation(format!("Failed to save avatar to fs: {e}")))?;

    Ok(file_name)
}

// Handler for creating new posts
pub async fn create_post_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Redirect, BlogError> {
    let mut username = None;
    let mut content = None;
    let mut avatar_url = None;
    let mut image_path = None;

    // Process multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| BlogError::Form(e.to_string()))?
    {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "username" => {
                username = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| BlogError::Form(format!("Failed to read username: {e}")))?,
                )
            }
            "post_content" => {
                content =
                    Some(field.text().await.map_err(|e| {
                        BlogError::Form(format!("Failed to read post content: {e}"))
                    })?)
            }
            "avatar_url" => {
                let url = field
                    .text()
                    .await
                    .map_err(|e| BlogError::Form(format!("Failed to read avatar_url: {e}")))?;
                if !url.is_empty() {
                    // Download and save avatar if URL is provided
                    let avatar_file_name = download_avatar(&url, &state.upload_dir).await?;
                    avatar_url = Some(format!("{}/{avatar_file_name}", &state.upload_dir));
                }
            }
            "post_image" => {
                if let Some(file_name) = field.file_name() {
                    if file_name.ends_with(".png") {
                        let data = field.bytes().await.map_err(|e| {
                            BlogError::Form(format!("Failed to read image data: {e}"))
                        })?;

                        if !data.is_empty() {
                            // Save uploaded image
                            let image_file_name =
                                save_file(data.to_vec(), "png", &state.upload_dir).await?;
                            image_path = Some(format!("{}/{image_file_name}", &state.upload_dir));
                        }
                    }
                }
            }
            _ => (),
        }
    }

    // Validate required fields
    let username = username.ok_or_else(|| BlogError::Validation("Username is required".into()))?;
    let content = content.ok_or_else(|| BlogError::Validation("Content is required".into()))?;

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
    .map_err(BlogError::Database)?;

    Ok(Redirect::to("/home"))
}
