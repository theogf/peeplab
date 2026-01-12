use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

mod app;
mod config;
mod editor;
mod error;
mod events;
mod git;
mod gitlab;
mod ui;

use app::App;
use events::{map_event_to_action, Action, Effect, EventHandler};
use gitlab::GitLabClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let settings = match config::load_config() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("\nPlease create a config file at: {:?}", config::get_config_path()?);
            eprintln!("\nExample config:");
            eprintln!("[gitlab]");
            eprintln!("token = \"glpat-xxxxxxxxxxxxxxxxxxxx\"");
            eprintln!("default_project_id = 12345");
            eprintln!("instance_url = \"https://gitlab.com\"");
            std::process::exit(1);
        }
    };

    // Initialize GitLab client
    let gitlab_client = GitLabClient::new(&settings.gitlab.instance_url, &settings.gitlab.token)?;

    // Determine project ID: use config value or detect from git
    let project_id = match settings.gitlab.default_project_id {
        Some(id) => {
            eprintln!("Using project ID from config: {}", id);
            id
        }
        None => {
            eprintln!("No project ID in config, detecting from git repository...");
            match git::detect_project_from_git() {
                Ok(git_project) => {
                    eprintln!("Detected GitLab project: {}", git_project.path());

                    // Check if the git remote host matches the configured instance
                    let instance_host = settings.gitlab.instance_url
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_end_matches('/');

                    if !git_project.host.contains(instance_host) && !instance_host.contains(&git_project.host) {
                        eprintln!("Warning: Git remote host '{}' doesn't match configured instance '{}'",
                            git_project.host, instance_host);
                    }

                    // Resolve project path to ID via API
                    eprintln!("Resolving project path to ID...");
                    match gitlab_client.get_project_by_path(&git_project.path()).await {
                        Ok(project) => {
                            eprintln!("Found project: {} (ID: {})", project.path_with_namespace, project.id);
                            project.id
                        }
                        Err(e) => {
                            eprintln!("Error: Failed to resolve project '{}': {}", git_project.path(), e);
                            eprintln!("\nPlease either:");
                            eprintln!("1. Add 'default_project_id' to your config file, or");
                            eprintln!("2. Ensure you're in a git repository with a GitLab remote");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    eprintln!("\nPlease either:");
                    eprintln!("1. Add 'default_project_id' to your config file, or");
                    eprintln!("2. Run this command from a git repository with a GitLab remote");
                    std::process::exit(1);
                }
            }
        }
    };

    // Detect current branch if focus_current_branch is enabled
    let current_branch = if settings.app.focus_current_branch {
        match git::get_current_branch() {
            Ok(branch) => {
                eprintln!("Current branch: {}", branch);
                Some(branch)
            }
            Err(e) => {
                eprintln!("Warning: Could not detect current branch: {}", e);
                eprintln!("Showing all open MRs instead");
                None
            }
        }
    } else {
        None
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Create app state
    let mut app = App::new(project_id, current_branch, settings.app.focus_current_branch);

    // Create event handler
    let mut event_handler = EventHandler::new(Duration::from_secs(settings.app.refresh_interval));

    // Create action channel
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    // Initial fetch of merge requests
    let initial_action_tx = action_tx.clone();
    tokio::spawn(async move {
        let _ = initial_action_tx.send(Action::Refresh);
    });

    // Main loop
    let result = run_app(
        &mut terminal,
        &mut app,
        &gitlab_client,
        &mut event_handler,
        &mut action_rx,
        action_tx,
    )
    .await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    gitlab_client: &GitLabClient,
    event_handler: &mut EventHandler,
    action_rx: &mut mpsc::UnboundedReceiver<Action>,
    action_tx: mpsc::UnboundedSender<Action>,
) -> Result<()> {
    loop {
        // Render
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events
        tokio::select! {
            // User input events
            Some(event) = event_handler.next() => {
                let action = map_event_to_action(event, app);
                action_tx.send(action)?;
            }

            // Actions from various sources
            Some(action) = action_rx.recv() => {
                // Update state and get effects
                if let Some(effect) = app.update(action) {
                    handle_effect(effect, gitlab_client, action_tx.clone()).await?;
                }

                if app.should_quit {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn handle_effect(
    effect: Effect,
    gitlab_client: &GitLabClient,
    action_tx: mpsc::UnboundedSender<Action>,
) -> Result<()> {
    match effect {
        Effect::FetchMergeRequests { project_id } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client.get_merge_requests(project_id).await {
                    Ok(mrs) => {
                        let _ = action_tx.send(Action::MergeRequestsLoaded(mrs));
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::FetchPipelines {
            mr_index,
            project_id,
            mr_iid,
        } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client.get_mr_pipelines(project_id, mr_iid).await {
                    Ok(pipelines) => {
                        let _ = action_tx.send(Action::PipelinesLoaded {
                            mr_index,
                            pipelines,
                        });
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::FetchJobs {
            mr_index,
            project_id,
            pipeline_id,
        } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client.get_pipeline_jobs(project_id, pipeline_id).await {
                    Ok(jobs) => {
                        let _ = action_tx.send(Action::JobsLoaded {
                            mr_index,
                            pipeline_id,
                            jobs,
                        });
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::FetchJobTrace { project_id, job_id, job_name } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client.get_job_trace(project_id, job_id).await {
                    Ok(trace) => {
                        let _ = action_tx.send(Action::JobTraceLoaded { job_id, job_name, trace });
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::FetchNotes {
            mr_index,
            project_id,
            mr_iid,
        } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client.get_mr_notes(project_id, mr_iid).await {
                    Ok(notes) => {
                        let _ = action_tx.send(Action::NotesLoaded { mr_index, notes });
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::OpenInEditor(content) => {
            // This needs special handling - must suspend TUI
            tokio::task::spawn_blocking(move || editor::open_in_editor(&content))
                .await??;
        }

        Effect::FetchMergeRequestsByBranch {
            project_id,
            source_branch,
        } => {
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                match client
                    .get_merge_requests_by_branch(project_id, &source_branch)
                    .await
                {
                    Ok(mrs) => {
                        let _ = action_tx.send(Action::MergeRequestsLoaded(mrs));
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::RefreshAll {
            project_id,
            source_branch,
        } => {
            // Fetch merge requests - either filtered by branch or all
            let action_tx = action_tx.clone();
            let client = gitlab_client.clone();
            tokio::spawn(async move {
                let result = if let Some(branch) = source_branch {
                    client.get_merge_requests_by_branch(project_id, &branch).await
                } else {
                    client.get_merge_requests(project_id).await
                };

                match result {
                    Ok(mrs) => {
                        let _ = action_tx.send(Action::MergeRequestsLoaded(mrs));
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::ApiError(e.to_string()));
                    }
                }
            });
        }

        Effect::OpenUrl(url) => {
            // Open URL in default browser
            tokio::task::spawn_blocking(move || {
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();

                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&url).spawn();

                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd").args(&["/C", "start", &url]).spawn();
            })
            .await?;
        }
    }

    Ok(())
}
