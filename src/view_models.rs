pub struct HomeViewModel {
    pub recent_posts: Vec<PostSummary>,
}

pub struct PostSummary {
    pub id: i32,
    pub title: String,
    pub created: String,
    pub topic: String, // split on comma before here
}