use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct AppState {
    pub posts: Vec<Post>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "Created")]
    pub created: String,
    #[serde(rename = "Updated")]
    pub updated: String,
    #[serde(rename = "Topic")]
    pub topic: String,
}
