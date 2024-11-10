# Hacker News TUI

A terminal-based user interface for browsing Hacker News with Vim-style navigation and Claude AI integration for story summarization.

![Hacker News TUI Demo](./demo.gif)

## Features

- 🚀 Browse top Hacker News stories in your terminal
- ⌨️ Vim-style keyboard navigation
- 🤖 Claude AI integration for story summarization
- 🌐 Open stories directly in your default browser
- 💚 Classic green-on-black terminal aesthetic
- 🎯 Minimalist, distraction-free interface

## Installation

### Prerequisites

- Rust and Cargo (Latest stable version)
- A Claude API key from Anthropic

### Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/hackernews-tui
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