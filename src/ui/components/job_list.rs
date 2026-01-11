use crate::app::App;
use crate::gitlab::JobStatus;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

fn format_duration(duration: Option<f64>) -> String {
    match duration {
        Some(d) => {
            let minutes = (d / 60.0) as u64;
            let seconds = (d % 60.0) as u64;
            if minutes > 0 {
                format!("{}m {:02}s", minutes, seconds)
            } else {
                format!("{}s", seconds)
            }
        }
        None => "-".to_string(),
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let _selected_mr = match app.get_selected_mr() {
        Some(mr) => mr,
        None => {
            let block = Block::default().borders(Borders::ALL).title("Jobs");
            f.render_widget(block, area);
            return;
        }
    };

    let jobs = match app.get_selected_jobs() {
        Some(jobs) if !jobs.is_empty() => jobs,
        _ => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Jobs")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(block, area);
            return;
        }
    };

    let rows: Vec<Row> = jobs
        .iter()
        .map(|job| {
            let (status_color, status_text) = match job.status {
                JobStatus::Success => (Color::Green, format!("{} success", job.status.symbol())),
                JobStatus::Failed => (Color::Red, format!("{} failed", job.status.symbol())),
                JobStatus::Running => (Color::Yellow, format!("{} running", job.status.symbol())),
                JobStatus::Pending => (Color::Blue, format!("{} pending", job.status.symbol())),
                JobStatus::Canceled => (Color::Gray, format!("{} canceled", job.status.symbol())),
                JobStatus::Skipped => (Color::DarkGray, format!("{} skipped", job.status.symbol())),
                _ => (Color::Gray, format!("{} {:?}", job.status.symbol(), job.status).to_lowercase()),
            };

            Row::new(vec![
                Cell::from(job.stage.clone()),
                Cell::from(job.name.clone()),
                Cell::from(status_text).style(Style::default().fg(status_color)),
                Cell::from(format_duration(job.duration)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Stage", "Job Name", "Status", "Duration"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title("Jobs"))
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ");

    let mut state = TableState::default();
    state.select(Some(app.selected_job_index));

    f.render_stateful_widget(table, area, &mut state);
}
