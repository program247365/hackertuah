use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use open;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::time::Duration;
use tokio; // Added for browser openin
mod loading_screen;
use loading_screen::MatrixRain;

// Hacker News API types
#[derive(Debug, Deserialize)]
struct Story {
    id: u32,
    title: String,
    url: Option<String>,
    text: Option<String>,
    by: String,
    score: i32,
    descendants: Option<i32>,
    #[serde(rename = "type")]
    story_type: String,
    time: u64,
    kids: Option<Vec<u32>>,
}

// App state
struct App {
    stories: Vec<Story>,
    selected_index: usize,
    show_menu: bool,
    menu_index: usize,
    story_content: Option<String>,
    mode: Mode,
    claude_summary: Option<String>,
    status_message: Option<(String, std::time::Instant)>, // Add this line/
    current_section: Section,                             // Add this line
    scroll_offset: usize,
    app_name: String,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Menu,
    Summary,
}

impl App {
    fn new() -> App {
        App {
            stories: Vec::new(),
            selected_index: 0,
            show_menu: false,
            menu_index: 0,
            story_content: None,
            mode: Mode::Normal,
            claude_summary: None,
            status_message: None,
            current_section: Section::Top, // Add this line
            scroll_offset: 0,              // Add this line
            app_name: "Hackertuah News".to_string(),
        }
    }

    fn next_story(&mut self) {
        if !self.stories.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.stories.len();
        }
    }

    fn previous_story(&mut self) {
        if !self.stories.is_empty() {
            self.selected_index = self
                .selected_index
                .checked_sub(1)
                .unwrap_or(self.stories.len() - 1);
        }
    }

    fn set_status_message(&mut self, message: String) {
        self.status_message = Some((message, std::time::Instant::now()));
    }

    fn open_current_story(&mut self) {
        if let Some(story) = self.stories.get(self.selected_index) {
            // First try to open the URL if it exists
            if let Some(url) = &story.url {
                match open::that(url) {
                    Ok(_) => self.set_status_message("Opened in browser".to_string()),
                    Err(_) => self.set_status_message("Failed to open URL".to_string()),
                }
            } else {
                // If no URL, open the HN discussion page
                let hn_url = format!("https://news.ycombinator.com/item?id={}", story.id);
                match open::that(&hn_url) {
                    Ok(_) => self.set_status_message("Opened discussion in browser".to_string()),
                    Err(_) => self.set_status_message("Failed to open discussion".to_string()),
                }
            }
        }
    }

    async fn refresh_stories(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Create loading animation
        let mut matrix_rain = MatrixRain::new(terminal.size()?.width as usize);

        // Clone the section before moving it into the spawned task
        let section = self.current_section;

        // Spawn the story fetching task
        let stories_future = tokio::spawn(async move { fetch_stories(section).await });

        let start_time = std::time::Instant::now();

        loop {
            terminal.draw(|f| matrix_rain.draw(f, f.size()))?;
            matrix_rain.update();

            // Check for quit
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }

            // Check if stories are ready
            if stories_future.is_finished() {
                match stories_future.await {
                    Ok(Ok(stories)) => {
                        self.stories = stories;
                        self.selected_index = 0;
                        self.set_status_message(format!("Refreshed {} stories", section.as_str()));
                        break;
                    }
                    Ok(Err(e)) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to fetch stories: {}", e),
                        )) as Box<dyn Error + Send + Sync>);
                    }
                    Err(e) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Task join error: {}", e),
                        )) as Box<dyn Error + Send + Sync>);
                    }
                }
            }

            // Check for timeout
            if start_time.elapsed() > Duration::from_secs(30) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timed out while refreshing stories",
                )) as Box<dyn Error + Send + Sync>);
            }

            tokio::time::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    fn ensure_story_visible(&mut self, height: usize) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + height {
            self.scroll_offset = self.selected_index - height + 1;
        }
    }
}

// Claude API types
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(PartialEq, Clone, Copy)]
enum Section {
    Top,
    Ask,
    Show,
    Jobs,
}

impl Section {
    fn as_str(&self) -> &str {
        match self {
            Section::Top => "Top",
            Section::Ask => "Ask",
            Section::Show => "Show",
            Section::Jobs => "Jobs",
        }
    }

    fn get_api_url(&self) -> String {
        match self {
            Section::Top => "https://hacker-news.firebaseio.com/v0/topstories.json".to_string(),
            Section::Ask => "https://hacker-news.firebaseio.com/v0/askstories.json".to_string(),
            Section::Show => "https://hacker-news.firebaseio.com/v0/showstories.json".to_string(),
            Section::Jobs => "https://hacker-news.firebaseio.com/v0/jobstories.json".to_string(),
        }
    }
}

async fn fetch_stories(section: Section) -> Result<Vec<Story>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Fetch story IDs for the selected section
    let ids: Vec<u32> = client
        .get(section.get_api_url())
        .send()
        .await?
        .json()
        .await?;

    // Fetch first 100 stories
    let mut stories = Vec::new();
    for id in ids.iter().take(100) {
        let story: Story = client
            .get(&format!(
                "https://hacker-news.firebaseio.com/v0/item/{}.json",
                id
            ))
            .send()
            .await?
            .json()
            .await?;
        stories.push(story);
    }

    Ok(stories)
}

async fn get_claude_summary(text: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let request = ClaudeRequest {
        model: "claude-3-opus-20240229".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: format!(
                "Please summarize this Hacker News post concisely:\n\n{}",
                text
            ),
        }],
        max_tokens: 150,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", std::env::var("CLAUDE_API_KEY")?)
        .json(&request)
        .send()
        .await?;

    // Parse response and extract summary
    // Note: Response parsing simplified for brevity
    Ok(response.text().await?)
}

fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(0),    // Main content
        ])
        .split(f.size());

    // Title bar
    let title = Paragraph::new(app.app_name.clone())
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Calculate visible area height
    let visible_height = (chunks[1].height as usize).saturating_sub(2); // Subtract 2 for borders

    // Ensure the selected story is visible
    app.ensure_story_visible(visible_height);

    // Create visible stories slice
    let visible_stories: Vec<ListItem> = app
        .stories
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(visible_height)
        .map(|(i, story)| {
            let content = Line::from(vec![Span::raw(format!(
                "{:2}. {} [{}] ({})",
                i + 1,
                story.title,
                story.score,
                story.by
            ))]);
            ListItem::new(content).style(Style::default().fg(Color::Green).add_modifier(
                if i == app.selected_index {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                },
            ))
        })
        .collect();

    let stories_list = List::new(visible_stories)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));

    f.render_widget(stories_list, chunks[1]);

    // Draw menu if active
    if app.show_menu {
        draw_menu(f, app);
    }

    // Draw Claude summary if available
    if let Some(summary) = &app.claude_summary {
        draw_summary(f, summary);
    }
}

fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create a full-screen clear overlay
    let overlay = Block::default().style(Style::default());
    f.render_widget(overlay, f.size());

    // Create the menu area
    let area = centered_rect(15, 12, f.size());

    let menu_items = vec!["Summarize this post...", "Open this post.....", "Close this menu"];
    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, &item)| {
            ListItem::new(item).style(Style::default().fg(Color::Green).add_modifier(
                if i == app.menu_index {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                },
            ))
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().bg(Color::Green));

    f.render_widget(menu, area);
}

fn draw_summary<B: Backend>(f: &mut Frame<B>, summary: &str) {
    let area = centered_rect(80, 60, f.size());

    let summary_widget = Paragraph::new(summary)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Claude Summary"),
        )
        .style(Style::default().fg(Color::Green).bg(Color::Reset))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(summary_widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Initial story fetch with loading screen
    if let Err(e) = app.refresh_stories(&mut terminal).await {
        app.set_status_message(format!("Failed to load stories: {}", e));
    }

    // Main event loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app.next_story(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_story(),
                    KeyCode::Enter => app.open_current_story(),
                    KeyCode::Char('r') => {
                        if let Err(e) = app.refresh_stories(&mut terminal).await {
                            app.set_status_message(format!("Refresh failed: {}", e));
                        }
                    }
                    KeyCode::Char('o') => {
                        app.show_menu = true;
                        app.mode = Mode::Menu;
                        app.menu_index = 0;
                    }
                    _ => {}
                },
                Mode::Menu => match key.code {
                    KeyCode::Esc => {
                        app.show_menu = false;
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        match app.menu_index {
                            0 => {
                                // Get Claude summary
                                if let Some(story) = app.stories.get(app.selected_index) {
                                    let text = story.text.clone().unwrap_or_default();
                                    match get_claude_summary(&text).await {
                                        Ok(summary) => {
                                            app.claude_summary = Some(summary);
                                            app.mode = Mode::Summary;
                                        }
                                        Err(e) => {
                                            app.set_status_message(format!(
                                                "Failed to get summary: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                            }
                            1 => {
                                app.open_current_story();
                                app.show_menu = false;
                                app.mode = Mode::Normal;
                            }
                            _ => {
                                app.show_menu = false;
                                app.mode = Mode::Normal;
                            }
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.menu_index = (app.menu_index + 1) % 3;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.menu_index = app.menu_index.checked_sub(1).unwrap_or(2);
                    }
                    _ => {}
                },
                Mode::Summary => match key.code {
                    KeyCode::Esc => {
                        app.claude_summary = None;
                        app.mode = Mode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
