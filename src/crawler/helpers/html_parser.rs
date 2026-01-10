use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct PageElements {
    pub title: String,
    pub h1: String,
    pub description: String,
    pub h2: String,
    pub mobile: bool,
    pub language: String,
    pub indexability: String,
    pub anchor_links: Vec<String>,
}

pub fn extract_page_elements(document: &Html) -> PageElements {
    let title_selector = Selector::parse("title").unwrap();
    let h1_selector = Selector::parse("h1").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();
    let desc_selector = Selector::parse("meta[name='description']").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let h1 = document
        .select(&h1_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let description = document
        .select(&desc_selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let h2 = document
        .select(&h2_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let mobile = document
        .select(&Selector::parse("meta[name='viewport']").unwrap())
        .next()
        .is_some();

    let language = document
        .select(&Selector::parse("html[lang]").unwrap())
        .next()
        .and_then(|e| e.value().attr("lang"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let indexability = document
        .select(&Selector::parse("meta[name='robots']").unwrap())
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let anchor_links = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| e.value().attr("href").unwrap().to_string())
        .collect();

    PageElements {
        title,
        h1,
        description,
        h2,
        mobile,
        language,
        indexability,
        anchor_links,
    }
}
