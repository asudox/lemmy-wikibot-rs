use dotenv::dotenv;
use lemmy_wikibot_rs::apis::lemmy_api::LemmyClient;
use lemmy_wikibot_rs::apis::wikipedia_api::get_wiki_page;
use lemmy_wikibot_rs::*;
use regex::Regex;
use reqwest::StatusCode;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    dotenv().unwrap();

    let (username_or_email, password, instance, community, sentence_reduction_limit) = (
        env::var("LEMMY_USERNAME_OR_EMAIL")
            .expect("LEMMY_USERNAME_OR_EMAIL not configured in .env"),
        env::var("LEMMY_PASSWORD").expect("LEMMY_PASSWORD not configured in .env"),
        env::var("LEMMY_INSTANCE").expect("LEMMY_INSTANCE not configured in .env"),
        env::var("LEMMY_COMMUNITY").expect("LEMMY_COMMUNITY not configured in .env"),
        env::var("SENTENCE_REDUCTION_LIMIT")
            .expect("SENTENCE_REDUCTION_LIMIT not configured in .env"),
    );
    let sentence_reduction_limit: u16 = sentence_reduction_limit
        .parse()
        .expect("SENTENCE_REDUCTION_LIMIT is not a number within the range of u16");

    // login to lemmy client
    let mut client = LemmyClient::new(username_or_email, password, instance, community);
    client.login();

    let check_inbox = Arc::new(AtomicBool::new(false));
    let check_inbox_clone = Arc::clone(&check_inbox);

    std::thread::spawn({
        move || {
            loop {
                std::thread::sleep(Duration::new(10 * 60, 0)); // every 10 mins
                check_inbox_clone.store(true, Ordering::SeqCst);
            }
        }
    });

    loop {
        println!("Getting posts");
        let post_list_resp = match client.get_posts("NewComments", "10") {
            Ok(resp) => resp,
            Err(err) => {
                if err.is_status() {
                    match err.status().unwrap() {
                        StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::REQUEST_TIMEOUT => {
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
                                    StatusCode::TOO_MANY_REQUESTS
                                    | StatusCode::BAD_GATEWAY
                                    | StatusCode::REQUEST_TIMEOUT => {
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
                    let checked_comments: Vec<u32> = load_cc_db();
                    let excluded_creators: Vec<u32> = load_ec_db();
                    let comment = comment_view.comment;
                    if checked_comments.contains(&comment.id)
                        || excluded_creators.contains(&comment_view.creator.id)
                        || comment_view.creator.bot_account
                    {
                        continue;
                    } else {
                        save_to_cc_db(Some(comment.id), None);

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

                        let mut wiki_page =
                            match get_wiki_page(title.to_string(), extracted_section) {
                                Some(wiki_page) => wiki_page,
                                None => continue,
                            };
                        wiki_page.reduce_sentences(sentence_reduction_limit);

                        let built_comment = comment_builder(wiki_page);
                        match client.create_comment(post.id, comment.id, built_comment.as_str()) {
                            Ok(_) => println!("Answered comment: {}", comment.id),
                            Err(err) => {
                                if err.is_status() {
                                    match err.status().unwrap() {
                                        StatusCode::TOO_MANY_REQUESTS
                                        | StatusCode::BAD_GATEWAY
                                        | StatusCode::REQUEST_TIMEOUT => {
                                            let mut new_vec = checked_comments.clone();
                                            new_vec.pop();
                                            save_to_cc_db(None, Some(new_vec));
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
        sleep(Duration::new(5, 0));
        if check_inbox.load(Ordering::SeqCst) {
            if let Ok(pms) = client.get_pms() {
                for private_message_view in pms.private_messages {
                    let exluded_creators = load_ec_db();
                    let private_message = private_message_view.private_message;
                    if !exluded_creators.contains(&private_message_view.creator.id)
                        && private_message.content.trim().to_lowercase() == "optout"
                    {
                        println!("Excluded user: {}", private_message_view.creator.id);
                        save_to_ec_db(Some(private_message_view.creator.id), None);
                        if client.create_pm(private_message_view.creator.id, "You have been successfully opted out. The bot will no longer respond to your comments containing wikipedia links.").is_ok(){ // wait 1 second if the pm was sent successfully to avoid rate limit
                            sleep(Duration::new(1, 0))
                        }; // any errors will be ignored
                    }
                }
                check_inbox.store(false, Ordering::SeqCst);
            }
        }
        sleep(Duration::new(5, 0));
    }
}
