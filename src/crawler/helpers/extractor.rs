use scraper::Html;

pub fn text(query: &str, html: &Html) -> Vec<String> {
    let all_text = html.root_element().text().collect::<String>();
    if all_text.contains(query) {
        vec![query.to_string()]
    } else {
        vec![]
    }
}
