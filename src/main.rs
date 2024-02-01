use dotenv::dotenv;
use lemmy_wikibot_rs::apis::lemmy_api::LemmyClient;
use lemmy_wikibot_rs::apis::wikipedia_api::get_wiki_page;
use lemmy_wikibot_rs::*;
use regex::Regex;
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // load the .env file
    dotenv().unwrap();

    // environment variables
    let (username_or_email, password, instance, sentence_reduction_limit) = (
        env::var("LEMMY_USERNAME_OR_EMAIL")
            .expect("LEMMY_USERNAME_OR_EMAIL not configured in .env"),
        env::var("LEMMY_PASSWORD").expect("LEMMY_PASSWORD not configured in .env"),
        env::var("LEMMY_INSTANCE").expect("LEMMY_INSTANCE not configured in .env"),
        // env::var("LEMMY_COMMUNITY").expect("LEMMY_COMMUNITY not configured in .env"), this is no longer used
        env::var("SENTENCE_REDUCTION_LIMIT")
            .expect("SENTENCE_REDUCTION_LIMIT not configured in .env")
            .parse::<u8>()
            .expect("SENTENCE_REDUCTION_LIMIT is not a number"),
    );

    // login to lemmy client
    let mut client = LemmyClient::new(username_or_email, password, instance);
    client.login();

    let mut unmarked_mentions: Vec<u32> = Vec::new();

    // main part of the program
    loop {
        let bot_mentions_resp = match client.get_mentions() {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("An error occurred: {}", err);
                sleep(Duration::new(3, 0));
                continue;
            }
        };

        for mention in bot_mentions_resp.mentions {
            sleep(Duration::new(2, 0)); // to avoid rate limits

            let comment = mention.comment;
            let post = mention.post;

            // check if the current comment is one of the unmarked ones, if yes, try to mark it
            if unmarked_mentions.contains(&comment.id)
                && client
                    .mark_mention_as_read(mention.person_mention.id)
                    .is_ok()
            {
                unmarked_mentions.retain(|x| x != &comment.id);
            }

            // if comment content has a []() syntax, extract the link from it, and match it against title_re, otherwise try to match the whole comment content
            let link_md_re = Regex::new(r"\[.*\]\(.+wiki\/(.+)\)").unwrap();
            let extracted_link = link_md_re
                .captures(&comment.content)
                .map(|caps| caps.get(1).unwrap().as_str());


            // extract title or section from comment
            let title_re = Regex::new(r"wikipedia.org/wiki/([^.#\s]+)").unwrap();
            let section_re = Regex::new(r"(#\S+)").unwrap();
            let haystack = if let Some(extracted) = extracted_link {
                extracted
            } else {
                &comment.content
            };
            let title = match title_re.captures(haystack) {
                Some(caps) => caps.get(1).unwrap().as_str(),
                None => {
                    client.create_comment(post.id, comment.id, "Sorry, I could not get the wikipedia summary for the wikipedia link mentioned in your comment.".to_owned()).ok().unwrap(); // errors here get ignored, might add an error handler here as well
                    continue;
                },
            };
            let extracted_section = section_re
                .captures(haystack)
                .map(|caps| caps.get(1).unwrap().as_str().to_owned());

            // get a new CustomWikipediaPage and reduce the summary sentences
            let mut wiki_page = match get_wiki_page(title.to_string(), extracted_section) {
                Some(wiki_page) => wiki_page,
                None => {
                    client.create_comment(post.id, comment.id, "Sorry, I could not get the wikipedia summary for the wikipedia link mentioned in your comment.".to_owned()).ok().unwrap(); // errors here get ignored, might add an error handler here as well
                    continue;
                },
            };
            wiki_page.reduce_sentences(sentence_reduction_limit);

            // build comment, reply to mention sender and mark mention as read
            let built_comment = comment_builder(wiki_page);
            if let Err(err) = client.create_comment(post.id, comment.id, built_comment) {
                eprintln!("An error occurred: {}", err);
                sleep(Duration::new(3, 0));
                continue;
            } else {
                println!("Answered user: {}", mention.creator.id);
                if let Err(err) = client.mark_mention_as_read(mention.person_mention.id) {
                    eprintln!("An error occurred: {}", err);
                    unmarked_mentions.push(comment.id);
                    sleep(Duration::new(3, 0));
                }
            }
        }
        sleep(Duration::new(60*2, 0)) // sleep for 2 minutes
    }
}
