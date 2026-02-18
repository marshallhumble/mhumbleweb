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

#[tokio::main]
async fn main() {
    let tera = Tera::new("templates/**/*").expect("Failed to load templates");

    let posts = load_posts();

    let state = AppState { posts, tera };

    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(|| async { "OK" }))
        .route("/articles", get(article_list))
        .route("/articles/{article_id}", get(article_view))
        .route("/about", get(about))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state);

    let app = apply_security_headers(app);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
