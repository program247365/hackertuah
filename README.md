# Hacker News TUI

A terminal-based user interface for browsing Hacker News with Vim-style navigation and Claude AI integration for story summarization.

![Hacker News TUI Demo](./demo.gif)

## Quickstart

Install and run immediately with Cargo:

```bash
git clone https://github.com/program247365/hackertuah.git
cd hackertuah
cargo run --release
```

Or install globally from the latest source:

```bash
cargo install --git https://github.com/program247365/hackertuah
```

## Features

- 🚀 Browse top Hacker News stories in your terminal
- ⌨️ Vim-style keyboard navigation
- 🤖 Claude AI integration for story summarization
- 🌐 Open stories directly in your default browser
- 💚 Classic green-on-black terminal aesthetic
- 🎯 Minimalist, distraction-free interface

## Installation

### Cargo

```bash
cargo install --git https://github.com/program247365/hackertuah
```

### Prerequisites

- Rust and Cargo (Latest stable version)
- A Claude API key from Anthropic

### Setup

1. Clone the repository:
```bash
git clone https://github.com/program247365/hackertuah
cd hackernews-tui
```

2. Add your Claude API key to your environment:
```bash
export CLAUDE_API_KEY=your_key_here
```

3. Build and run:
```bash
cargo build --release
cargo run
```

## Usage

### Keyboard Controls

- `j` or `↓`: Move down
- `k` or `↑`: Move up
- `Enter`: Open selected story in default browser
- `C`: Open comments for selected story
- `o`: Open options menu
- `q`: Quit application
- `Esc`: Close menus/summaries
- `T`: Switch to Top stories
- `A`: Switch to Ask HN
- `S`: Switch to Show HN
- `J`: Switch to Jobs
- `h`/`l`: Navigate between sections
- `r`: Refresh current section
- `R`: Refresh all sections
- `Ctrl+K`: Open command palette (search and execute commands)
- `/`: Start search (type to filter stories)

### Command Palette

Press `Ctrl+K` to open the command palette, which provides:
- Searchable list of all available commands
- Real-time filtering as you type
- Command descriptions
- Execute commands with Enter
- Navigate with Up/Down arrows
- Close with Esc

Available commands now include:
- Open in Browser
- Open Comments
- Summarize
- Search (activate search/filter mode)
- Switch to Top/Ask/Show/Jobs
- Refresh/Refresh All
- Quit

### Options Menu

Press `o` to open the options menu, which provides:
1. Summarize this post (uses Claude AI)
2. Open in browser
3. Close menu

### Story Information

Each story displays:
- Title
- Score
- Author
- Direct link to article or discussion

## Dependencies

```toml
[dependencies]
ratatui = "0.21.0"
crossterm = "0.26.0"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
open = "3.2"
```

## Project Structure

```
src/
├── main.rs          # Main application logic
├── types.rs         # Data structures and type definitions
├── ui.rs            # UI rendering and layout
└── hn_api.rs        # Hacker News API integration
```

## Features in Detail

### Hacker News Integration
- Fetches top 30 stories from Hacker News API
- Real-time score and comment updates
- Direct access to article URLs and discussion pages

### Claude AI Integration
- Summarizes long articles and discussions
- Provides concise, intelligent summaries of complex topics
- Accessible through the options menu with `o`

### Terminal UI
- Built with ratatui for smooth rendering
- Classic green-on-black color scheme
- Efficient memory usage and fast rendering
- Responsive layout that adapts to terminal size

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Search

Press `/` to start searching:
- Type to filter stories by title
- Results update in real-time
- Use Up/Down arrows to navigate filtered results
- Press Enter to select or Esc to cancel
- Search works across all sections (Top, Ask, Show, Jobs)

You can also activate search from the command palette (Ctrl+K → type 'search').
