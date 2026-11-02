use crate::models::Post;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use std::collections::HashMap;
use tera::Context;

/// Article filenames come from data.json and are joined into a filesystem
/// path. Enforce the invariant so a bad entry can never become path
/// traversal, even if data.json is later sourced from somewhere less trusted.
fn valid_filename(s: &str) -> bool {
    !s.is_empty()
        && s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Reads every article body once, at startup. Panics on an unsafe filename or
/// a missing file, so a bad deploy fails at boot instead of on a visitor's
/// request. Called from AppState::load.
pub fn load_article_bodies(posts: &[Post]) -> HashMap<u32, String> {
    posts
        .iter()
        .map(|p| {
            assert!(
                valid_filename(&p.filename),
                "unsafe filename in post data: {:?}",
                p.filename
            );
            let path = format!("templates/articles/{}.html", p.filename);
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read article body {path}: {e}"));
            (p.id, body)
        })
        .collect()
}

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
    // `&*` derefs through Arc, which avoids needing serde's "rc" feature.
    context.insert("posts", &*state.posts);
    let rendered = state
        .tera
        .render("articles.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

/// The id is taken as a String and parsed here on purpose. With `Path<u32>`,
/// axum builds its default rejection before the handler runs, and that body
/// echoes the raw, unescaped path segment back to the client.
pub async fn article_view(
    State(state): State<AppState>,
    Path(raw_id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let article_id: u32 = raw_id.parse().map_err(|_| StatusCode::NOT_FOUND)?;

    // One canonical URL per article: reject "+5", "007", and similar.
    if article_id.to_string() != raw_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let post = state
        .posts
        .iter()
        .find(|p| p.id == article_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Bodies were loaded and validated at startup. No filesystem access, and
    // no path is built from anything on the request path.
    let content = state.bodies.get(&article_id).ok_or(StatusCode::NOT_FOUND)?;

    let total = state.posts.len();

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("content", content);
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

    fn test_state() -> AppState {
        AppState::load()
    }

    // Handler-level tests. Router-level tests (routing, extractors, security
    // headers) live in main.rs next to the real app() router.

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

        let result = article_view(State(state), Path(known_id.to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn article_view_returns_404_for_unknown_id() {
        let result = article_view(State(test_state()), Path(u32::MAX.to_string())).await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn article_view_returns_404_for_non_numeric_id() {
        for bad in [
            "abc",
            "12<script>alert(1)</script>",
            "-1",
            "4294967296",
            "+1",
            "007",
            "",
        ] {
            let result = article_view(State(test_state()), Path(bad.to_string())).await;
            assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND, "input: {bad:?}");
        }
    }

    #[tokio::test]
    async fn about_renders() {
        let result = about(State(test_state())).await;
        assert!(result.is_ok());
    }

    #[test]
    fn valid_filename_accepts_slugs_and_rejects_traversal() {
        assert!(valid_filename("my-post_1"));
        assert!(!valid_filename(""));
        assert!(!valid_filename("../etc/passwd"));
        assert!(!valid_filename("a/b"));
        assert!(!valid_filename("a\\b"));
        assert!(!valid_filename("a.html"));
        assert!(!valid_filename("a\0b"));
    }

    #[test]
    fn every_post_has_a_loaded_body() {
        // load_article_bodies already panics on unsafe filenames or missing
        // files, so this also guards data.json against bad entries in CI.
        let posts = load_posts();
        let bodies = load_article_bodies(&posts);
        assert_eq!(bodies.len(), posts.len(), "duplicate post ids in data.json?");
        for post in &posts {
            let body = bodies.get(&post.id).expect("missing body");
            assert!(!body.is_empty(), "empty article body for id {}", post.id);
        }
    }
}