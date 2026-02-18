use axum::extract::{Query, State, Json, Path};
use crate::models::Post;
use crate::view_models::HomeViewModel;

use axum::response::Html;

pub async fn home(
    State(state): State<AppState>,
) -> Html<String> {
    let mut context = tera::Context::new();

    // Take the 4 most recent (already sorted at startup)
    let recent = state.posts.iter().take(4).collect::<Vec<_>>();
    context.insert("posts", &recent);

    let rendered = state.tera.render("index.html", &context).unwrap();
    Html(rendered)
}

pub async fn article_list() -> Json<Vec<Post>> {
    Json(vec![
        Post {
            id: 1,
            title: "Test".into(),
            content: "Hello".into(),
            created: "2025-03-16T12:00:00Z".into(),
            updated: "".to_string(),
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
        created: "2025-03-16T12:00:00Z".into(),
        updated: "".to_string(),
        topic: "Security".into(),
    })
}
