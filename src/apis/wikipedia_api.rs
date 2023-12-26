use crate::structs::CustomWikipediaPage;
use wikipedia::http::default::Client;
use wikipedia::Wikipedia;

pub fn get_wiki_page(
    page_title: String,
    section_title: Option<String>,
) -> Option<CustomWikipediaPage> {
    let wiki = Wikipedia::<Client>::default();
    let page = wiki.page_from_title(page_title.clone());

    if section_title.is_some() {
        // add the section name to the page_title
        let mut new_page_title = page_title;
        new_page_title.push_str(&section_title.clone().unwrap());

        // isn't actually a summary, just the whole content of the section
        let content = page.get_section_content(&section_title.unwrap().replace('#', ""));

        let customwikipage: Option<CustomWikipediaPage> = match content {
            Ok(section_content) => section_content.map(|content| CustomWikipediaPage {
                page_title: new_page_title,
                content,
                is_section: true,
            }),
            Err(_) => None,
        };

        customwikipage
    } else {
        let page_title = page.get_title().ok().or(None);

        let summary = page.get_summary().ok().or(None);

        // check if title or summary is None, if yes, return it else return Some(CustomWikipediaPage)
        page_title.and_then(|page_title| {
            summary.map(|summary| CustomWikipediaPage {
                page_title,
                content: summary,
                is_section: false,
            })
        })
    }
}
