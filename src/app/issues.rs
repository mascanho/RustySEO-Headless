use crate::models::App;
use crate::helpers::issues::IssueAnalyzer;

impl App {
    /// Populate issues_table_data with real crawled data analysis
    pub fn update_issues_from_crawled_data(&mut self) {
        self.issues_table_data = IssueAnalyzer::generate_issues_table_data(&self.page_data);
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(&self, issue_type: &str) -> Vec<String> {
        IssueAnalyzer::get_urls_for_issue(&self.page_data, issue_type)
    }
}
