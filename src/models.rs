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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_posts_is_sorted_newest_first() {
        let posts = load_posts();
        assert!(!posts.is_empty());
        for pair in posts.windows(2) {
            assert!(
                pair[0].created >= pair[1].created,
                "posts are not sorted newest first: {} came before {}",
                pair[0].created,
                pair[1].created
            );
        }
    }

    #[test]
    fn load_posts_have_matching_template_files() {
        for post in load_posts() {
            let path = format!("templates/articles/{}.html", post.filename);
            assert!(
                fs::metadata(&path).is_ok(),
                "missing template file for post {}: {}",
                post.id,
                path
            );
        }
    }
}
