use crate::error::{LabpeepError, Result};
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;

/// Guard that ensures terminal state is restored when dropped
struct TerminalRestoreGuard;

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        // Always attempt to restore terminal, even on panic
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        );
        let _ = crossterm::terminal::enable_raw_mode();
    }
}

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

    // Suspend terminal before launching editor
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    // Create a guard that will restore terminal on drop (even on panic/error)
    let _guard = TerminalRestoreGuard;

    // Launch editor (blocking)
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .map_err(|e| LabpeepError::EditorLaunch(format!("Failed to launch {}: {}", editor, e)))?;

    // Guard will restore terminal when it drops
    // We can also do it explicitly here for clarity
    drop(_guard);

    // Manually restore (in case guard didn't run yet)
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;
    crossterm::terminal::enable_raw_mode()?;

    if !status.success() {
        // Terminal is already restored, safe to return error
        return Err(LabpeepError::EditorLaunch(
            "Editor exited with non-zero status".to_string(),
        ));
    }

    Ok(())
}
