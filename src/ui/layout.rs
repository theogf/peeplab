use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::components;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // MR Tabs
            Constraint::Length(10), // Pipeline list
            Constraint::Min(10),    // Jobs table
            Constraint::Length(2),  // Status bar
        ])
        .split(f.area());

    components::mr_tabs::render(f, app, chunks[0]);
    components::pipeline_list::render(f, app, chunks[1]);
    components::job_list::render(f, app, chunks[2]);
    components::status_bar::render(f, app, chunks[3]);
}
