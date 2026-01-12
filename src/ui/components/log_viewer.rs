use crate::app::{App, TimestampDisplayMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use regex::Regex;

/// Parse and format log line based on timestamp display mode
fn process_log_line(line: &str, mode: &TimestampDisplayMode) -> String {
    // Regex to match ISO timestamps at the start of the line
    // Matches patterns like: 2024-01-15T10:30:45.123Z or 2024-01-15T10:30:45+00:00
    let re = Regex::new(r"^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2})(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?\s+").unwrap();

    match mode {
        TimestampDisplayMode::Hidden => {
            // Strip timestamp completely
            re.replace(line, "").to_string()
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

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Calculate the log viewer area (90% width, 90% height, centered)
    let log_area = centered_rect(90, 90, area);

    // Clear the background to prevent rendering artifacts
    f.render_widget(Clear, log_area);

    let log_content = match &app.log_content {
        Some(content) => content,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Job Log")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(block, log_area);
            return;
        }
    };

    let job_name = app
        .log_job_name
        .as_deref()
        .unwrap_or("Unknown Job");

    // Process timestamps and parse ANSI codes, converting to ratatui Lines
    let lines: Vec<Line> = log_content
        .lines()
        .map(|line| {
            // First, process the timestamp based on display mode
            let processed_line = process_log_line(line, &app.timestamp_mode);

            // Then parse ANSI escape sequences
            match ansi_to_tui::IntoText::into_text(&processed_line) {
                Ok(text) => {
                    // Convert ratatui Text to Line
                    if text.lines.is_empty() {
                        Line::from("")
                    } else {
                        text.lines[0].clone()
                    }
                }
                Err(_) => {
                    // If parsing fails, show raw text
                    Line::from(processed_line)
                }
            }
        })
        .collect();

    // Calculate visible range based on scroll offset
    let content_height = log_area.height.saturating_sub(2) as usize; // Account for borders
    let total_lines = lines.len();
    let max_offset = total_lines.saturating_sub(content_height);
    let scroll_offset = app.log_scroll_offset.min(max_offset);

    // Get visible lines
    let visible_lines: Vec<Line> = if total_lines > 0 {
        let start = scroll_offset;
        let end = (scroll_offset + content_height).min(total_lines);
        lines[start..end].to_vec()
    } else {
        vec![Line::from("(empty log)")]
    };

    let scroll_indicator = if total_lines > content_height {
        format!(
            " [{}/{}] ",
            scroll_offset + 1,
            max_offset + 1
        )
    } else {
        String::new()
    };

    let timestamp_indicator = match &app.timestamp_mode {
        TimestampDisplayMode::Hidden => "[Timestamps: Hidden]",
        TimestampDisplayMode::DateOnly => "[Timestamps: Date]",
        TimestampDisplayMode::Full => "[Timestamps: Full]",
    };

    let title = format!(
        "Job Log: {}{}{} (q/Esc to close, ↑↓/jk scroll, t toggle time, PgUp/PgDn/Home/End)",
        job_name,
        if scroll_indicator.is_empty() { " " } else { &scroll_indicator },
        timestamp_indicator
    );

    let paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, log_area);
}
