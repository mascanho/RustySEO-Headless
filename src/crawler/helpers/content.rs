use scraper::Html;

pub fn classify_content(html: &Html) -> f32 {
    let text = html.root_element().text().collect::<String>();

    if text.len() < 10 {
        0.0
    } else {
        1.0
    }
}
