use serde::Deserialize;

#[derive(Deserialize)]
pub struct Post {
    pub id: u32,
    pub locked: bool,
}

#[derive(Deserialize)]
pub struct PostView {
    pub post: Post,
}

#[derive(Deserialize)]
pub struct GetPostsResponse {
    pub posts: Vec<PostView>,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: u32,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CommentView {
    pub comment: Comment,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentsResponse {
    pub comments: Vec<CommentView>,
}

pub struct CustomWikipediaPage {
    pub title: String,
    pub summary: String,
}
