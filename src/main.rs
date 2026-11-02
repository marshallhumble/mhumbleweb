use axum::{Router, routing::get};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tera::Tera;
use tokio::net::TcpListener;

mod handlers;
mod middleware;
mod models;

use handlers::{about, article_list, article_view, home, load_article_bodies};
use middleware::apply_security_headers;
use models::load_posts;

/// Shared, read-only application state. Axum clones this for every request,
/// so every field is behind an Arc: a clone is three pointer copies instead
/// of a deep copy of all posts, templates, and article bodies.
#[derive(Clone)]
pub struct AppState {
    pub posts: Arc<Vec<models::Post>>,
    pub tera: Arc<Tera>,
    pub bodies: Arc<HashMap<u32, String>>,
}

impl AppState {
    /// Loads templates, posts, and article bodies. Panics on bad content so a
    /// broken deploy fails at boot instead of on a visitor's request.
    fn load() -> Self {
        let mut tera = Tera::new();
        tera.load_from_glob("templates/**/*")
            .expect("Failed to load templates");

        let posts = load_posts();
        let bodies = load_article_bodies(&posts);

        AppState {
            posts: Arc::new(posts),
            tera: Arc::new(tera),
            bodies: Arc::new(bodies),
        }
    }
}

fn origin_secrets() -> Vec<Vec<u8>> {
    let secrets = std::env::var("ORIGIN_SECRET")
        .map(|raw| middleware::parse_secrets(&raw))
        .unwrap_or_default();

    // Fail closed in production: a missing, misnamed, or empty secret must not unlock fly.dev.
    if secrets.is_empty() && std::env::var("FLY_APP_NAME").is_ok() {
        panic!("ORIGIN_SECRET must be set to a non-empty value when running on Fly");
    }
    secrets // empty locally means unlocked
}

fn app(state: AppState, secrets: Vec<Vec<u8>>) -> Router {
    let router = Router::new()
        .route("/", get(home))
        .route("/health", get(|| async { "OK" }))
        .route("/articles", get(article_list))
        .route("/articles/{article_id}", get(article_view))
        .route("/about", get(about))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state);

    // Header layers wrap the lock so its 404 carries them.
    apply_security_headers(middleware::lock_origin(router, secrets))
}

#[tokio::main]
async fn main() {
    let state = AppState::load();

    let app = app(state, origin_secrets());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.expect("Server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    const TEST_SECRET: &str = "0123456789abcdef";

    fn test_state() -> AppState {
        AppState::load()
    }

    fn locked() -> Vec<Vec<u8>> {
        middleware::parse_secrets(TEST_SECRET)
    }

    async fn send(
        secrets: Vec<Vec<u8>>,
        uri: &str,
        origin_header: Option<&str>,
    ) -> axum::http::Response<Body> {
        let mut builder = Request::builder().uri(uri);
        if let Some(value) = origin_header {
            builder = builder.header("x-origin-verify", value);
        }
        app(test_state(), secrets)
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    /// Unlocked router, as in local dev. Existing tests use this.
    async fn get(uri: &str) -> axum::http::Response<Body> {
        send(Vec::new(), uri, None).await
    }

    async fn body_string(response: axum::http::Response<Body>) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn health_check_returns_ok() {
        let response = get("/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn home_page_returns_ok() {
        let response = get("/").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn article_list_returns_ok() {
        let response = get("/articles").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn article_view_unknown_id_returns_404() {
        let response = get("/articles/999999").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn about_page_returns_ok() {
        let response = get("/about").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let response = get("/nope").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn security_headers_present_on_response() {
        let response = get("/health").await;
        let headers = response.headers();
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert!(headers.get("content-security-policy").is_some());
    }

    // ---- Regression tests for the reflected-input finding ----
    //
    // Burp flagged /articles/<id><payload>. The cause was axum's default
    // Path<u32> rejection echoing the raw segment. These tests go through the
    // real router, so they cover extractors and the security header layer.

    #[tokio::test]
    async fn malformed_article_ids_404_without_reflection() {
        // "qq" is the reflection marker. It must never appear in a response.
        let cases = [
            "/articles/12zz%22%27%3Cqq%3E",
            "/articles/qq",
            "/articles/-1qq",
            "/articles/4294967296qq",
            "/articles/%2B1qq",
            "/articles/007qq",
        ];
        for path in cases {
            let response = get(path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "path: {path}");
            assert_eq!(
                response.headers().get("x-content-type-options").unwrap(),
                "nosniff",
                "missing nosniff on {path}"
            );
            let body = body_string(response).await;
            assert!(!body.contains("qq"), "input reflected for {path}: {body:?}");
        }
    }

    #[tokio::test]
    async fn invalid_utf8_segment_is_rejected_without_reflection() {
        let response = get("/articles/qq%ff").await;
        assert!(
            response.status().is_client_error(),
            "got {}",
            response.status()
        );
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        let body = body_string(response).await;
        assert!(!body.contains("qq"), "input reflected: {body:?}");
    }

    #[tokio::test]
    async fn known_article_serves_html_with_security_headers() {
        let id = test_state().posts[0].id;
        let response = get(&format!("/articles/{id}")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            content_type.starts_with("text/html"),
            "content-type: {content_type}"
        );
        assert!(!body_string(response).await.is_empty());
    }

    // ---- Origin lock, tested through the router ----

    #[tokio::test]
    async fn locked_origin_rejects_direct_requests_with_security_headers() {
        for path in ["/", "/articles", "/about", "/static/anything.css"] {
            let response = send(locked(), path, None).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "path: {path}");
            let headers = response.headers();
            assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
            assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
            assert!(headers.get("content-security-policy").is_some());
        }
    }

    #[tokio::test]
    async fn locked_origin_rejects_wrong_secret() {
        let response = send(locked(), "/", Some("wrong-secret-value")).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn locked_origin_allows_correct_secret_on_real_routes() {
        for path in ["/", "/articles", "/about"] {
            let response = send(locked(), path, Some(TEST_SECRET)).await;
            assert_eq!(response.status(), StatusCode::OK, "path: {path}");
        }
    }

    #[tokio::test]
    async fn locked_origin_leaves_health_open() {
        let response = send(locked(), "/health", None).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
