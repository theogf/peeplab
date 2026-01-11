use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Resize,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _tx: mpsc::UnboundedSender<AppEvent>, // Keep alive for senders
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn input handler
        let input_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(true) = event::poll(Duration::from_millis(100)) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if input_tx.send(AppEvent::Input(key)).is_err() {
                            break;
                        }
                    } else if let Ok(Event::Resize(_, _)) = event::read() {
                        if input_tx.send(AppEvent::Resize).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Spawn tick handler
        let tick_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tick_tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}

use crate::app::{App, AppMode};
use crate::events::actions::Action;

pub fn map_event_to_action(event: AppEvent, app: &App) -> Action {
    match event {
        AppEvent::Input(key) => match app.mode {
            AppMode::Normal => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Action::Quit
                }
                KeyCode::Char('?') => Action::ShowHelp,
                KeyCode::Left | KeyCode::Char('h') => Action::PrevMr,
                KeyCode::Right | KeyCode::Char('l') => Action::NextMr,
                KeyCode::Up | KeyCode::Char('k') => Action::PrevJob,
                KeyCode::Down | KeyCode::Char('j') => Action::NextJob,
                KeyCode::Char('[') => Action::PrevPipeline,
                KeyCode::Char(']') => Action::NextPipeline,
                KeyCode::Enter => Action::OpenSelectedJobLog,
                KeyCode::Char('r') => Action::Refresh,
                KeyCode::Char('d') => Action::RemoveCurrentMr,
                _ => Action::None,
            },
            AppMode::SelectingMr => match key.code {
                KeyCode::Esc => Action::None, // Exit selection mode
                KeyCode::Char('q') => Action::Quit,
                _ => Action::None,
            },
            AppMode::ShowingHelp => match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => Action::HideHelp,
                _ => Action::None,
            },
        },
        AppEvent::Tick => Action::Tick,
        AppEvent::Resize => Action::None,
    }
}
