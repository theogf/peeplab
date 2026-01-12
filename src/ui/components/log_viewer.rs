use crate::app::{App, TimestampDisplayMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Highlight search query matches in a line
fn highlight_search_in_line(line: &Line, query: &str) -> Line<'static> {
    // Convert line to plain text for searching
    let line_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    let query_lower = query.to_lowercase();
    let line_lower = line_text.to_lowercase();

    // Find all match positions
    let mut matches: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while let Some(pos) = line_lower[start..].find(&query_lower) {
        let match_start = start + pos;
        let match_end = match_start + query.len();
        matches.push((match_start, match_end));
        start = match_end;
    }

    if matches.is_empty() {
        // Return owned version of the line with plain text
        return Line::from(line_text);
    }

    // Build new line with highlights
    let mut new_spans = Vec::new();
    let mut last_end = 0;

    for (match_start, match_end) in matches {
        // Add text before match
        if match_start > last_end {
            new_spans.push(Span::raw(line_text[last_end..match_start].to_string()));
        }

        // Add highlighted match
        new_spans.push(Span::styled(
            line_text[match_start..match_end].to_string(),
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        last_end = match_end;
    }

    // Add remaining text
    if last_end < line_text.len() {
        new_spans.push(Span::raw(line_text[last_end..].to_string()));
    }

    Line::from(new_spans)
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

    let _log_content = match &app.log_content {
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

    // Use cached processed lines for instant rendering
    let lines = &app.log_processed_lines;

    // Calculate visible range based on scroll offset
    let content_height = log_area.height.saturating_sub(2) as usize; // Account for borders
    let total_lines = lines.len();
    let max_offset = total_lines.saturating_sub(content_height);
    let scroll_offset = app.log_scroll_offset.min(max_offset);

    // Get visible lines with search highlighting
    let visible_lines: Vec<Line> = if total_lines > 0 {
        let start = scroll_offset;
        let end = (scroll_offset + content_height).min(total_lines);

        lines[start..end]
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let line_number = start + idx;

                // Check if this line has a search match
                if !app.search_query.is_empty() && app.search_results.contains(&line_number) {
                    highlight_search_in_line(line, &app.search_query)
                } else {
                    line.clone()
                }
            })
            .collect()
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

    // Build search indicator
    let search_indicator = if !app.search_results.is_empty() {
        format!(
            " [Match {}/{}]",
            app.current_search_result + 1,
            app.search_results.len()
        )
    } else if !app.search_query.is_empty() && !app.is_searching {
        " [No matches]".to_string()
    } else {
        String::new()
    };

    let title = format!(
        "Job Log: {}{}{}{} (q/Esc close, / search, n/N next/prev, t time)",
        job_name,
        if scroll_indicator.is_empty() { " " } else { &scroll_indicator },
        timestamp_indicator,
        search_indicator
    );

    // If searching, show search input bar at the bottom
    let (render_area, search_area) = if app.is_searching {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(log_area);
        (chunks[0], Some(chunks[1]))
    } else {
        (log_area, None)
    };

    let paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, render_area);

    // Render search input bar if in search mode
    if let Some(search_area) = search_area {
        let search_line = Line::from(vec![
            Span::raw("Search: "),
            Span::styled(
                &app.search_query,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "█",
                Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK),
            ),
        ]);

        let search_paragraph = Paragraph::new(search_line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Enter to search, Esc to cancel ")
                .style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(search_paragraph, search_area);
    }
}
