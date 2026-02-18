use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: u32,
    pub title: String,
    pub created: String,
    pub updated: String,
    pub topic: String,
    pub filename: String,
}

pub fn load_posts() -> Vec<Post> {
    let data =
        fs::read_to_string("internal/models/json/data.json").expect("Failed to read data.json");
    let mut posts: Vec<Post> = serde_json::from_str(&data).expect("Failed to parse data.json");
    posts.sort_by(|a, b| b.created.cmp(&a.created));
    posts
}
