use crate::models::Post;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HomeViewModel {
    pub posts: Vec<Post>,
}

#[derive(Debug, Serialize)]
pub struct ArticleListViewModel {
    pub posts: Vec<Post>,
}

#[derive(Debug, Serialize)]
pub struct ArticleViewModel {
    pub post: Post,
    pub content: String,
    pub total: usize,
}
