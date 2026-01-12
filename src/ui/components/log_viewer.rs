use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Clear the background to prevent rendering artifacts
    f.render_widget(Clear, area);

    let log_content = match &app.log_content {
        Some(content) => content,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Job Log")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(block, area);
            return;
        }
    };

    let job_name = app
        .log_job_name
        .as_deref()
        .unwrap_or("Unknown Job");

    // Parse ANSI codes and convert to ratatui Lines
    let lines: Vec<Line> = log_content
        .lines()
        .map(|line| {
            // Use ansi-to-tui to parse ANSI escape sequences
            match ansi_to_tui::IntoText::into_text(&line) {
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
                    Line::from(line.to_string())
                }
            }
        })
        .collect();

    // Calculate visible range based on scroll offset
    let content_height = area.height.saturating_sub(2) as usize; // Account for borders
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

    let title = format!(
        "Job Log: {}{}(q/Esc to close, ↑↓/jk to scroll, Home/End, PgUp/PgDn)",
        job_name,
        if scroll_indicator.is_empty() { " " } else { &scroll_indicator }
    );

    let paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
