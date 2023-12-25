use crate::structs::CustomWikipediaPage;
use wikipedia::http::default::Client;
use wikipedia::Wikipedia;

pub fn get_wiki_page(page_title: String) -> Option<CustomWikipediaPage> {
    let wiki = Wikipedia::<Client>::default();
    let page = wiki.page_from_title(page_title);

    let title = page.get_title().ok().or(None);
    let summary = page.get_summary().ok().or(None);

    // check if title or summary is None, if yes, return it else return Some(CustomWikipediaPage)
    let customwikipage: Option<CustomWikipediaPage> =
        title.and_then(|title| summary.map(|summary| CustomWikipediaPage { title, summary }));

    customwikipage
}
