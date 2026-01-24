use crate::tui_println;

pub fn export_data(tab: &str) -> Result<(), String> {
    match tab {
        "overview" => tui_println!("Exporting Overview Data to CSV"),
        _ => return Err(format!("Unsupported tab: {}", tab)),
    }
    Ok(())
}
