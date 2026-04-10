# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Hackertuah is a Rust TUI application for browsing Hacker News. It uses ratatui/crossterm for the terminal UI, tokio/reqwest for async HTTP, and integrates with the Claude API for story summarization. The app features vim-style navigation, a command palette, search/filter, and section switching (Top, Ask, Show, Jobs).

## Build & Run

```bash
make build        # cargo build
make run          # cargo run
make test         # cargo test
make lint         # cargo clippy -- -D warnings
make format       # cargo fmt
make verify       # format + lint + build + test (run after any change)
make clean        # cargo clean
make install      # release build + copy binary to ~/bin
make bump         # cog bump --auto (cocogitto versioning)
```

## Architecture

The `src/` directory is split by concern:

- **`types.rs`** — Data types: `Story`, `Comment`, `FlatComment`, `Section`, `Mode`, `ClaudeRequest`, `Message`. No dependencies on other local modules.
- **`hn_api.rs`** — HTTP layer: `fetch_stories`, `fetch_comments` (HN Firebase API), `get_claude_summary` (Anthropic API). Depends on `types`.
- **`ui.rs`** — All rendering: `draw_ui`, `draw_comments`, `draw_help_bar`, `draw_menu`, `draw_summary`, `draw_command_palette`, `centered_rect`, `strip_html`. Depends on `types` and `app`.
- **`loading_screen.rs`** — `MatrixRain` struct for the Matrix-style loading animation.
- **`main.rs`** — `App` struct (all application state), `Command`/`CommandPalette`, terminal setup/teardown, and the main event loop. `App` is exposed via `pub mod app` so `ui.rs` can reference it.

The app uses a single-threaded tokio runtime. Story fetching spawns tokio tasks per section that run concurrently. The `App` struct holds all state: stories, UI mode, cached stories per section, command palette state, comments state, and search state.

## Key Details

- HN API: Firebase REST API at `hacker-news.firebaseio.com/v0/`. Fetches up to 100 stories per section.
- Claude API: Requires `CLAUDE_API_KEY` env var. Currently hardcoded to `claude-3-opus-20240229` model.
- Versioning: Uses cocogitto (`cog.toml`) with conventional commits. Changelog at `CHANGELOG.md`.
- CI: GitHub Actions runs `cargo build` and `cargo test` on push/PR to main.
- Cargo.toml lists edition 2021; version is `0.3.0`.
