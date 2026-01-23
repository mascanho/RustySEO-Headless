use crate::crawler::helpers::html_parser::PageData;
use crate::models::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub url: Option<String>,
    pub status: Option<String>,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub level: usize,
}

impl TreeNode {
    pub fn new(
        id: String,
        label: String,
        url: Option<String>,
        status: Option<String>,
        level: usize,
    ) -> Self {
        Self {
            id,
            label,
            url,
            status,
            children: Vec::new(),
            is_expanded: false,
            level,
        }
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    pub fn render_tree_line(&self, is_selected: bool) -> Line<'_> {
        let mut spans = Vec::new();

        // Add indentation based on level
        for _ in 0..self.level {
            spans.push(Span::raw("  "));
        }

        // Add expand/collapse indicator if has children
        if !self.children.is_empty() {
            let indicator = if self.is_expanded { "▼" } else { "▶" };
            spans.push(Span::styled(
                format!(" {} ", indicator),
                Style::default().fg(Color::Yellow),
            ));
        } else {
            spans.push(Span::raw("   "));
        }

        // Add status indicator
        if let Some(status) = &self.status {
            let (icon, color) = match status.as_str() {
                "200" | "OK" => ("●", Color::Green),
                "404" => ("●", Color::Red),
                "500" | "Error" => ("●", Color::Red),
                "Redirect" => ("●", Color::Yellow),
                _ => ("●", Color::Gray),
            };
            spans.push(Span::styled(icon, Style::default().fg(color)));
            spans.push(Span::raw(" "));
        } else {
            spans.push(Span::raw("  "));
        }

        // Add label with selection highlighting
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        spans.push(Span::styled(&self.label, label_style));

        Line::from(spans)
    }

    pub fn flatten_to_list<'a>(
        &'a self,
        items: &mut Vec<ListItem<'a>>,
        selected_index: usize,
        current_index: &mut usize,
    ) {
        // Add current node
        let is_selected = *current_index == selected_index;
        items.push(ListItem::new(self.render_tree_line(is_selected)));
        *current_index += 1;

        // Add children if expanded
        if self.is_expanded {
            for child in &self.children {
                child.flatten_to_list(items, selected_index, current_index);
            }
        }
    }
}

pub fn build_tree_structure(page_data: &[PageData]) -> TreeNode {
    let mut root = TreeNode::new(
        "root".to_string(),
        "Website Structure".to_string(),
        None,
        None,
        0,
    );

    if page_data.is_empty() {
        return root;
    }

    // Group pages by domain/path
    let mut domain_map: BTreeMap<String, Vec<&PageData>> = BTreeMap::new();

    for page in page_data {
        if let Ok(url) = url::Url::parse(&page.url) {
            let domain = url.domain().unwrap_or("unknown");
            domain_map.entry(domain.to_string()).or_default().push(page);
        } else {
            domain_map
                .entry("invalid".to_string())
                .or_default()
                .push(page);
        }
    }

    // Create domain nodes
    for (domain, pages) in domain_map {
        let mut domain_node = TreeNode::new(
            format!("domain-{}", domain),
            format!("🌐 {}", domain),
            None,
            None,
            1,
        );

        // Group by path segments
        let mut path_map: BTreeMap<String, Vec<&PageData>> = BTreeMap::new();

        for page in pages {
            if let Ok(url) = url::Url::parse(&page.url) {
                let path = url.path();
                let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

                if path_segments.is_empty() {
                    path_map.entry("/".to_string()).or_default().push(page);
                } else {
                    let parent_path = format!("/{}", path_segments[0]);
                    path_map.entry(parent_path).or_default().push(page);
                }
            }
        }

        // Create path nodes
        for (path, mut path_pages) in path_map {
            // Sort pages within the path by URL for stability
            path_pages.sort_by(|a, b| a.url.cmp(&b.url));

            let mut path_node = TreeNode::new(
                format!("path-{}-{}", domain, path),
                format!(
                    "📁 {}",
                    if path == "/" {
                        "root"
                    } else {
                        path.trim_start_matches('/')
                    }
                ),
                None,
                None,
                2,
            );

            // Add individual pages
            for page in path_pages {
                let page_label = if page.title.is_empty() {
                    format!(
                        "Untitled ({})",
                        page.url.split('/').last().unwrap_or("page")
                    )
                } else {
                    page.title.clone()
                };

                let page_node = TreeNode::new(
                    format!("page-{}", page.id),
                    format!("📄 {}", page_label),
                    Some(page.url.clone()),
                    Some(page.status.clone()),
                    3,
                );

                path_node.add_child(page_node);
            }

            domain_node.add_child(path_node);
        }

        root.add_child(domain_node);
    }

    root
}

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block) {
    let tree_root = build_tree_structure(&app.page_data);

    // Update tree state if needed
    if app.tree_view_expanded_nodes.is_empty() && !app.page_data.is_empty() {
        // Auto-expand first level
        app.tree_view_expanded_nodes.insert("root".to_string());
    }

    // Apply expansion state to tree
    let mut tree_root = tree_root;
    apply_expansion_state(&mut tree_root, &app.tree_view_expanded_nodes);

    // Flatten tree to list items
    let mut items = Vec::new();
    let mut current_index = 0;
    tree_root.flatten_to_list(&mut items, app.tree_view_selected_index, &mut current_index);

    // Create list widget
    let list = List::new(items)
        .block(content_block.title(Span::styled(
            " Site Tree ",
            Style::default().fg(Color::Yellow),
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.tree_view_state);
}

fn apply_expansion_state(node: &mut TreeNode, expanded_nodes: &std::collections::HashSet<String>) {
    node.is_expanded = expanded_nodes.contains(&node.id);

    for child in &mut node.children {
        apply_expansion_state(child, expanded_nodes);
    }
}
