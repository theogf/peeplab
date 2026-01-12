# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Recent Changes (2026-01-12)

### Package Rename
- **Renamed from `labpeep` to `peeplab`**
- Updated all references in code, README, Cargo.toml
- Renamed error type from `LabpeepError` to `PeeplabError`
- Config directory changed from `~/.config/labpeep/` to `~/.config/peeplab/`

### Log Viewer Performance Optimization
- **Added `log_processor.rs` module** for centralized log processing
- **Implemented line caching**: Logs are processed once and cached in `App.log_processed_lines`
- **Performance improvement**: 50,000+ line logs now render instantly (no per-frame regex/ANSI parsing)
- Processing only happens:
  - Once when log is loaded (`Action::JobTraceLoaded`)
  - When timestamp mode changes (press 't')
  - Never during scrolling

### GitLab CI Log Prefix Stripping
- **Fixed prefix stripping** to handle GitLab CI control codes
- Format: `2026-01-12T10:35:38.187431Z 00O [0KMessage...`
- Strips prefixes like `00O`, `00E`, `01O` that appear after timestamps
- Removes section markers (`section_start:`, `section_end:`)
- Handles ANSI escape codes mixed with prefixes

### Search Highlighting
- **Added visual highlighting** for search matches in log viewer
- Matches highlighted with yellow background and black text
- All occurrences on visible lines are highlighted
- Search is case-insensitive ASCII search
- Use `/` to search, `n`/`N` to navigate matches

### Internal Log Viewer
- **Replaced external editor** with internal TUI log viewer for job logs
- Supports ANSI color codes (using `ansi-to-tui` library)
- Timestamp display modes: Hidden, DateOnly, Full (toggle with 't')
- Scrolling: arrow keys, PageUp/Down, Home/End
- Search functionality with visual feedback
- Centered popup (90% width/height) to prevent overflow

## Project Overview

**peeplab** is a TUI (Terminal User Interface) application for monitoring GitLab CI/CD pipelines and merge requests. It provides real-time pipeline status, job details, and the ability to view job logs directly in an internal viewer with search and highlighting capabilities.

## Build & Test Commands

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# Binary location
target/release/peeplab
```

### Testing
```bash
# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run specific test
cargo test <test_name> --lib

# Run tests for a specific module
cargo test gitlab::client::tests --lib
cargo test app::tests --lib
```

### Running
```bash
# Run the application
cargo run

# Or use the binary directly
./target/release/peeplab
```

## Architecture: The Elm Architecture (TEA)

The application follows **The Elm Architecture** pattern with strict unidirectional data flow:

```
User Input → Action → Update (State Change) → Effect → Async Operation → Action (Result)
                                    ↓
                                  Render
```

### Core Components

**1. State Management (`src/app.rs`)**
- `App` struct: Central state container
- `TrackedMergeRequest`: Per-MR state (pipelines, jobs, notes, loading status)
- `AppMode` enum: UI modes (Normal, ViewingLog, ViewingComments, SelectingMr, ShowingHelp)
- `update()` method: Pure function that takes `Action`, returns `Option<Effect>`
- Log viewer state: `log_processed_lines` (cached), `log_scroll_offset`, `timestamp_mode`, search state

**2. Actions & Effects (`src/events/actions.rs`)**
- `Action` enum: Synchronous state changes (user input + API responses)
- `Effect` enum: Asynchronous side effects to be executed
- Pattern: Actions trigger state updates which may return Effects

**3. Effect Execution (`src/main.rs`)**
- `handle_effect()`: Spawns async tasks for each effect type
- Uses `tokio::spawn` for non-blocking API calls
- Sends results back as new Actions via `mpsc` channel

**4. Event Handling (`src/events/handler.rs`)**
- `map_event_to_action()`: Maps keyboard events to Actions based on current AppMode
- Handles different key bindings per mode (Normal, ViewingComments, ShowingHelp)

**5. GitLab API Client (`src/gitlab/client.rs`)**
- Async methods for all GitLab API v4 endpoints
- Centralized error handling via `handle_response()`
- Uses `reqwest` with `PRIVATE-TOKEN` header authentication

**6. UI Rendering (`src/ui/`)**
- `layout.rs`: Main layout with 4 vertical panes
- `components/`: Individual widgets (tabs, pipeline list, job list, comments list, status bar, help)
- Uses `ratatui` for TUI rendering with crossterm backend

### Key Patterns

**State Updates (The Elm Architecture)**
```rust
// In app.rs update() method:
Action::ToggleCommentsView => {
    // 1. Check if we need to fetch data
    if !mr.notes_loaded {
        self.mode = AppMode::ViewingComments;
        self.status_message = Some("Loading...".to_string());

        // 2. Return Effect to trigger async operation
        return Some(Effect::FetchNotes { mr_index, project_id, mr_iid });
    }
    // 3. Update state synchronously if data already loaded
    self.mode = AppMode::ViewingComments;
    None
}

Action::NotesLoaded { mr_index, notes } => {
    // 4. Handle async result, update state
    mr.notes = notes;
    mr.notes_loaded = true;
    None
}
```

**Async Effect Handling**
```rust
// In main.rs handle_effect():
Effect::FetchNotes { mr_index, project_id, mr_iid } => {
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
```

**Editor Integration with Terminal Suspension**
```rust
// In editor/mod.rs:
// 1. Disable raw mode
crossterm::terminal::disable_raw_mode()?;

// 2. Exit alternate screen
crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;

// 3. Launch editor (blocking)
Command::new(&editor).arg(&temp_file).status()?;

// 4. Restore terminal state
crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
crossterm::terminal::enable_raw_mode()?;
```

## Module Organization

```
src/
├── main.rs              # Entry point, event loop, effect execution
├── app.rs               # State management, update logic
├── lib.rs               # Library exports for testing
├── error.rs             # Error types (PeeplabError, thiserror)
├── config/              # TOML config loading
├── events/
│   ├── actions.rs       # Action/Effect enums
│   └── handler.rs       # Event to Action mapping
├── gitlab/
│   ├── client.rs        # GitLab API client (async)
│   └── models.rs        # API response models (serde)
├── git.rs               # Git operations (project detection, branch)
├── editor/
│   └── mod.rs           # Editor launching with terminal suspension (legacy)
├── log_processor.rs     # Log processing: prefix stripping, timestamp formatting, ANSI parsing
└── ui/
    ├── layout.rs        # Main render function
    └── components/      # Individual UI widgets
        ├── log_viewer.rs    # Internal log viewer with search highlighting
        ├── comments_list.rs # MR comments/notes display
        ├── help.rs          # Help popup
        ├── job_list.rs      # Job table
        ├── mr_tabs.rs       # MR tabs
        └── pipeline_list.rs # Pipeline list
```

## Important Implementation Details

### Adding New Actions

1. Add variant to `Action` enum in `src/events/actions.rs`
2. Add handler in `App::update()` in `src/app.rs` (return `Option<Effect>`)
3. If async work needed, add `Effect` variant and handle in `main.rs`
4. Add key binding in `src/events/handler.rs` for appropriate `AppMode`
5. Update help text in `src/ui/components/help.rs`

### Adding New GitLab API Endpoints

1. Define response model in `src/gitlab/models.rs` with serde derives
2. Add async method to `GitLabClient` in `src/gitlab/client.rs`
3. Use `self.handle_response(response).await` for consistent error handling
4. Export model from `src/gitlab/mod.rs`
5. Add mockito-based tests in `#[cfg(test)]` module

### Adding New UI Components

1. Create module in `src/ui/components/`
2. Export from `src/ui/components/mod.rs`
3. Implement `render(f: &mut Frame, app: &App, area: Rect)` function
4. Use ratatui widgets (List, Table, Paragraph, etc.)
5. Call from `src/ui/layout.rs`

### State Management Rules

- **Never** mutate state outside `App::update()`
- **Always** use immutable borrows in render functions
- **Separate** data fetching (Effects) from state updates (Actions)
- **Cache** API responses in `TrackedMergeRequest` to minimize API calls
- **Clear cache** on refresh action

### Testing Patterns

- **Models**: Test serde deserialization with JSON fixtures
- **API Client**: Use mockito to mock HTTP responses
- **App State**: Test action handlers with helper functions (`create_test_mr()`)
- **Integration**: Manual testing in `tests/integration_test.rs`

## Configuration

Config file: `~/.config/peeplab/config.toml`

```toml
[gitlab]
token = "glpat-xxxx"                  # Required: GitLab personal access token
default_project_id = 12345            # Optional: Auto-detected from git remote
instance_url = "https://gitlab.com"   # Default: gitlab.com

[app]
refresh_interval = 30                 # Default: 30 seconds
max_tracked_mrs = 5                   # Default: 5
focus_current_branch = true           # Default: true (show only current branch MR)
```

## Git Integration

- **Auto-detection**: Parses `.git/config` to extract GitLab project from remote URL
- **Supported formats**: SSH (`git@gitlab.com:namespace/project.git`) and HTTPS
- **Branch detection**: Uses `git2` crate to get current branch
- **Fallback**: Manual `default_project_id` in config if not in git repo

## Performance Considerations

- **Log line caching**: Processed log lines stored in `App.log_processed_lines` for instant rendering
  - Processing (regex, ANSI parsing) happens once on load, not every frame
  - Dramatically improves scrolling performance for large logs (50K+ lines)
  - Reprocessed only when timestamp mode changes
- **Lazy loading**: Notes (comments) fetched only when user toggles to comments view
- **Buffered I/O**: Uses `BufWriter` with 8KB buffer for large log files
- **Non-blocking**: All API calls use `tokio::spawn` to avoid UI freezes
- **Pagination**: API calls use `per_page=100` for jobs, `per_page=20` for MRs
- **Responsive input**: Event polling at 16ms (~60fps) for smooth keyboard response
- **Search optimization**: Search operates on raw log content, highlighting done only on visible lines

## Common Pitfalls

1. **Borrow checker in update()**: Get data length first before mutating state
   ```rust
   // Wrong: borrow conflict
   if let Some(notes) = self.get_selected_notes() {
       if let Some(mr) = self.tracked_mrs.get_mut(idx) {
           mr.index = notes.len(); // Error: notes borrowed
       }
   }

   // Right: get length first
   let len = self.get_selected_notes().map(|n| n.len()).unwrap_or(0);
   if let Some(mr) = self.tracked_mrs.get_mut(idx) {
       mr.index = len;
   }
   ```

2. **Terminal restoration**: Always use `TerminalRestoreGuard` with Drop trait to ensure cleanup

3. **Mode handling**: Every `AppMode` variant must have a match arm in `map_event_to_action()`

4. **Effect spawning**: Always clone `action_tx` and `client` before moving into `tokio::spawn`
