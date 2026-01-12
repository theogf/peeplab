use crate::app::App;
use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let duration = Utc::now().signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let selected_mr = match app.get_selected_mr() {
        Some(mr) => mr,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Comments");
            f.render_widget(block, area);
            return;
        }
    };

    // Show loading state
    if !selected_mr.notes_loaded {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Comments")
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(block, area);
        return;
    }

    let notes = &selected_mr.notes;

    if notes.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Comments")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(block, area);
        return;
    }

    // Calculate available width for text wrapping
    let content_width = area.width.saturating_sub(4) as usize; // Account for borders and padding

    let items: Vec<ListItem> = notes
        .iter()
        .map(|note| {
            let author_style = if note.system {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };

            let time_ago = format_relative_time(note.created_at);

            // Build header line
            let header = Line::from(vec![
                Span::styled(
                    if note.system {
                        "System"
                    } else {
                        &note.author.name
                    },
                    author_style,
                ),
                Span::raw(" • "),
                Span::styled(time_ago, Style::default().fg(Color::DarkGray)),
            ]);

            // Process body - handle multi-line and wrap
            let body_lines: Vec<Line> = note
                .body
                .lines()
                .flat_map(|line| {
                    // Wrap long lines
                    let chars: Vec<char> = line.chars().collect();
                    let mut wrapped_lines = Vec::new();

                    for chunk in chars.chunks(content_width) {
                        let chunk_str: String = chunk.iter().collect();
                        wrapped_lines.push(Line::from(vec![
                            Span::raw("  "), // Indent body
                            Span::raw(chunk_str),
                        ]));
                    }

                    if wrapped_lines.is_empty() {
                        vec![Line::from("  ")] // Empty line
                    } else {
                        wrapped_lines
                    }
                })
                .collect();

            // Combine header and body
            let mut lines = vec![header];
            lines.extend(body_lines);
            lines.push(Line::from("")); // Separator

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Comments (press 'c' to toggle view)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(selected_mr.selected_note_index));

    f.render_stateful_widget(list, area, &mut state);
}
