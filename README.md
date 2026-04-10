# Hacker News TUI

A terminal-based user interface for browsing Hacker News with Vim-style navigation and Claude AI integration for story summarization.

![Hacker News TUI Demo](./demo.gif)

## Quickstart

### Homebrew (recommended)

```bash
brew tap program247365/tap
brew install hackertuah
```

### Cargo

```bash
cargo install --git https://github.com/program247365/hackertuah
```

## Features

- **Browse** top Hacker News stories in your terminal across Top, Ask, Show, and Jobs sections
- **Vim-style navigation** (j/k, h/l, arrows) throughout the app
- **Threaded comments view** — press `c` to read comments inline with indentation mirroring HN's thread structure, scroll through them with j/k, and reply directly
- **Comment counts** displayed for each story in the list
- **Claude AI integration** for story summarization via the options menu
- **Command Palette** (`Ctrl+K`) for quick access to all commands with fuzzy search
- **Instant search/filter** — press `/` to filter stories by title as you type
- **Open in browser** — open stories, comments, or reply pages directly in your default browser
- **Section switching** — navigate between Top, Ask, Show, and Jobs with `h/l` or hotkeys
- **Context-aware help bar** — keyboard shortcuts displayed at the bottom of every screen, updating per context
- **Matrix-style loading screen** while fetching data
- **Classic green-on-black** terminal aesthetic

## Keyboard Controls

### Stories (Normal Mode)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Open story in browser |
| `c` | View comments inline |
| `C` | Open comments in browser |
| `o` | Open options menu |
| `h` / `l` | Previous / next section |
| `T` / `A` / `S` / `J` | Jump to Top / Ask / Show / Jobs |
| `r` | Refresh current section |
| `R` | Refresh all sections |
| `/` | Search / filter stories |
| `Ctrl+K` | Open command palette |
| `q` / `Ctrl+C` | Quit |

### Comments View

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `o` / `Enter` | Open selected comment in browser |
| `r` | Reply to selected comment (opens HN reply page) |
| `R` | Refresh comments |
| `Esc` / `q` | Back to stories |

### Search Mode

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate filtered results |
| `Enter` | Open selected story |
| `Esc` | Cancel search |

### Command Palette

Press `Ctrl+K` to open the command palette, which provides:
- Searchable list of all available commands
- Real-time filtering as you type
- Navigate with Up/Down arrows, execute with Enter, close with Esc

### Options Menu

Press `o` to open the options menu:
1. Summarize this post (uses Claude AI)
2. Open in browser
3. Close menu

## Installation

### Prerequisites

- A Claude API key from Anthropic (for the summarization feature)

### Setup

Add your Claude API key to your environment:

```bash
export CLAUDE_API_KEY=your_key_here
```

## Project Structure

```
src/
├── main.rs              # App state, event loop, terminal setup
├── types.rs             # Data types (Story, Comment, Section, Mode)
├── hn_api.rs            # Hacker News & Claude API integration
├── ui.rs                # UI rendering and layout
└── loading_screen.rs    # Matrix-style loading animation
```

## Dependencies

```toml
[dependencies]
ratatui = "0.30.0"
crossterm = "0.29.0"
tokio = { version = "1.51", features = ["full"] }
reqwest = { version = "0.13", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
open = "5.3"
rand = "0.9"
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
