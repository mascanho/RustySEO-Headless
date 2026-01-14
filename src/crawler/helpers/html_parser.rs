use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct PageData {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub title_len: usize,
    pub h1: String,
    pub h1_len: usize,
    pub h2: String,
    pub h2_len: usize,
    pub description: String,
    pub description_len: usize,
    pub status: String,
    pub mobile: bool,
    pub language: String,
    pub indexability: String,
    pub anchor_links: Vec<(String, String)>,
    pub outlinks: Vec<(String, String)>,
    pub images: Vec<(String, String)>,
    pub headings: Vec<(String, String)>,
    pub headers: Vec<String>,
    pub schema: Vec<String>,
}

pub fn extract_page_elements(document: &Html) -> PageData {
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

    let anchor_links: Vec<(String, String)> = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| {
            let href = e.value().attr("href").unwrap().to_string();
            let text = e.text().collect::<String>();
            (href, text)
        })
        .collect();

    let outlinks: Vec<(String, String)> = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| {
            let href = e.value().attr("href").unwrap().to_string();
            let text = e.text().collect::<String>();
            (href, text)
        })
        .collect();

    let images: Vec<(String, String)> = document
        .select(&Selector::parse("img[src]").unwrap())
        .map(|e| {
            let src = e.value().attr("src").unwrap().to_string();
            let alt = e.value().attr("alt").unwrap_or("").to_string();
            (src, alt)
        })
        .collect();

    let heading_selector = Selector::parse("h1,h2,h3,h4,h5,h6").unwrap();
    let mut headings = Vec::new();
    for element in document.select(&heading_selector) {
        let tag = element.value().name().to_string();
        let text = element.text().collect::<String>();
        headings.push((tag, text));
    }

    let schema_selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let schema: Vec<String> = document
        .select(&schema_selector)
        .map(|e| e.text().collect::<String>())
        .collect();

    PageData {
        id: 0,
        url: "".to_string(),
        title: title.clone(),
        title_len: title.len(),
        h1: h1.clone(),
        h1_len: h1.len(),
        h2: h2.clone(),
        h2_len: h2.len(),
        description: description.clone(),
        description_len: description.len(),
        status: "".to_string(),
        mobile,
        language,
        indexability,
        anchor_links,
        outlinks,
        images,
        headings,
        headers: vec![],
        schema,
    }
}
