use axum::extract::{Query, State, Json, Path};
use crate::models::Post;

use axum::response::Html;

pub async fn home() -> Html<&'static str> {
    Html("<h1>Home</h1>")
}

pub async fn article_list() -> Json<Vec<Post>> {
    Json(vec![
        Post {
            id: 1,
            title: "Test".into(),
            content: "Hello".into(),
            published: true,
            created: "2025-03-16T12:00:00Z".into(),
            topic: "Security".into(),
        },
    ])
}


pub async fn article_view(
    Path(article_id): Path<i32>,
) -> Json<Post> {
    Json(Post {
        id: article_id,
        title: "Example".into(),
        content: "Article body".into(),
        published: true,
        created: "2025-03-16T12:00:00Z".into(),
        topic: "Security".into(),
    })
}
