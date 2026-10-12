use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use std::fs;
use tera::Context;

pub async fn home(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let mut context = Context::new();
    let recent: Vec<_> = state.posts.iter().take(4).collect();
    context.insert("posts", &recent);
    let rendered = state
        .tera
        .render("index.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

pub async fn article_list(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let mut context = Context::new();
    context.insert("posts", &state.posts);
    let rendered = state
        .tera
        .render("articles.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

pub async fn article_view(
    State(state): State<AppState>,
    Path(article_id): Path<u32>,
) -> Result<Html<String>, StatusCode> {
    let post = state
        .posts
        .iter()
        .find(|p| p.id == article_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let content_path = format!("templates/articles/{}.html", post.filename);
    let content = fs::read_to_string(&content_path).map_err(|_| StatusCode::NOT_FOUND)?;

    let total = state.posts.len();

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("content", &content);
    context.insert("total", &total);

    let rendered = state
        .tera
        .render("article.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(rendered))
}

pub async fn about(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let context = Context::new();
    let rendered = state
        .tera
        .render("about.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::load_posts;
    use tera::Tera;

    fn test_state() -> AppState {
        let mut tera = Tera::new();
        tera.load_from_glob("templates/**/*")
            .expect("Failed to load templates");
        AppState {
            posts: load_posts(),
            tera,
        }
    }

    #[tokio::test]
    async fn home_renders_recent_posts() {
        let result = home(State(test_state())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn article_list_renders_all_posts() {
        let result = article_list(State(test_state())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn article_view_returns_post_for_known_id() {
        let state = test_state();
        let known_id = state.posts[0].id;

        let result = article_view(State(state), Path(known_id)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn article_view_returns_404_for_unknown_id() {
        let result = article_view(State(test_state()), Path(u32::MAX)).await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn about_renders() {
        let result = about(State(test_state())).await;
        assert!(result.is_ok());
    }
}
