use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let status_line = if let Some(error) = &app.error_message {
        Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red)),
            Span::styled(error, Style::default().fg(Color::Red)),
        ])
    } else if let Some(status) = &app.status_message {
        Line::from(vec![Span::styled(
            status,
            Style::default().fg(Color::Yellow),
        )])
    } else {
        Line::from(vec![
            Span::raw("?: help | "),
            Span::raw("q: quit | "),
            Span::raw("←/→: switch MR | "),
            Span::raw("↑/↓: select job | "),
            Span::raw("Enter: open log | "),
            Span::raw("r: refresh"),
        ])
    };

    let paragraph = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(paragraph, area);
}
