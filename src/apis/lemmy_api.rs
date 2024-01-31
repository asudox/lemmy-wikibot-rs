use std::collections::HashMap;

use crate::structs::{GetPrivateMessagesResponse, GetUserMentionsResponse, LoginResponse};

pub struct LemmyClient {
    username_or_email: String,
    password: String,
    instance: String,
    // community: String,  this is no longer used
    jwt: Option<String>,
}

impl LemmyClient {
    pub fn new(
        username_or_email: String,
        password: String,
        instance: String,
        // community: String,
    ) -> LemmyClient {
        LemmyClient {
            username_or_email,
            password,
            instance,
            // community,
            jwt: None,
        }
    }

    pub fn login(&mut self) {
        let payload = HashMap::from([
            ("username_or_email", &self.username_or_email),
            ("password", &self.password),
        ]);
        let resp: LoginResponse = reqwest::blocking::Client::new()
            .post(format!("https://{}/api/v3/user/login", self.instance))
            .json(&payload)
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .unwrap();

        self.jwt = Some(resp.jwt);
    }

    // this is no longer used

    // pub fn get_posts(
    //     &self,
    //     sort_type: &str,
    //     limit: &str,
    // ) -> Result<GetPostsResponse, reqwest::Error> {
    //     let params = [
    //         ("community_name", self.community.as_str()),
    //         ("sort", sort_type),
    //         ("limit", limit),
    //     ];
    //     let resp: GetPostsResponse = reqwest::blocking::Client::new()
    //         .get(format!("https://{}/api/v3/post/list", self.instance))
    //         .query(&params)
    //         .send()?
    //         .error_for_status()?
    //         .json()?;

    //     Ok(resp)
    // }

    // this is no longer used

    // pub fn get_comments(
    //     &self,
    //     post_id: &str,
    //     sort_type: &str,
    // ) -> Result<GetCommentsResponse, reqwest::Error> {
    //     let params = [("post_id", post_id), ("sort", sort_type)];
    //     let resp: GetCommentsResponse = reqwest::blocking::Client::new()
    //         .get(format!("https://{}/api/v3/comment/list", self.instance))
    //         .query(&params)
    //         .send()?
    //         .error_for_status()?
    //         .json()?;

    //     Ok(resp)
    // }

    pub fn create_comment(
        &self,
        post_id: u32,
        parent_id: u32,
        content: String,
    ) -> Result<(), reqwest::Error> {
        reqwest::blocking::Client::new()
            .post(format!("https://{}/api/v3/comment", self.instance))
            .json(&serde_json::json!({
                "content": content,
                "post_id": post_id,
                "parent_id": parent_id,
                "auth": self.jwt.as_ref().unwrap()
            }))
            // .bearer_auth(self.jwt.as_ref().unwrap()) // can be used when lemmy.world upgrades to 0.19v, remove auth from json body and uncomment this
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_pms(&self) -> Result<GetPrivateMessagesResponse, reqwest::Error> {
        let params = [("auth", self.jwt.as_ref().unwrap())];
        let resp: GetPrivateMessagesResponse = reqwest::blocking::Client::new()
            .get(format!(
                "https://{}/api/v3/private_message/list",
                self.instance
            ))
            .query(&params)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(resp)
    }

    pub fn create_pm(&self, recipient_id: u32, content: &str) -> Result<(), reqwest::Error> {
        reqwest::blocking::Client::new()
            .post(format!("https://{}/api/v3/private_message", self.instance))
            .json(&serde_json::json!({
                "content": content,
                "recipient_id": recipient_id,
                "auth": self.jwt.as_ref().unwrap()
            }))
            // .bearer_auth(self.jwt.as_ref().unwrap()) // can be used when lemmy.world upgrades to 0.19v, remove auth from json body and uncomment this
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_mentions(&self) -> Result<GetUserMentionsResponse, reqwest::Error> {
        //TODO change once lemmy.world upgrades to Lemmy v19
        let params = [
            ("auth", self.jwt.as_ref().unwrap().as_str()),
            ("unread_only", "true"),
        ];
        let resp: GetUserMentionsResponse = reqwest::blocking::Client::new()
            .get(format!("https://{}/api/v3/user/mention", self.instance))
            .query(&params)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(resp)
    }

    pub fn mark_mention_as_read(&self, person_mention_id: u32) -> Result<(), reqwest::Error> {
        reqwest::blocking::Client::new()
            .post(format!(
                "https://{}/api/v3/user/mention/mark_as_read",
                self.instance
            ))
            .json(&serde_json::json!({
                "person_mention_id": person_mention_id,
                "read": true,
                "auth": self.jwt.as_ref().unwrap()
            }))
            // .bearer_auth(self.jwt.as_ref().unwrap()) // can be used when lemmy.world upgrades to 0.19v, remove auth from json body and uncomment this
            .send()?
            .error_for_status()?;

        Ok(())
    }
}
