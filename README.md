# Hacker News TUI

A terminal-based user interface for browsing Hacker News with Vim-style navigation and Claude AI integration for story summarization. Also an agent-friendly CLI: `hackertuah stories --json` gives scripts and coding agents clean JSON on stdout.

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
- **Agent-friendly CLI** — `hackertuah stories [section] [--limit N] [--json]` for scripts, pipelines, and coding agents; stable exit codes, data on stdout, progress on stderr (see [Agent / scripting usage](#agent--scripting-usage))

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

## Agent / scripting usage

Running `hackertuah` with no arguments launches the TUI. Subcommands never
touch the terminal — data goes to stdout, progress and errors to stderr:

```bash
hackertuah stories [top|ask|show|jobs] [--limit N] [--json]
```

- `--json` emits raw HN story objects (`id`, `title`, `url`, `text`, `by`,
  `score`, `descendants`, `kids`), pretty-printed.
- Exit codes: 0 success, 1 network/runtime failure, 2 usage error (also
  returned when the bare TUI is invoked with stdout piped).

```bash
hackertuah stories --limit 10 --json | jq -r '.[].title'
```

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
├── cli.rs               # Agent-friendly CLI (stories subcommand, --json)
├── types.rs             # Data types (Story, Comment, Section, Mode)
├── hn_api.rs            # Hacker News & Claude API integration
├── ui.rs                # UI rendering and layout
└── loading_screen.rs    # Matrix-style loading animation
```

## Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
ratatui = "0.30.0"
crossterm = "0.29.0"
tokio = { version = "1.51", features = ["full"] }
reqwest = { version = "0.13", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
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
