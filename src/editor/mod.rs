use crate::error::{LabpeepError, Result};
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub fn open_in_editor(content: &str) -> Result<()> {
    // Get editor from env or use fallback
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    // Create temporary file
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("labpeep_job_log.txt");

    // Write content to temp file
    let mut file = File::create(&temp_file)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;

    // Restore terminal before launching editor
    crossterm::terminal::disable_raw_mode()?;

    // Show cursor
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    // Launch editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .map_err(|e| LabpeepError::EditorLaunch(format!("Failed to launch {}: {}", editor, e)))?;

    // Re-enable raw mode and hide cursor after editor closes
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;
    crossterm::terminal::enable_raw_mode()?;

    if !status.success() {
        return Err(LabpeepError::EditorLaunch(
            "Editor exited with non-zero status".to_string(),
        ));
    }

    Ok(())
}
