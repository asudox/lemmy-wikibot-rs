use std::fs;
use std::path::Path;

pub mod apis;
pub mod structs;

pub fn comment_builder(title: &str, summary: &str) -> String {
    format!(
"Here's the summary for the wikipedia article you mentioned in your comment:

> {}

[^article^](https://en.wikipedia.org/wiki/{}) ^|^ [about](https://lemmy.world/u/wikibot)
", summary, title)
}

pub fn save_to_db(comment_id: u32) {
    let vec_path = Path::new("checked_comments.json");
    if vec_path.exists() {
        let file_content = fs::read_to_string(vec_path).unwrap();
        let mut deserialized_vec = serde_json::from_str::<Vec<u32>>(&file_content).unwrap();
        deserialized_vec.push(comment_id);
        fs::write(vec_path, serde_json::to_string(&deserialized_vec).unwrap()).unwrap();
    } else {
        panic!("checked_comments.json does not exist!")
    }
}

pub fn load_db() -> Vec<u32> {
    let vec_path = Path::new("checked_comments.json");
    if !vec_path.exists() {
        println!("checked_comments.vec file does not exist, creating it...");
        fs::write(vec_path, serde_json::to_vec(&vec![0_u32]).unwrap()).unwrap();
        Vec::new().push(0)
    }
    let file_content = fs::read_to_string(vec_path).unwrap();

    serde_json::from_str::<Vec<u32>>(&file_content).unwrap()
}
