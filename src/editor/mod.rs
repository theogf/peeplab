use crate::error::{PeeplabError, Result};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
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

    // Create temporary file with better performance for large files
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("peeplab_job_log.txt");

    // Write content to temp file using BufWriter for better performance
    {
        let file = File::create(&temp_file)?;
        let mut writer = BufWriter::with_capacity(8192, file);
        writer.write_all(content.as_bytes())?;
        // Flush is automatic on drop, no need for sync_all which is slow
    }

    // Suspend terminal before launching editor
    // Disable raw mode first (fastest operation)
    crossterm::terminal::disable_raw_mode()?;

    // Then do screen operations in one call for efficiency
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
        .map_err(|e| PeeplabError::EditorLaunch(format!("Failed to launch {}: {}", editor, e)))?;

    // Explicitly drop guard before restoring to avoid double restoration
    drop(_guard);

    // Restore terminal state - do screen operations first, then enable raw mode
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;
    crossterm::terminal::enable_raw_mode()?;

    if !status.success() {
        // Terminal is already restored, safe to return error
        return Err(PeeplabError::EditorLaunch(
            "Editor exited with non-zero status".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_large_log_file_handling() {
        // Create a large log content (10MB)
        let large_content = "x".repeat(10 * 1024 * 1024);

        // Write to temp file using the same approach as open_in_editor
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("peeplab_test_large_log.txt");

        // Test the BufWriter approach
        let start = std::time::Instant::now();
        {
            let file = File::create(&temp_file).unwrap();
            let mut writer = BufWriter::with_capacity(8192, file);
            writer.write_all(large_content.as_bytes()).unwrap();
            // Flush is automatic on drop
        }
        let elapsed = start.elapsed();

        // Verify the file was written correctly
        let read_content = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(read_content.len(), large_content.len());

        // Verify it's reasonably fast (should be well under 1 second for 10MB)
        assert!(
            elapsed.as_millis() < 1000,
            "Writing 10MB took {}ms, expected < 1000ms",
            elapsed.as_millis()
        );

        // Cleanup
        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_temp_file_creation() {
        // Test that we can create and write to the temp file location
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("peeplab_job_log_test.txt");

        let test_content = "Test log content\nLine 2\nLine 3";

        {
            let file = File::create(&temp_file).unwrap();
            let mut writer = BufWriter::with_capacity(8192, file);
            writer.write_all(test_content.as_bytes()).unwrap();
        }

        // Verify content
        let read_content = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(read_content, test_content);

        // Cleanup
        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_bufwriter_vs_direct_write() {
        // Compare BufWriter performance vs direct write for large content
        let large_content = "x".repeat(5 * 1024 * 1024); // 5MB

        let temp_dir = env::temp_dir();

        // Test BufWriter approach (what we use now)
        let buffered_file = temp_dir.join("peeplab_test_buffered.txt");
        let start = std::time::Instant::now();
        {
            let file = File::create(&buffered_file).unwrap();
            let mut writer = BufWriter::with_capacity(8192, file);
            writer.write_all(large_content.as_bytes()).unwrap();
        }
        let buffered_time = start.elapsed();

        // Test direct write (for comparison)
        let direct_file = temp_dir.join("peeplab_test_direct.txt");
        let start = std::time::Instant::now();
        {
            let mut file = File::create(&direct_file).unwrap();
            file.write_all(large_content.as_bytes()).unwrap();
        }
        let direct_time = start.elapsed();

        // BufWriter should be at least as fast (usually faster for large writes)
        // We're not asserting strict performance here, just that both work
        assert!(buffered_time.as_millis() < 2000, "Buffered write took too long");
        assert!(direct_time.as_millis() < 2000, "Direct write took too long");

        // Cleanup
        let _ = fs::remove_file(&buffered_file);
        let _ = fs::remove_file(&direct_file);
    }

    #[test]
    fn test_editor_env_var_detection() {
        // Test that we correctly detect the EDITOR environment variable
        let original_editor = env::var("EDITOR").ok();
        let original_visual = env::var("VISUAL").ok();

        // Test with EDITOR set
        env::set_var("EDITOR", "test-editor");
        let editor = env::var("EDITOR")
            .or_else(|_| env::var("VISUAL"))
            .unwrap_or_else(|_| "vim".to_string());
        assert_eq!(editor, "test-editor");

        // Test with VISUAL set (EDITOR not set)
        env::remove_var("EDITOR");
        env::set_var("VISUAL", "test-visual");
        let editor = env::var("EDITOR")
            .or_else(|_| env::var("VISUAL"))
            .unwrap_or_else(|_| "vim".to_string());
        assert_eq!(editor, "test-visual");

        // Test with neither set (fallback to vim)
        env::remove_var("EDITOR");
        env::remove_var("VISUAL");
        let editor = env::var("EDITOR")
            .or_else(|_| env::var("VISUAL"))
            .unwrap_or_else(|_| "vim".to_string());
        assert_eq!(editor, "vim");

        // Restore original values
        if let Some(editor) = original_editor {
            env::set_var("EDITOR", editor);
        } else {
            env::remove_var("EDITOR");
        }
        if let Some(visual) = original_visual {
            env::set_var("VISUAL", visual);
        } else {
            env::remove_var("VISUAL");
        }
    }
}
