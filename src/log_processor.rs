use crate::app::TimestampDisplayMode;
use ratatui::text::Line;
use regex::Regex;

/// Strip GitLab CI log prefixes like 00E, 00O, section markers, etc.
fn strip_gitlab_prefixes(line: &str) -> String {
    // Strip section markers first (these lines should be hidden entirely)
    if line.contains("section_start:") || line.contains("section_end:") {
        return String::new();
    }

    // Use regex to strip GitLab CI prefixes
    // These can appear at the start: 00E, 00O, 000, 001, 002, etc.
    // Format is typically: "00E " or "00O " followed by timestamp and message
    // Also handle null bytes and ANSI escape sequences mixed in
    let prefix_re = Regex::new(r"^(?:\x00*|\x1b\[[0-9;]*[A-Za-z])*(?:00[0-9A-Fa-fEO])(?:\x00*|\x1b\[[0-9;]*[A-Za-z])*\s*").unwrap();

    let result = prefix_re.replace(line, "");
    result.to_string()
}

/// Parse and format log line based on timestamp display mode
fn process_log_line(line: &str, mode: &TimestampDisplayMode) -> String {
    // First, check for section markers (these lines should be hidden entirely)
    if line.contains("section_start:") || line.contains("section_end:") {
        return String::new();
    }

    // Regex to match ISO timestamps followed by GitLab CI prefixes
    // Format: 2026-01-12T10:35:38.187431Z 00O [0KMessage...
    // Captures: (date) (time) and skips the prefix part
    let re = Regex::new(r"^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2})(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?\s+\d{2}[OE]\s+(?:\[0K)?").unwrap();

    match mode {
        TimestampDisplayMode::Hidden => {
            // Strip timestamp and prefix completely
            if let Some(m) = re.find(line) {
                line[m.end()..].to_string()
            } else {
                // Fallback: just strip any prefix at the start
                strip_gitlab_prefixes(line)
            }
        }
        TimestampDisplayMode::DateOnly => {
            // Show only the date part
            if let Some(caps) = re.captures(line) {
                let date = &caps[1];
                let rest = &line[caps.get(0).unwrap().end()..];
                format!("{} {}", date, rest)
            } else {
                line.to_string()
            }
        }
        TimestampDisplayMode::Full => {
            // Show date and time (but not milliseconds/timezone)
            if let Some(caps) = re.captures(line) {
                let date = &caps[1];
                let time = &caps[2];
                let rest = &line[caps.get(0).unwrap().end()..];
                format!("{} {} {}", date, time, rest)
            } else {
                line.to_string()
            }
        }
    }
}

/// Process all log lines: strip prefixes, format timestamps, parse ANSI codes
pub fn process_log_content(content: &str, mode: &TimestampDisplayMode) -> Vec<Line<'static>> {
    content
        .lines()
        .map(|line| {
            // First, process the timestamp based on display mode
            let processed_line = process_log_line(line, mode);

            // Then parse ANSI escape sequences
            match ansi_to_tui::IntoText::into_text(&processed_line) {
                Ok(text) => {
                    // Convert ratatui Text to Line
                    if text.lines.is_empty() {
                        Line::from("").to_owned()
                    } else {
                        text.lines[0].clone().to_owned()
                    }
                }
                Err(_) => {
                    // If parsing fails, show raw text
                    Line::from(processed_line).to_owned()
                }
            }
        })
        .collect()
}
