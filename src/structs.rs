use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    // pub name: String,
    pub id: u32,
    pub bot_account: bool,
}

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
pub struct PrivateMessageView {
    pub private_message: Comment, // this is fine because a pm is basically a comment under the hood
    pub creator: User,
}

#[derive(Deserialize)]
pub struct GetPostsResponse {
    pub posts: Vec<PostView>,
}

#[derive(Deserialize)]
pub struct Comment {
    pub id: u32,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CommentView {
    pub comment: Comment,
    pub creator: User,
}

#[derive(Deserialize)]
pub struct GetCommentsResponse {
    pub comments: Vec<CommentView>,
}

#[derive(Deserialize)]
pub struct GetPrivateMessagesResponse {
    pub private_messages: Vec<PrivateMessageView>,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub jwt: String,
}

pub struct CustomWikipediaPage {
    pub page_title: String,
    pub content: String,
    pub is_section: bool,
}
