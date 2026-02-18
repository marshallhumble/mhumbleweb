mod models;
mod handlers;
mod view_models;

use axum::{
    routing::get,
    Router,
};

use handlers::{home, article_list, article_view};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(|| async { "OK" }))
        .route("/articles", get(article_list))
        .route("/articles/{article_id}", get(article_view))
        .route("/about", get(|| async { "About me!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
