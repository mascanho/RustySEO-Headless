use crate::models::App;
use directories::ProjectDirs;

impl App {
    pub fn set_sidebar_tab(&mut self, index: usize) {
        if index < 7 {
            self.sidebar_tab = index;
            self.sidebar_scroll = 0;
            self.sidebar_visible = true;
        }
    }

    pub fn sidebar_scroll_down(&mut self) {
        self.sidebar_scroll = self.sidebar_scroll.saturating_add(1);
    }

    pub fn sidebar_scroll_up(&mut self) {
        self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1);
    }

    pub fn next_bookmark(&mut self) {
        if !self.bookmarks.is_empty() {
            self.bookmark_index = (self.bookmark_index + 1) % self.bookmarks.len();
            self.bookmarks_state.select(Some(self.bookmark_index));
        }
    }

    pub fn previous_bookmark(&mut self) {
        if !self.bookmarks.is_empty() {
            self.bookmark_index = if self.bookmark_index == 0 {
                self.bookmarks.len() - 1
            } else {
                self.bookmark_index - 1
            };
            self.bookmarks_state.select(Some(self.bookmark_index));
        }
    }

    pub fn remove_selected_bookmark(&mut self) {
        if !self.bookmarks.is_empty() && self.bookmark_index < self.bookmarks.len() {
            let url = self.bookmarks[self.bookmark_index].clone();
            crate::db::remove_bookmark(&url);
            self.bookmarks = crate::db::load_bookmarks();
            // Update ListState to match the bookmark_index
            self.sync_bookmarks_state();
        }
    }

    /// Sync bookmark_index with bookmarks_state
    pub fn sync_bookmarks_state(&mut self) {
        if self.bookmarks.is_empty() {
            self.bookmarks_state.select(None);
            self.bookmark_index = 0;
        } else {
            // Ensure bookmark_index is within bounds
            if self.bookmark_index >= self.bookmarks.len() {
                self.bookmark_index = self.bookmarks.len() - 1;
            }
            self.bookmarks_state.select(Some(self.bookmark_index));
        }
    }

    pub fn toggle_bookmark_subview(&mut self) {
        self.bookmark_subview = if self.bookmark_subview == 0 { 1 } else { 0 };
        self.last_crawled_index = 0;
    }

    pub fn next_last_crawled(&mut self) {
        let recent_urls = self.get_recent_crawled_urls();
        if !recent_urls.is_empty() {
            self.last_crawled_index = (self.last_crawled_index + 1) % recent_urls.len();
        }
    }

    pub fn previous_last_crawled(&mut self) {
        let recent_urls = self.get_recent_crawled_urls();
        if !recent_urls.is_empty() {
            self.last_crawled_index = if self.last_crawled_index == 0 {
                recent_urls.len() - 1
            } else {
                self.last_crawled_index - 1
            };
        }
    }

    pub fn get_recent_crawled_urls(&self) -> Vec<String> {
        let project_dirs = ProjectDirs::from("", "", "rustyseo").unwrap();
        let recent_crawls_path = project_dirs.data_dir().join("recent-crawls.json");

        if recent_crawls_path.exists() {
            std::fs::read_to_string(&recent_crawls_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    // Tree View Methods
    pub fn tree_view_next(&mut self) {
        let tree_root = crate::ui::sidebar::tree_view::build_tree_structure(&self.page_summaries);
        let total_items = self.count_tree_items(&tree_root);

        if total_items > 0 {
            self.tree_view_selected_index = (self.tree_view_selected_index + 1) % total_items;
            self.tree_view_state
                .select(Some(self.tree_view_selected_index));
        }
    }

    pub fn tree_view_previous(&mut self) {
        let tree_root = crate::ui::sidebar::tree_view::build_tree_structure(&self.page_summaries);
        let total_items = self.count_tree_items(&tree_root);

        if total_items > 0 {
            self.tree_view_selected_index = if self.tree_view_selected_index == 0 {
                total_items - 1
            } else {
                self.tree_view_selected_index - 1
            };
            self.tree_view_state
                .select(Some(self.tree_view_selected_index));
        }
    }

    pub fn tree_view_toggle_expand(&mut self) {
        let tree_root = crate::ui::sidebar::tree_view::build_tree_structure(&self.page_summaries);
        let node_id = self.get_node_id_at_index(&tree_root, self.tree_view_selected_index);

        if let Some(node_id) = node_id {
            if self.tree_view_expanded_nodes.contains(&node_id) {
                self.tree_view_expanded_nodes.remove(&node_id);
            } else {
                self.tree_view_expanded_nodes.insert(node_id);
            }
        }
    }

    pub fn tree_view_expand_all(&mut self) {
        let tree_root = crate::ui::sidebar::tree_view::build_tree_structure(&self.page_summaries);
        let mut all_node_ids = std::collections::HashSet::new();
        self.collect_all_node_ids(&tree_root, &mut all_node_ids);
        self.tree_view_expanded_nodes = all_node_ids;
    }

    pub fn tree_view_collapse_all(&mut self) {
        self.tree_view_expanded_nodes.clear();
        // Always keep root expanded
        self.tree_view_expanded_nodes.insert("root".to_string());
    }

    fn count_tree_items(&self, node: &crate::ui::sidebar::tree_view::TreeNode) -> usize {
        let mut count = 1; // Count this node

        if self.tree_view_expanded_nodes.contains(&node.id) {
            for child in &node.children {
                count += self.count_tree_items(child);
            }
        }

        count
    }

    fn get_node_id_at_index(
        &self,
        node: &crate::ui::sidebar::tree_view::TreeNode,
        target_index: usize,
    ) -> Option<String> {
        let mut current_index = 0;
        self.find_node_id_at_index_recursive(node, target_index, &mut current_index)
    }

    fn find_node_id_at_index_recursive(
        &self,
        node: &crate::ui::sidebar::tree_view::TreeNode,
        target_index: usize,
        current_index: &mut usize,
    ) -> Option<String> {
        if *current_index == target_index {
            return Some(node.id.clone());
        }
        *current_index += 1;

        if self.tree_view_expanded_nodes.contains(&node.id) {
            for child in &node.children {
                if let Some(id) =
                    self.find_node_id_at_index_recursive(child, target_index, current_index)
                {
                    return Some(id);
                }
            }
        }

        None
    }

    fn collect_all_node_ids(
        &self,
        node: &crate::ui::sidebar::tree_view::TreeNode,
        node_ids: &mut std::collections::HashSet<String>,
    ) {
        node_ids.insert(node.id.clone());

        for child in &node.children {
            self.collect_all_node_ids(child, node_ids);
        }
    }
}
