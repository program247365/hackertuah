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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
#[derive(Debug, Deserialize, Clone)]
struct Story {
    id: u32,
    title: String,
    url: Option<String>,
    text: Option<String>,
    by: String,
    score: i32,
}

// App state
struct App {
    stories: Vec<Story>,
    selected_index: usize,
    show_menu: bool,
    menu_index: usize,
    mode: Mode,
    claude_summary: Option<String>,
    status_message: Option<(String, std::time::Instant)>,
    current_section: Section,
    scroll_offset: usize,
    app_name: String,
    cached_stories: std::collections::HashMap<Section, Vec<Story>>,
    command_palette: CommandPalette,
    search_query: String,
    filtered_stories: Vec<usize>,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Menu,
    Summary,
    CommandPalette,
    Search,
}

struct Command {
    name: String,
    description: String,
    action: fn(&mut App) -> Result<(), Box<dyn Error + Send + Sync>>,
}

struct CommandPalette {
    commands: Vec<Command>,
    filtered_commands: Vec<usize>,
    search_query: String,
    selected_index: usize,
}

impl CommandPalette {
    fn new() -> Self {
        CommandPalette {
            commands: vec![
                Command {
                    name: "Open in Browser".to_string(),
                    description: "Open the selected story in your default browser".to_string(),
                    action: |_app| {
                        _app.open_current_story();
                        Ok(())
                    },
                },
                Command {
                    name: "Open Comments".to_string(),
                    description: "Open the comments for the selected story".to_string(),
                    action: |_app| {
                        _app.open_comments();
                        Ok(())
                    },
                },
                Command {
                    name: "Summarize".to_string(),
                    description: "Get an AI summary of the selected story".to_string(),
                    action: |_app| {
                        _app.show_menu = true;
                        _app.mode = Mode::Menu;
                        _app.menu_index = 0;
                        Ok(())
                    },
                },
                Command {
                    name: "Search".to_string(),
                    description: "Filter stories by text".to_string(),
                    action: |_app| {
                        _app.mode = Mode::Search;
                        _app.search_query.clear();
                        _app.filtered_stories = (0.._app.stories.len()).collect();
                        Ok(())
                    },
                },
                Command {
                    name: "Switch to Top".to_string(),
                    description: "Switch to Top stories section".to_string(),
                    action: |_app| {
                        _app.current_section = Section::Top;
                        _app.set_status_message("Switching to Top stories...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Switch to Ask".to_string(),
                    description: "Switch to Ask HN section".to_string(),
                    action: |_app| {
                        _app.current_section = Section::Ask;
                        _app.set_status_message("Switching to Ask HN...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Switch to Show".to_string(),
                    description: "Switch to Show HN section".to_string(),
                    action: |_app| {
                        _app.current_section = Section::Show;
                        _app.set_status_message("Switching to Show HN...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Switch to Jobs".to_string(),
                    description: "Switch to Jobs section".to_string(),
                    action: |_app| {
                        _app.current_section = Section::Jobs;
                        _app.set_status_message("Switching to Jobs...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Refresh".to_string(),
                    description: "Refresh the current section".to_string(),
                    action: |_app| {
                        _app.set_status_message("Refreshing...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Refresh All".to_string(),
                    description: "Refresh all sections".to_string(),
                    action: |_app| {
                        _app.set_status_message("Refreshing all sections...".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "Quit".to_string(),
                    description: "Exit the application".to_string(),
                    action: |_app| {
                        std::process::exit(0);
                    },
                },
            ],
            filtered_commands: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
        }
    }

    fn filter_commands(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_commands = (0..self.commands.len()).collect();
        } else {
            self.filtered_commands = self.commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| {
                    cmd.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    cmd.description.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected_index = 0;
    }

    fn next_command(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }

    fn previous_command(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = self.selected_index.checked_sub(1)
                .unwrap_or(self.filtered_commands.len() - 1);
        }
    }

    fn get_selected_command(&self) -> Option<&Command> {
        self.filtered_commands
            .get(self.selected_index)
            .map(|&idx| &self.commands[idx])
    }
}

impl App {
    fn new() -> App {
        App {
            stories: Vec::new(),
            selected_index: 0,
            show_menu: false,
            menu_index: 0,
            mode: Mode::Normal,
            claude_summary: None,
            status_message: None,
            current_section: Section::Top,
            scroll_offset: 0,
            app_name: "Hackertuah News".to_string(),
            cached_stories: std::collections::HashMap::new(),
            command_palette: CommandPalette::new(),
            search_query: String::new(),
            filtered_stories: Vec::new(),
        }
    }

    fn set_stories(&mut self, stories: Vec<Story>) {
        self.stories = stories;
        self.filtered_stories = (0..self.stories.len()).collect();
        self.selected_index = 0;
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

    fn open_comments(&mut self) {
        if let Some(story) = self.stories.get(self.selected_index) {
            let hn_url = format!("https://news.ycombinator.com/item?id={}", story.id);
            match open::that(&hn_url) {
                Ok(_) => self.set_status_message("Opened comments in browser".to_string()),
                Err(_) => self.set_status_message("Failed to open comments".to_string()),
            }
        }
    }

    async fn load_all_sections(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut matrix_rain = MatrixRain::new(terminal.size()?.width as usize);
        let sections = vec![Section::Top, Section::Ask, Section::Show, Section::Jobs];

        // Create futures for all sections
        let futures: Vec<_> = sections
            .into_iter()
            .map(|section| tokio::spawn(async move { (section, fetch_stories(section).await) }))
            .collect();

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

            // Check if all futures are complete
            let all_complete = futures.iter().all(|f| f.is_finished());

            if all_complete {
                for future in futures {
                    match future.await {
                        Ok((section, Ok(stories))) => {
                            self.cached_stories.insert(section, stories);
                        }
                        Ok((section, Err(e))) => {
                            self.set_status_message(format!(
                                "Failed to load {}: {}",
                                section.as_str(),
                                e
                            ));
                        }
                        Err(e) => {
                            self.set_status_message(format!("Task error: {}", e));
                        }
                    }
                }

                // Set initial stories from cache
                if let Some(stories) = self.cached_stories.get(&self.current_section) {
                    self.set_stories(stories.clone());
                }

                break;
            }

            // Check for timeout
            if start_time.elapsed() > Duration::from_secs(30) {
                return Err("Timed out while loading sections".into());
            }

            tokio::time::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    async fn refresh_stories(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // If we're just switching sections, use cached data
        if let Some(cached) = self.cached_stories.get(&self.current_section) {
            self.set_stories(cached.clone());
            self.set_status_message(format!(
                "Switched to {} stories",
                self.current_section.as_str()
            ));
            return Ok(());
        }

        // Otherwise, fetch new data (existing implementation)
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
                        self.set_stories(stories);
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

    fn filter_stories(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_stories = (0..self.stories.len()).collect();
        } else {
            self.filtered_stories = self.stories
                .iter()
                .enumerate()
                .filter(|(_, story)| {
                    story.title.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();
        }
        // Reset selection to first item if current selection is not in filtered list
        if !self.filtered_stories.contains(&self.selected_index) {
            self.selected_index = *self.filtered_stories.first().unwrap_or(&0);
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

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Length(3), // Section menu
            Constraint::Min(0),    // Main content
            Constraint::Length(if app.mode == Mode::Search { 3 } else { 0 }), // Search box
        ])
        .split(f.size());

    // Title bar
    let title = Paragraph::new(app.app_name.clone())
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Section menu
    let sections = vec!["Top", "Ask", "Show", "Jobs"];
    let section_spans: Vec<Span> = sections
        .iter()
        .map(|&section| {
            if section == app.current_section.as_str() {
                Span::styled(
                    format!(" {} ", section),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::REVERSED),
                )
            } else {
                Span::styled(format!(" {} ", section), Style::default().fg(Color::Green))
            }
        })
        .collect();

    let section_menu = Paragraph::new(Line::from(section_spans))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(section_menu, chunks[1]);

    // Stories list (main content)
    let visible_height = (chunks[2].height as usize).saturating_sub(2);

    // Ensure the selected story is visible
    app.ensure_story_visible(visible_height);

    // Create visible stories slice
    let visible_stories: Vec<ListItem> = app
        .filtered_stories
        .iter()
        .map(|&i| &app.stories[i])
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

    f.render_widget(stories_list, chunks[2]);

    // Draw search box if in search mode
    if app.mode == Mode::Search {
        let search_input = Paragraph::new(format!("/{}", app.search_query))
            .style(Style::default().fg(Color::Green))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(Style::default().fg(Color::Green)));
        f.render_widget(search_input, chunks[3]);
    }

    // Draw menu if active
    if app.show_menu {
        draw_menu(f, app);
    }

    // Draw Claude summary if available
    if let Some(summary) = &app.claude_summary {
        draw_summary(f, summary);
    }

    // Draw command palette if active
    if app.mode == Mode::CommandPalette {
        draw_command_palette(f, app);
    }
}

fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create a full-screen clear overlay
    let overlay = Block::default().style(Style::default());
    f.render_widget(overlay, f.size());

    // Create the menu area
    let area = centered_rect(15, 12, f.size());

    let menu_items = vec![
        "Summarize this post...",
        "Open this post.....",
        "Close this menu",
    ];
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

fn draw_command_palette<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = centered_rect(60, 30, f.size());
    
    // Draw the search input
    let search_input = Paragraph::new(app.command_palette.search_query.clone())
        .style(Style::default().fg(Color::Green))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Command Palette")
            .border_style(Style::default().fg(Color::Green)));
    f.render_widget(search_input, Rect::new(area.x, area.y, area.width, 3));

    // Draw the command list
    let commands_area = Rect::new(area.x, area.y + 3, area.width, area.height - 3);
    let items: Vec<ListItem> = app.command_palette.filtered_commands
        .iter()
        .map(|&idx| {
            let cmd = &app.command_palette.commands[idx];
            let content = vec![
                Line::from(vec![
                    Span::styled(cmd.name.clone(), Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled(cmd.description.clone(), Style::default().fg(Color::DarkGray)),
                ])
            ];
            ListItem::new(content)
        })
        .collect();

    let commands_list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.command_palette.selected_index));
    f.render_stateful_widget(commands_list, commands_area, &mut list_state);
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

    // Initial load of all sections
    if let Err(e) = app.load_all_sections(&mut terminal).await {
        app.set_status_message(format!("Failed to load sections: {}", e));
    }

    // Main event loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                    KeyCode::Char('k') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.mode = Mode::CommandPalette;
                        app.command_palette.search_query.clear();
                        app.command_palette.filter_commands();
                    }
                    KeyCode::Char('/') => {
                        app.mode = Mode::Search;
                        app.search_query.clear();
                        app.filtered_stories = (0..app.stories.len()).collect();
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_story(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_story(),
                    KeyCode::Char('R') => {
                        if let Err(e) = app.load_all_sections(&mut terminal).await {
                            app.set_status_message(format!(
                                "Failed to refresh all sections: {}",
                                e
                            ));
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Err(e) = app.refresh_stories(&mut terminal).await {
                            app.set_status_message(format!("Refresh failed: {}", e));
                        }
                    }
                    KeyCode::Char('T') => {
                        if app.current_section != Section::Top {
                            app.current_section = Section::Top;
                            if let Err(e) = app.refresh_stories(&mut terminal).await {
                                app.set_status_message(format!("Failed to load stories: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('A') => {
                        if app.current_section != Section::Ask {
                            app.current_section = Section::Ask;
                            if let Err(e) = app.refresh_stories(&mut terminal).await {
                                app.set_status_message(format!("Failed to load stories: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('S') => {
                        if app.current_section != Section::Show {
                            app.current_section = Section::Show;
                            if let Err(e) = app.refresh_stories(&mut terminal).await {
                                app.set_status_message(format!("Failed to load stories: {}", e));
                            }
                        }
                    }
                    KeyCode::Char('J') => {
                        if app.current_section != Section::Jobs {
                            app.current_section = Section::Jobs;
                            if let Err(e) = app.refresh_stories(&mut terminal).await {
                                app.set_status_message(format!("Failed to load stories: {}", e));
                            }
                        }
                    }
                    KeyCode::Enter => app.open_current_story(),
                    KeyCode::Char('o') => {
                        app.show_menu = true;
                        app.mode = Mode::Menu;
                        app.menu_index = 0;
                    }
                    KeyCode::Char('C') => {
                        app.open_comments();
                    }
                    KeyCode::Char('h') => {
                        app.current_section = match app.current_section {
                            Section::Top => Section::Jobs,
                            Section::Jobs => Section::Show,
                            Section::Show => Section::Ask,
                            Section::Ask => Section::Top,
                        };
                        if let Err(e) = app.refresh_stories(&mut terminal).await {
                            app.set_status_message(format!("Failed to load stories: {}", e));
                        }
                    }
                    KeyCode::Char('l') => {
                        app.current_section = match app.current_section {
                            Section::Top => Section::Ask,
                            Section::Ask => Section::Show,
                            Section::Show => Section::Jobs,
                            Section::Jobs => Section::Top,
                        };
                        if let Err(e) = app.refresh_stories(&mut terminal).await {
                            app.set_status_message(format!("Failed to load stories: {}", e));
                        }
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
                Mode::CommandPalette => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.command_palette.search_query.clear();
                    }
                    KeyCode::Char(c) => {
                        app.command_palette.search_query.push(c);
                        app.command_palette.filter_commands();
                    }
                    KeyCode::Backspace => {
                        app.command_palette.search_query.pop();
                        app.command_palette.filter_commands();
                    }
                    KeyCode::Down => app.command_palette.next_command(),
                    KeyCode::Up => app.command_palette.previous_command(),
                    KeyCode::Enter => {
                        if let Some(cmd) = app.command_palette.get_selected_command() {
                            match cmd.name.as_str() {
                                "Refresh" => {
                                    if let Err(e) = app.refresh_stories(&mut terminal).await {
                                        app.set_status_message(format!("Refresh failed: {}", e));
                                    }
                                }
                                "Refresh All" => {
                                    if let Err(e) = app.load_all_sections(&mut terminal).await {
                                        app.set_status_message(format!("Failed to refresh all sections: {}", e));
                                    }
                                }
                                "Switch to Top" | "Switch to Ask" | "Switch to Show" | "Switch to Jobs" => {
                                    if let Err(e) = (cmd.action)(&mut app) {
                                        app.set_status_message(format!("Error switching section: {}", e));
                                    }
                                    if let Err(e) = app.refresh_stories(&mut terminal).await {
                                        app.set_status_message(format!("Failed to load stories: {}", e));
                                    }
                                }
                                "Search" => {
                                    let _ = (cmd.action)(&mut app);
                                    // Command palette closes, search mode opens
                                }
                                _ => {
                                    if let Err(e) = (cmd.action)(&mut app) {
                                        app.set_status_message(format!("Error executing command: {}", e));
                                    }
                                }
                            }
                        }
                        if app.mode != Mode::Search {
                            app.mode = Mode::Normal;
                        }
                        app.command_palette.search_query.clear();
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.search_query.clear();
                        app.filtered_stories = (0..app.stories.len()).collect();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.filter_stories();
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.filter_stories();
                    }
                    KeyCode::Enter => {
                        // Open the selected (filtered) story in the browser
                        if let Some(&story_idx) = app.filtered_stories.get(app.selected_index) {
                            app.selected_index = story_idx;
                            app.open_current_story();
                        }
                        app.mode = Mode::Normal;
                        app.search_query.clear();
                        app.filtered_stories = (0..app.stories.len()).collect();
                    }
                    KeyCode::Down => {
                        if !app.filtered_stories.is_empty() {
                            app.selected_index = (app.selected_index + 1) % app.filtered_stories.len();
                        }
                    }
                    KeyCode::Up => {
                        if !app.filtered_stories.is_empty() {
                            app.selected_index = app.selected_index.checked_sub(1)
                                .unwrap_or(app.filtered_stories.len() - 1);
                        }
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
