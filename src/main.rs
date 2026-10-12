use axum::{Router, routing::get};
use std::net::SocketAddr;
use tera::Tera;
use tokio::net::TcpListener;

mod handlers;
mod middleware;
mod models;

use handlers::{about, article_list, article_view, home};
use middleware::apply_security_headers;
use models::load_posts;

#[derive(Clone)]
pub struct AppState {
    pub posts: Vec<models::Post>,
    pub tera: Tera,
}

fn app(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(home))
        .route("/health", get(|| async { "OK" }))
        .route("/articles", get(article_list))
        .route("/articles/{article_id}", get(article_view))
        .route("/about", get(about))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state);

    apply_security_headers(router)
}

#[tokio::main]
async fn main() {
    let mut tera = Tera::new();
    tera.load_from_glob("templates/**/*")
        .expect("Failed to load templates");

    let posts = load_posts();

    let state = AppState { posts, tera };

    let app = app(state);

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

    fn test_state() -> AppState {
        let mut tera = Tera::new();
        tera.load_from_glob("templates/**/*")
            .expect("Failed to load templates");
        AppState {
            posts: load_posts(),
            tera,
        }
    }

    async fn get(uri: &str) -> axum::http::Response<Body> {
        app(test_state())
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
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
}
