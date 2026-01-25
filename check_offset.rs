
fn main() {
    let mut state = ratatui::widgets::ListState::default();
    let off = state.offset();
    println!("Offset: {}", off);
}
