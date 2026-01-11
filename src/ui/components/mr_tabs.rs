use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs},
    Frame,
};

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.tracked_mrs.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Merge Requests");
        f.render_widget(block, area);
        return;
    }

    let titles: Vec<Line> = app
        .tracked_mrs
        .iter()
        .map(|tracked_mr| {
            let status_indicator = match tracked_mr.pipelines.first() {
                Some(p) => p.status.symbol(),
                None if tracked_mr.loading => "⟳",
                _ => "•",
            };
            Line::from(format!(
                "{} MR #{}: {}",
                status_indicator,
                tracked_mr.mr.iid,
                truncate(&tracked_mr.mr.title, 25)
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Merge Requests"))
        .select(app.selected_mr_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}
