use dotenv::dotenv;
use lemmy_wikibot_rs::apis::lemmy_api::LemmyClient;
use lemmy_wikibot_rs::apis::wikipedia_api::get_wiki_page;
use lemmy_wikibot_rs::comment_builder;
use lemmy_wikibot_rs::{load_db, save_to_db};
use regex::Regex;
use reqwest::StatusCode;
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    dotenv().unwrap();

    let (username_or_email, password, instance, community) = (
        env::var("LEMMY_USERNAME_OR_EMAIL")
            .expect("LEMMY_USERNAME_OR_EMAIL not configured in .env"),
        env::var("LEMMY_PASSWORD").expect("LEMMY_PASSWORD not configured in .env"),
        env::var("LEMMY_INSTANCE").expect("LEMMY_INSTANCE not configured in .env"),
        env::var("LEMMY_COMMUNITY").expect("LEMMY_COMMUNITY not configured in .env"),
    );

    // login to lemmy client
    let mut client = LemmyClient::new(username_or_email, password, instance, community);
    client.login();

    loop {
        println!("Getting posts");
        let post_list_resp = match client.get_posts("NewComments", "10") {
            Ok(resp) => resp,
            Err(err) => {
                if err.is_status() {
                    match err.status().unwrap() {
                        StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_GATEWAY => {
                            sleep(Duration::new(5, 0));
                            continue;
                        }
                        StatusCode::INTERNAL_SERVER_ERROR => panic!("INTERNAL_SERVER_ERROR"),
                        _ => panic!("Unexpected status code: {}", err),
                    }
                } else if err.is_timeout() {
                    sleep(Duration::new(5, 0));
                    continue;
                } else {
                    panic!("Unexpected error occurred: {}", err);
                }
            }
        };
        sleep(Duration::new(3, 0));
        for post_view in post_list_resp.posts {
            let post = post_view.post;
            if post.locked {
                continue;
            } else {
                println!("Getting comments");
                let comment_list_resp =
                    match client.get_comments(post.id.to_string().as_str(), "New") {
                        Ok(resp) => resp,
                        Err(err) => {
                            if err.is_status() {
                                match err.status().unwrap() {
                                    StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_GATEWAY => {
                                        sleep(Duration::new(5, 0));
                                        continue;
                                    }
                                    StatusCode::INTERNAL_SERVER_ERROR => {
                                        panic!("INTERNAL_SERVER_ERROR")
                                    }
                                    _ => panic!("Unexpected status code: {}", err),
                                }
                            } else if err.is_timeout() {
                                sleep(Duration::new(5, 0));
                                continue;
                            } else {
                                panic!("Unexpected error occurred: {}", err);
                            }
                        }
                    };
                sleep(Duration::new(2, 0));
                for comment_view in comment_list_resp.comments {
                    let checked_comments: Vec<u32> = load_db();
                    let comment = comment_view.comment;
                    if checked_comments.contains(&comment.id) || comment.creator.bot_account {
                        continue;
                    } else {
                        save_to_db(Some(comment.id), None);

                        // if comment content has a []() syntax, extract the link from it, and match it against title_re, otherwise try to match the whole comment content
                        let link_md_re = Regex::new(r"\[.+\]\((.+)\)").unwrap();
                        let extracted_link = link_md_re
                            .captures(&comment.content)
                            .map(|caps| caps.get(1).unwrap().as_str());
                        let title_re = Regex::new(r"wikipedia.org/wiki/([^#\s]+)").unwrap();
                        let section_re = Regex::new(r"(#\S+)").unwrap();
                        let haystack = if let Some(extracted) = extracted_link {
                            extracted
                        } else {
                            &comment.content
                        };
                        let title = match title_re.captures(haystack) {
                            Some(caps) => caps.get(1).unwrap().as_str(),
                            None => continue,
                        };
                        let extracted_section = section_re
                            .captures(haystack)
                            .map(|caps| caps.get(1).unwrap().as_str().to_string());

                        let wiki_page = match get_wiki_page(title.to_string(), extracted_section) {
                            Some(wiki_page) => wiki_page,
                            None => continue,
                        };

                        let built_comment = comment_builder(wiki_page);
                        match client.create_comment(post.id, comment.id, built_comment.as_str()) {
                            Ok(_) => println!("Answered comment: {}", comment.id),
                            Err(err) => {
                                if err.is_status() {
                                    match err.status().unwrap() {
                                        StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_GATEWAY => {
                                            let mut new_vec = checked_comments.clone();
                                            new_vec.pop();
                                            save_to_db(None, Some(new_vec));
                                            sleep(Duration::new(5, 0));
                                            continue;
                                        }
                                        StatusCode::INTERNAL_SERVER_ERROR => {
                                            panic!("INTERNAL_SERVER_ERROR")
                                        }
                                        _ => panic!("Unexpected status code: {}", err),
                                    }
                                } else if err.is_timeout() {
                                    sleep(Duration::new(5, 0));
                                    continue;
                                } else {
                                    panic!("Unexpected error occurred: {}", err);
                                }
                            }
                        }
                        sleep(Duration::new(1, 0));
                    }
                }
            }
        }
        sleep(Duration::new(10, 0));
    }
}
