# labpeep - GitLab Pipeline Monitor TUI

A terminal user interface (TUI) application for monitoring GitLab CI/CD pipelines and merge requests.

## Features

- **Monitor Multiple MRs**: Track multiple merge requests simultaneously in tabs
- **Pipeline Status**: View pipeline statuses with visual indicators (✓/✗/⟳)
- **Job Details**: See all jobs in a pipeline with their statuses and durations
- **Log Viewing**: Open failed job logs directly in your preferred editor
- **Auto-refresh**: Automatically refresh pipeline statuses at configurable intervals
- **Keyboard Navigation**: Fast, keyboard-driven interface

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/labpeep`.

## Configuration

Create a configuration file at `~/.config/labpeep/config.toml`:

```toml
[gitlab]
# Your GitLab personal access token (requires 'read_api' scope)
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

# The project ID you want to monitor (optional - can be auto-detected from git)
# default_project_id = 12345

# GitLab instance URL (defaults to gitlab.com)
instance_url = "https://gitlab.com"

[app]
# Auto-refresh interval in seconds (default: 30)
refresh_interval = 30

# Maximum number of MRs to track simultaneously (default: 5)
max_tracked_mrs = 5

# Focus on MR for current git branch only (default: true)
# When true, only shows the MR associated with your current branch
# When false, shows all open MRs
focus_current_branch = true

[ui]
# Show timestamps in relative format (default: true)
relative_timestamps = true

# Color theme: "dark" or "light" (default: "dark")
theme = "dark"

[editor]
# Override $EDITOR environment variable if needed
# If not set, uses $EDITOR, $VISUAL, or falls back to vim
# custom_editor = "nvim"
```

### Getting Your GitLab Token

1. Go to your GitLab instance (e.g., https://gitlab.com)
2. Navigate to User Settings → Access Tokens
3. Create a new token with the `read_api` scope
4. Copy the token to your config file

### Finding Your Project ID

**Option 1: Auto-detection (Recommended)**

If you run `labpeep` from within a git repository that has a GitLab remote, the project ID will be automatically detected! Just omit the `default_project_id` field in your config.

Supported remote URL formats:
- SSH: `git@gitlab.com:namespace/project.git`
- HTTPS: `https://gitlab.com/namespace/project.git`

**Option 2: Manual Configuration**

1. Go to your project in GitLab
2. The project ID is shown below the project name on the project's main page
3. Or find it in Settings → General → Project ID
4. Add it to your config as `default_project_id = 12345`

## Usage

```bash
labpeep
```

### Keyboard Controls

- `?`: Show help popup with all keyboard shortcuts
- `q` or `Ctrl+C`: Quit the application
- `←` / `→` or `h` / `l`: Switch between merge request tabs
- `↑` / `↓` or `k` / `j`: Navigate jobs in the selected pipeline
- `[` / `]`: Switch between pipelines for the current MR
- `Enter`: Open the selected job's log in your editor
- `r`: Refresh all data
- `d`: Remove the current MR from tracking

**Tip:** Press `?` at any time to see the help popup with all available commands!

### Branch-Focused Mode

By default, `labpeep` focuses on the MR for your current git branch only. This keeps you focused on your current work without distraction from other open MRs.

**How it works:**
- When you run `labpeep` from a git repository, it detects your current branch
- It fetches only the MR that has this branch as its source branch
- You see pipeline status and jobs for just your current work

**To see all open MRs instead:**
Set `focus_current_branch = false` in your config file's `[app]` section.

**Benefits:**
- Less clutter - only see what you're working on
- Faster loading - fewer API calls
- Better focus - track just your current MR's CI/CD status

## How It Works

1. **Launch**: The app loads your configuration and fetches merge requests (for your current branch if focus mode is enabled)
2. **Display**: Each MR is shown in a tab with its latest pipelines
3. **Navigation**: Use keyboard shortcuts to navigate between MRs, pipelines, and jobs
4. **Log Viewing**: When you press Enter on a job, it fetches the job log and opens it in your configured editor
5. **Auto-refresh**: The app automatically refreshes pipeline statuses based on your configured interval

## Architecture

The application follows The Elm Architecture (TEA) pattern:

- **Model**: Application state (`app.rs`)
- **View**: TUI rendering (`ui/` modules)
- **Update**: State transitions based on actions (`app.rs`)
- **Effects**: Asynchronous GitLab API calls (`gitlab/client.rs`)

## Troubleshooting

### "Config file not found" error

Create the config file at `~/.config/labpeep/config.toml` with your GitLab token and project ID.

### "Authentication failed" error

- Verify your GitLab token is correct
- Ensure the token has the `read_api` scope
- Check that the token hasn't expired

### "Resource not found" error

- Verify the project ID in your config
- Ensure you have access to the project

### Editor doesn't open

- Set your `EDITOR` environment variable: `export EDITOR=vim`
- Or configure a custom editor in the config file

## License

This project is provided as-is for monitoring GitLab pipelines.
