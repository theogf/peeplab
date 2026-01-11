use crate::app::App;
use crate::gitlab::PipelineStatus;
use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

fn format_relative_time(dt: &chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let selected_mr = match app.get_selected_mr() {
        Some(mr) => mr,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Pipelines");
            f.render_widget(block, area);
            return;
        }
    };

    if selected_mr.pipelines.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Pipelines")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(block, area);
        return;
    }

    let items: Vec<ListItem> = selected_mr
        .pipelines
        .iter()
        .map(|pipeline| {
            let status_color = match pipeline.status {
                PipelineStatus::Success => Color::Green,
                PipelineStatus::Failed => Color::Red,
                PipelineStatus::Running => Color::Yellow,
                PipelineStatus::Canceled => Color::DarkGray,
                _ => Color::Gray,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", pipeline.status.symbol()),
                    Style::default().fg(status_color),
                ),
                Span::raw(format!("Pipeline #{} ", pipeline.iid)),
                Span::styled(
                    format!("({:?})", pipeline.status).to_lowercase(),
                    Style::default().fg(status_color),
                ),
                Span::raw(" - "),
                Span::styled(
                    format_relative_time(&pipeline.created_at),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(selected_mr.selected_pipeline_index));

    f.render_stateful_widget(list, area, &mut state);
}
