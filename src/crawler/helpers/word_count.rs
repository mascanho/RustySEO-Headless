use scraper::Html;

pub fn get_words(html: &Html) -> usize {
    let text = html
        .root_element()
        .text()
        .collect::<String>()
        .replace("\n", " ")
        .replace("\t", " ");

    text.split_whitespace().count()
}
