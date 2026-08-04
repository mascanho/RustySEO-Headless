//! "Export Data" (Actions Menu) - writes one sheet/file per application tab,
//! each mirroring that tab's exact columns and content. Uses a real multi-sheet
//! Excel workbook (.xlsx) so tabs can be clicked between like in the app, except
//! when a sheet would be large enough to make Excel/Sheets sluggish to open, in
//! which case we fall back to one CSV per tab bundled into a .zip.

use crate::models::App;
use rust_xlsxwriter::{Format, Workbook};
use std::collections::HashMap;
use std::io::Write;

/// Sheets bigger than this (in data rows, not counting the header) make Excel /
/// Google Sheets noticeably slow to open, so we switch to zipped CSVs instead.
const MAX_XLSX_ROWS: usize = 200_000;

/// Excel sheet names: no `: \ / ? * [ ]`, and capped at 31 characters.
const TAB_NAMES: [&str; 11] = [
    "Overview",
    "External",
    "Internal",
    "Redirects",
    "Images",
    "CSS",
    "Javascript",
    "CWV",
    "Content",
    "Files",
    "Custom Extractor",
];

struct ExportSheet {
    name: &'static str,
    header: Vec<&'static str>,
    rows: Vec<Vec<String>>,
}

pub struct ExportSummary {
    pub path: String,
    pub format: &'static str,
    pub sheet_count: usize,
    pub total_rows: usize,
}

/// Build one export sheet per application tab and write them out, choosing the
/// format based on the largest sheet's row count.
pub fn export_all_tabs(app: &App) -> Result<ExportSummary, String> {
    let sheets = build_sheets(app);

    let total_rows: usize = sheets.iter().map(|s| s.rows.len()).sum();
    let max_rows = sheets.iter().map(|s| s.rows.len()).max().unwrap_or(0);
    if total_rows == 0 {
        return Err("No data to export yet".to_string());
    }

    let dir = exports_dir()?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

    if max_rows > MAX_XLSX_ROWS {
        let path = dir.join(format!("rustyseo_export_{}.zip", timestamp));
        write_csv_zip(&sheets, &path)?;
        Ok(ExportSummary {
            path: path.display().to_string(),
            format: "zip of CSVs (largest sheet exceeded 200k rows)",
            sheet_count: sheets.len(),
            total_rows,
        })
    } else {
        let path = dir.join(format!("rustyseo_export_{}.xlsx", timestamp));
        write_xlsx(&sheets, &path).map_err(|e| e.to_string())?;
        Ok(ExportSummary {
            path: path.display().to_string(),
            format: "xlsx",
            sheet_count: sheets.len(),
            total_rows,
        })
    }
}

fn exports_dir() -> Result<std::path::PathBuf, String> {
    let dirs = directories::ProjectDirs::from("", "", "rustyseo")
        .ok_or("Could not determine project directories")?;
    let dir = dirs.data_dir().join("exports");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn write_xlsx(sheets: &[ExportSheet], path: &std::path::Path) -> Result<(), rust_xlsxwriter::XlsxError> {
    let mut workbook = Workbook::new();
    let header_format = Format::new().set_bold();

    for sheet in sheets {
        let worksheet = workbook.add_worksheet_with_constant_memory();
        worksheet.set_name(sheet.name)?;
        worksheet.write_row_with_format(0, 0, sheet.header.clone(), &header_format)?;
        for (i, row) in sheet.rows.iter().enumerate() {
            worksheet.write_row(i as u32 + 1, 0, row)?;
        }
    }

    workbook.save(path)?;
    Ok(())
}

fn write_csv_zip(sheets: &[ExportSheet], path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    for sheet in sheets {
        let filename = format!("{}.csv", sheet.name.to_lowercase().replace(' ', "_"));
        zip.start_file(filename, options).map_err(|e| e.to_string())?;

        let mut csv = String::new();
        csv.push_str(&sheet.header.iter().map(|h| csv_escape(h)).collect::<Vec<_>>().join(","));
        csv.push('\n');
        for row in &sheet.rows {
            csv.push_str(&row.iter().map(|f| csv_escape(f)).collect::<Vec<_>>().join(","));
            csv.push('\n');
        }
        zip.write_all(csv.as_bytes()).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn build_sheets(app: &App) -> Vec<ExportSheet> {
    vec![
        build_overview_sheet(app),
        build_link_sheet(TAB_NAMES[1], &app.external_full_filtered_table_data, &app.url_to_status),
        build_link_sheet(TAB_NAMES[2], &app.internal_full_filtered_table_data, &app.url_to_status),
        build_redirects_sheet(app),
        build_images_sheet(app),
        build_css_sheet(app),
        build_js_sheet(app),
        build_cwv_sheet(app),
        build_content_sheet(app),
        build_files_sheet(app),
        build_extractor_sheet(app),
    ]
}

/// Mirrors the status-code simplification shown in the Overview table
/// (src/ui/tabs/dashboard.rs).
fn format_status(raw: &str) -> String {
    if raw.contains("200") {
        "200".to_string()
    } else if raw.contains("404") {
        "404".to_string()
    } else if raw.contains("301") || raw.contains("302") {
        raw.to_string()
    } else if raw.contains("500") {
        "500".to_string()
    } else if raw.contains("403") {
        "403".to_string()
    } else if raw.contains("503") {
        format!("🚧 {}", raw)
    } else {
        raw.to_string()
    }
}

fn col(row: &[String], idx: usize) -> String {
    row.get(idx).cloned().unwrap_or_default()
}

fn build_overview_sheet(app: &App) -> ExportSheet {
    let header = vec![
        "ID",
        "URL",
        "Title",
        "Title Length",
        "Description",
        "Description Length",
        "H1",
        "H1 Length",
        "H2",
        "H2 Length",
        "Status",
        "Size",
        "Language",
        "Indexability",
        "Link Score",
    ];

    let rows = app
        .full_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let url = col(data, 1);
            let size_kb = col(data, 17).trim().parse::<usize>().unwrap_or(0) / 1024;
            let indexability = if col(data, 13).contains("noindex") {
                "Non-indexable"
            } else {
                "Indexable"
            };
            let link_score = app
                .link_scores
                .get(&url)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string());

            vec![
                (i + 1).to_string(),
                url,
                col(data, 2),
                col(data, 3),
                col(data, 6),
                col(data, 7),
                col(data, 4),
                col(data, 5),
                col(data, 8),
                col(data, 9),
                format_status(&col(data, 10)),
                format!("{} KB", size_kb),
                col(data, 12),
                indexability.to_string(),
                link_score,
            ]
        })
        .collect();

    ExportSheet { name: TAB_NAMES[0], header, rows }
}

fn lookup_status(url: &str, url_to_status: &HashMap<String, String>) -> String {
    url_to_status.get(url).cloned().unwrap_or_else(|| "Pending".to_string())
}

fn build_link_sheet(
    name: &'static str,
    data: &[impl LinkEntry],
    url_to_status: &HashMap<String, String>,
) -> ExportSheet {
    let header = vec!["#", "Source URL", "Destination URL", "Anchor Text", "Rel", "Status"];
    let rows = data
        .iter()
        .map(|link| {
            vec![
                link.id().to_string(),
                link.source().to_string(),
                link.destination().to_string(),
                link.anchor().to_string(),
                link.rel().to_string(),
                lookup_status(link.destination(), url_to_status),
            ]
        })
        .collect();
    ExportSheet { name, header, rows }
}

trait LinkEntry {
    fn id(&self) -> usize;
    fn source(&self) -> &str;
    fn destination(&self) -> &str;
    fn anchor(&self) -> &str;
    fn rel(&self) -> &str;
}

impl LinkEntry for crate::models::InternalLink {
    fn id(&self) -> usize { self.id }
    fn source(&self) -> &str { &self.source }
    fn destination(&self) -> &str { &self.destination }
    fn anchor(&self) -> &str { &self.anchor }
    fn rel(&self) -> &str { &self.rel }
}

impl LinkEntry for crate::models::ExternalLink {
    fn id(&self) -> usize { self.id }
    fn source(&self) -> &str { &self.source }
    fn destination(&self) -> &str { &self.destination }
    fn anchor(&self) -> &str { &self.anchor }
    fn rel(&self) -> &str { &self.rel }
}

fn build_redirects_sheet(app: &App) -> ExportSheet {
    let header = vec!["ID", "Initial URL", "Status", "Redirect Chain"];
    let rows = app
        .redirects_full_filtered_table_data
        .iter()
        .map(|entry| {
            let chain = entry
                .chain
                .iter()
                .map(|h| format!("{} ({})", h.url, h.status))
                .collect::<Vec<_>>()
                .join(" -> ");
            vec![
                entry.id.to_string(),
                entry.initial_url.clone(),
                entry.status_code.to_string(),
                chain,
            ]
        })
        .collect();
    ExportSheet { name: TAB_NAMES[3], header, rows }
}

fn build_images_sheet(app: &App) -> ExportSheet {
    let header = vec!["ID", "Image URL", "Alt Text", "Status", "Size", "Pages"];
    let rows = app
        .images_full_filtered_table_data
        .iter()
        .map(|img| {
            vec![
                img.id.to_string(),
                img.url.clone(),
                img.alt.clone(),
                img.status.clone(),
                img.size.clone(),
                img.page_count.to_string(),
            ]
        })
        .collect();
    ExportSheet { name: TAB_NAMES[4], header, rows }
}

fn build_css_sheet(app: &App) -> ExportSheet {
    let header = vec!["#", "CSS URL", "Pages Using"];
    let rows = app
        .css_urls_full_filtered_table_data
        .iter()
        .map(|css| vec![css.id.to_string(), css.url.clone(), css.page_count.to_string()])
        .collect();
    ExportSheet { name: TAB_NAMES[5], header, rows }
}

fn build_js_sheet(app: &App) -> ExportSheet {
    let header = vec!["#", "JS URL", "Type", "Async", "Defer", "Pages Using"];
    let rows = app
        .js_urls_full_filtered_table_data
        .iter()
        .map(|js| {
            vec![
                js.id.to_string(),
                js.url.clone(),
                js.script_type.clone(),
                if js.is_async { "Yes" } else { "No" }.to_string(),
                if js.is_defer { "Yes" } else { "No" }.to_string(),
                js.page_count.to_string(),
            ]
        })
        .collect();
    ExportSheet { name: TAB_NAMES[6], header, rows }
}

fn build_cwv_sheet(app: &App) -> ExportSheet {
    let header = vec![
        "ID", "URL", "D: Score", "D: FCP", "D: LCP", "D: CLS", "D: TBT", "D: SI", "M: Score",
        "M: FCP", "M: LCP", "M: CLS", "M: TBT", "M: SI",
    ];
    let rows = app
        .full_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let mut row = vec![(i + 1).to_string(), col(data, 1)];
            for idx in 23..=34 {
                row.push(col(data, idx));
            }
            row
        })
        .collect();
    ExportSheet { name: TAB_NAMES[7], header, rows }
}

fn build_content_sheet(app: &App) -> ExportSheet {
    let mut header = vec!["ID", "URL", "Word Count"];
    for n in 1..=10 {
        header.push(match n {
            1 => "Keyword 1",
            2 => "Keyword 2",
            3 => "Keyword 3",
            4 => "Keyword 4",
            5 => "Keyword 5",
            6 => "Keyword 6",
            7 => "Keyword 7",
            8 => "Keyword 8",
            9 => "Keyword 9",
            _ => "Keyword 10",
        });
    }

    let rows = app
        .content_full_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let mut row = vec![(i + 1).to_string(), col(data, 1), col(data, 18)];
            for idx in 35..45 {
                row.push(col(data, idx));
            }
            row
        })
        .collect();
    ExportSheet { name: TAB_NAMES[8], header, rows }
}

fn build_files_sheet(app: &App) -> ExportSheet {
    let header = vec!["#", "URL", "File Type"];
    let rows = app
        .files_full_filtered_table_data
        .iter()
        .map(|f| vec![f.id.to_string(), f.url.clone(), f.filetype.clone()])
        .collect();
    ExportSheet { name: TAB_NAMES[9], header, rows }
}

fn build_extractor_sheet(app: &App) -> ExportSheet {
    let header = vec!["ID", "URL", "Element", "Snippet"];
    let rows = app
        .extractor_full_filtered_table_data
        .iter()
        .map(|e| vec![e.id.to_string(), e.url.clone(), e.element.clone(), e.snippet.clone()])
        .collect();
    ExportSheet { name: TAB_NAMES[10], header, rows }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_overview_row() -> Vec<String> {
        let mut row = vec![String::new(); 45];
        row[1] = "https://example.com/".to_string();
        row[2] = "Example Title".to_string();
        row[3] = "13".to_string();
        row[4] = "Example H1".to_string();
        row[5] = "10".to_string();
        row[6] = "Example description".to_string();
        row[7] = "20".to_string();
        row[10] = "200 OK".to_string();
        row[12] = "en".to_string();
        row[13] = "index, follow".to_string();
        row[17] = "2048".to_string();
        row[18] = "500".to_string();
        row[23] = "95".to_string();
        row
    }

    fn sample_app() -> App {
        let mut app = App::default();
        app.table_data = vec![sample_overview_row()];
        app.full_filtered_table_data = app.table_data.clone();
        app.content_full_filtered_table_data = app.table_data.clone();
        app.link_scores.insert("https://example.com/".to_string(), 72);
        app.url_to_status
            .insert("https://example.com/".to_string(), "200 OK".to_string());

        app.internal_full_filtered_table_data = vec![crate::models::InternalLink {
            id: 1,
            source: "https://example.com/".to_string(),
            destination: "https://example.com/about".to_string(),
            anchor: "About".to_string(),
            rel: String::new(),
        }];
        app.external_full_filtered_table_data = vec![crate::models::ExternalLink {
            id: 1,
            source: "https://example.com/".to_string(),
            destination: "https://other.com/".to_string(),
            anchor: "Other".to_string(),
            rel: "nofollow".to_string(),
        }];
        app.redirects_full_filtered_table_data = vec![crate::models::RedirectEntry {
            id: 1,
            initial_url: "https://example.com/old".to_string(),
            status_code: 301,
            chain: vec![crate::models::RedirectHop {
                url: "https://example.com/old".to_string(),
                status: 301,
            }],
        }];
        app.images_full_filtered_table_data = vec![crate::models::ImageTableEntry {
            id: 1,
            url: "https://example.com/img.png".to_string(),
            alt: "An image".to_string(),
            status: "200".to_string(),
            size: "12 KB".to_string(),
            page_count: 1,
        }];
        app.css_urls_full_filtered_table_data = vec![crate::models::CssUrl {
            id: 1,
            url: "https://example.com/style.css".to_string(),
            page_count: 1,
        }];
        app.js_urls_full_filtered_table_data = vec![crate::models::JsUrl {
            id: 1,
            url: "https://example.com/app.js".to_string(),
            script_type: "text/javascript".to_string(),
            is_async: true,
            is_defer: false,
            page_count: 1,
        }];
        app.files_full_filtered_table_data = vec![crate::models::FileEntry {
            id: 1,
            url: "https://example.com/doc.pdf".to_string(),
            filetype: "PDF".to_string(),
        }];
        app.extractor_full_filtered_table_data = vec![crate::models::ExtractionTableEntry {
            id: 1,
            url: "https://example.com/".to_string(),
            element: "h1".to_string(),
            snippet: "Example H1".to_string(),
        }];

        app
    }

    #[test]
    fn builds_all_eleven_sheets_with_expected_rows() {
        let app = sample_app();
        let sheets = build_sheets(&app);
        assert_eq!(sheets.len(), 11);
        for sheet in &sheets {
            assert_eq!(sheet.rows.len(), 1, "sheet {} should have exactly 1 row", sheet.name);
        }
        assert_eq!(sheets[0].rows[0][1], "https://example.com/");
        assert_eq!(sheets[0].rows[0][14], "72"); // Link Score
    }

    #[test]
    fn exports_real_xlsx_file_for_small_crawl() {
        let app = sample_app();
        let summary = export_all_tabs(&app).expect("export should succeed");
        assert_eq!(summary.format, "xlsx");
        assert!(summary.path.ends_with(".xlsx"));
        assert!(std::path::Path::new(&summary.path).exists());
        std::fs::remove_file(&summary.path).ok();
    }

    #[test]
    fn falls_back_to_zip_when_a_sheet_exceeds_200k_rows() {
        let mut app = sample_app();
        // Blow one sheet (internal links) past the threshold to force the CSV fallback.
        let template = app.internal_full_filtered_table_data[0].clone();
        app.internal_full_filtered_table_data =
            (0..MAX_XLSX_ROWS + 1).map(|_| template.clone()).collect();

        let summary = export_all_tabs(&app).expect("export should succeed");
        assert_eq!(summary.format, "zip of CSVs (largest sheet exceeded 200k rows)");
        assert!(summary.path.ends_with(".zip"));

        let bytes = std::fs::read(&summary.path).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "content.csv",
                "css.csv",
                "custom_extractor.csv",
                "cwv.csv",
                "external.csv",
                "files.csv",
                "images.csv",
                "internal.csv",
                "javascript.csv",
                "overview.csv",
                "redirects.csv",
            ]
        );
        std::fs::remove_file(&summary.path).ok();
    }
}
