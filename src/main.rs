use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io;
mod hn_api;
mod loading_screen;
mod types;
mod ui;

pub mod app {
    pub use crate::app_impl::*;
}

mod app_impl {
    use std::error::Error;
    use std::io;
    use std::time::Duration;

    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{backend::CrosstermBackend, Terminal};

    use crate::hn_api::fetch_comments;
    use crate::hn_api::fetch_stories;
    use crate::loading_screen::MatrixRain;
    use crate::types::{FlatComment, Mode, Section, Story};

    pub struct Command {
        pub name: String,
        pub description: String,
        pub action: fn(&mut App) -> Result<(), Box<dyn Error + Send + Sync>>,
    }

    pub struct CommandPalette {
        pub commands: Vec<Command>,
        pub filtered_commands: Vec<usize>,
        pub search_query: String,
        pub selected_index: usize,
    }

    impl Default for CommandPalette {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CommandPalette {
        pub fn new() -> Self {
            CommandPalette {
                commands: vec![
                    Command {
                        name: "Open in Browser".to_string(),
                        description: "Open the selected story in your default browser".to_string(),
                        action: |app| {
                            app.open_current_story();
                            Ok(())
                        },
                    },
                    Command {
                        name: "Open Comments".to_string(),
                        description: "Open the comments for the selected story".to_string(),
                        action: |app| {
                            app.open_comments();
                            Ok(())
                        },
                    },
                    Command {
                        name: "Summarize".to_string(),
                        description: "Get an AI summary of the selected story".to_string(),
                        action: |app| {
                            app.show_menu = true;
                            app.mode = Mode::Menu;
                            app.menu_index = 0;
                            Ok(())
                        },
                    },
                    Command {
                        name: "Search".to_string(),
                        description: "Filter stories by text".to_string(),
                        action: |app| {
                            app.mode = Mode::Search;
                            app.search_query.clear();
                            app.filtered_stories = (0..app.stories.len()).collect();
                            Ok(())
                        },
                    },
                    Command {
                        name: "Switch to Top".to_string(),
                        description: "Switch to Top stories section".to_string(),
                        action: |app| {
                            app.current_section = Section::Top;
                            app.set_status_message("Switching to Top stories...".to_string());
                            Ok(())
                        },
                    },
                    Command {
                        name: "Switch to Ask".to_string(),
                        description: "Switch to Ask HN section".to_string(),
                        action: |app| {
                            app.current_section = Section::Ask;
                            app.set_status_message("Switching to Ask HN...".to_string());
                            Ok(())
                        },
                    },
                    Command {
                        name: "Switch to Show".to_string(),
                        description: "Switch to Show HN section".to_string(),
                        action: |app| {
                            app.current_section = Section::Show;
                            app.set_status_message("Switching to Show HN...".to_string());
                            Ok(())
                        },
                    },
                    Command {
                        name: "Switch to Jobs".to_string(),
                        description: "Switch to Jobs section".to_string(),
                        action: |app| {
                            app.current_section = Section::Jobs;
                            app.set_status_message("Switching to Jobs...".to_string());
                            Ok(())
                        },
                    },
                    Command {
                        name: "Refresh".to_string(),
                        description: "Refresh the current section".to_string(),
                        action: |app| {
                            app.set_status_message("Refreshing...".to_string());
                            Ok(())
                        },
                    },
                    Command {
                        name: "Refresh All".to_string(),
                        description: "Refresh all sections".to_string(),
                        action: |app| {
                            app.set_status_message("Refreshing all sections...".to_string());
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

        pub fn filter_commands(&mut self) {
            if self.search_query.is_empty() {
                self.filtered_commands = (0..self.commands.len()).collect();
            } else {
                self.filtered_commands = self
                    .commands
                    .iter()
                    .enumerate()
                    .filter(|(_, cmd)| {
                        cmd.name
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                            || cmd
                                .description
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                    })
                    .map(|(i, _)| i)
                    .collect();
            }
            self.selected_index = 0;
        }

        pub fn next_command(&mut self) {
            if !self.filtered_commands.is_empty() {
                self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
            }
        }

        pub fn previous_command(&mut self) {
            if !self.filtered_commands.is_empty() {
                self.selected_index = self
                    .selected_index
                    .checked_sub(1)
                    .unwrap_or(self.filtered_commands.len() - 1);
            }
        }

        pub fn get_selected_command(&self) -> Option<&Command> {
            self.filtered_commands
                .get(self.selected_index)
                .map(|&idx| &self.commands[idx])
        }
    }

    pub struct App {
        pub stories: Vec<Story>,
        pub selected_index: usize,
        pub show_menu: bool,
        pub menu_index: usize,
        pub mode: Mode,
        pub claude_summary: Option<String>,
        pub status_message: Option<(String, std::time::Instant)>,
        pub current_section: Section,
        pub scroll_offset: usize,
        pub app_name: String,
        pub cached_stories: std::collections::HashMap<Section, Vec<Story>>,
        pub command_palette: CommandPalette,
        pub search_query: String,
        pub filtered_stories: Vec<usize>,
        pub comments: Vec<FlatComment>,
        pub comments_selected: usize,
        pub comments_scroll: usize,
        pub comments_story_title: String,
        pub comments_story_id: u32,
    }

    impl Default for App {
        fn default() -> Self {
            Self::new()
        }
    }

    impl App {
        pub fn new() -> App {
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
                comments: Vec::new(),
                comments_selected: 0,
                comments_scroll: 0,
                comments_story_title: String::new(),
                comments_story_id: 0,
            }
        }

        pub fn set_stories(&mut self, stories: Vec<Story>) {
            self.stories = stories;
            self.filtered_stories = (0..self.stories.len()).collect();
            self.selected_index = 0;
        }

        pub fn next_story(&mut self) {
            if !self.stories.is_empty() {
                self.selected_index = (self.selected_index + 1) % self.stories.len();
            }
        }

        pub fn previous_story(&mut self) {
            if !self.stories.is_empty() {
                self.selected_index = self
                    .selected_index
                    .checked_sub(1)
                    .unwrap_or(self.stories.len() - 1);
            }
        }

        pub fn set_status_message(&mut self, message: String) {
            self.status_message = Some((message, std::time::Instant::now()));
        }

        pub fn open_current_story(&mut self) {
            if let Some(story) = self.stories.get(self.selected_index) {
                if let Some(url) = &story.url {
                    match open::that(url) {
                        Ok(_) => self.set_status_message("Opened in browser".to_string()),
                        Err(_) => self.set_status_message("Failed to open URL".to_string()),
                    }
                } else {
                    let hn_url = format!("https://news.ycombinator.com/item?id={}", story.id);
                    match open::that(&hn_url) {
                        Ok(_) => {
                            self.set_status_message("Opened discussion in browser".to_string())
                        }
                        Err(_) => self.set_status_message("Failed to open discussion".to_string()),
                    }
                }
            }
        }

        pub fn open_comments(&mut self) {
            if let Some(story) = self.stories.get(self.selected_index) {
                let hn_url = format!("https://news.ycombinator.com/item?id={}", story.id);
                match open::that(&hn_url) {
                    Ok(_) => self.set_status_message("Opened comments in browser".to_string()),
                    Err(_) => self.set_status_message("Failed to open comments".to_string()),
                }
            }
        }

        pub async fn load_all_sections(
            &mut self,
            terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            let mut matrix_rain = MatrixRain::new(terminal.size()?.width as usize);
            let sections = vec![Section::Top, Section::Ask, Section::Show, Section::Jobs];

            let futures: Vec<_> = sections
                .into_iter()
                .map(|section| tokio::spawn(async move { (section, fetch_stories(section).await) }))
                .collect();

            let start_time = std::time::Instant::now();

            loop {
                terminal.draw(|f| matrix_rain.draw(f, f.area()))?;
                matrix_rain.update();

                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                }

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

                    if let Some(stories) = self.cached_stories.get(&self.current_section) {
                        self.set_stories(stories.clone());
                    }

                    break;
                }

                if start_time.elapsed() > Duration::from_secs(30) {
                    return Err("Timed out while loading sections".into());
                }

                tokio::time::sleep(Duration::from_millis(16)).await;
            }

            Ok(())
        }

        pub async fn refresh_stories(
            &mut self,
            terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            if let Some(cached) = self.cached_stories.get(&self.current_section) {
                self.set_stories(cached.clone());
                self.set_status_message(format!(
                    "Switched to {} stories",
                    self.current_section.as_str()
                ));
                return Ok(());
            }

            let mut matrix_rain = MatrixRain::new(terminal.size()?.width as usize);
            let section = self.current_section;
            let stories_future = tokio::spawn(async move { fetch_stories(section).await });
            let start_time = std::time::Instant::now();

            loop {
                terminal.draw(|f| matrix_rain.draw(f, f.area()))?;
                matrix_rain.update();

                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                }

                if stories_future.is_finished() {
                    match stories_future.await {
                        Ok(Ok(stories)) => {
                            self.set_stories(stories);
                            self.set_status_message(format!(
                                "Refreshed {} stories",
                                section.as_str()
                            ));
                            break;
                        }
                        Ok(Err(e)) => {
                            return Err(Box::new(std::io::Error::other(format!(
                                "Failed to fetch stories: {}",
                                e
                            )))
                                as Box<dyn Error + Send + Sync>);
                        }
                        Err(e) => {
                            return Err(Box::new(std::io::Error::other(format!(
                                "Task join error: {}",
                                e
                            )))
                                as Box<dyn Error + Send + Sync>);
                        }
                    }
                }

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

        pub fn ensure_story_visible(&mut self, height: usize) {
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            } else if self.selected_index >= self.scroll_offset + height {
                self.scroll_offset = self.selected_index - height + 1;
            }
        }

        pub async fn load_comments(
            &mut self,
            terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            let story = match self.stories.get(self.selected_index) {
                Some(s) => s.clone(),
                None => return Ok(()),
            };
            self.comments_story_title = story.title.clone();
            self.comments_story_id = story.id;

            if story.kids.is_empty() {
                self.comments = Vec::new();
                self.comments_selected = 0;
                self.comments_scroll = 0;
                self.mode = Mode::Comments;
                return Ok(());
            }

            let mut matrix_rain = MatrixRain::new(terminal.size()?.width as usize);
            let story_clone = story.clone();
            let comments_future = tokio::spawn(async move { fetch_comments(&story_clone).await });
            let start_time = std::time::Instant::now();

            loop {
                terminal.draw(|f| matrix_rain.draw(f, f.area()))?;
                matrix_rain.update();

                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            return Ok(());
                        }
                    }
                }

                if comments_future.is_finished() {
                    match comments_future.await {
                        Ok(Ok(comments)) => {
                            self.comments = comments;
                            self.comments_selected = 0;
                            self.comments_scroll = 0;
                            self.mode = Mode::Comments;
                            break;
                        }
                        Ok(Err(e)) => {
                            self.set_status_message(format!("Failed to load comments: {}", e));
                            break;
                        }
                        Err(e) => {
                            self.set_status_message(format!("Task error: {}", e));
                            break;
                        }
                    }
                }

                if start_time.elapsed() > Duration::from_secs(30) {
                    self.set_status_message("Timed out loading comments".to_string());
                    break;
                }

                tokio::time::sleep(Duration::from_millis(16)).await;
            }

            Ok(())
        }

        pub fn next_comment(&mut self) {
            if !self.comments.is_empty() {
                self.comments_selected = (self.comments_selected + 1) % self.comments.len();
            }
        }

        pub fn previous_comment(&mut self) {
            if !self.comments.is_empty() {
                self.comments_selected = self
                    .comments_selected
                    .checked_sub(1)
                    .unwrap_or(self.comments.len() - 1);
            }
        }

        pub fn ensure_comment_visible(&mut self, height: usize) {
            if self.comments_selected < self.comments_scroll {
                self.comments_scroll = self.comments_selected;
            } else if self.comments_selected >= self.comments_scroll + height {
                self.comments_scroll = self.comments_selected - height + 1;
            }
        }

        pub fn filter_stories(&mut self) {
            if self.search_query.is_empty() {
                self.filtered_stories = (0..self.stories.len()).collect();
            } else {
                self.filtered_stories = self
                    .stories
                    .iter()
                    .enumerate()
                    .filter(|(_, story)| {
                        story
                            .title
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                    })
                    .map(|(i, _)| i)
                    .collect();
            }
            if !self.filtered_stories.contains(&self.selected_index) {
                self.selected_index = *self.filtered_stories.first().unwrap_or(&0);
            }
        }
    }
}

use app::App;
use hn_api::get_claude_summary;
use types::{Mode, Section};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    if let Err(e) = app.load_all_sections(&mut terminal).await {
        app.set_status_message(format!("Failed to load sections: {}", e));
    }

    loop {
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break
                    }
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
                    KeyCode::Char('c') => {
                        if let Err(e) = app.load_comments(&mut terminal).await {
                            app.set_status_message(format!("Failed to load comments: {}", e));
                        }
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
                    KeyCode::Enter => match app.menu_index {
                        0 => {
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
                    },
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.menu_index = (app.menu_index + 1) % 3;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.menu_index = app.menu_index.checked_sub(1).unwrap_or(2);
                    }
                    _ => {}
                },
                Mode::Summary => {
                    if key.code == KeyCode::Esc {
                        app.claude_summary = None;
                        app.mode = Mode::Normal;
                    }
                }
                Mode::Comments => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_comment(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_comment(),
                    KeyCode::Char('r') => {
                        if let Some(fc) = app.comments.get(app.comments_selected) {
                            let url = format!(
                                "https://news.ycombinator.com/reply?id={}&goto=item%3Fid%3D{}%23{}",
                                fc.comment.id, app.comments_story_id, fc.comment.id
                            );
                            match open::that(&url) {
                                Ok(_) => app
                                    .set_status_message("Opened reply page in browser".to_string()),
                                Err(_) => {
                                    app.set_status_message("Failed to open reply page".to_string())
                                }
                            }
                        }
                    }
                    KeyCode::Char('R') => {
                        if let Err(e) = app.load_comments(&mut terminal).await {
                            app.set_status_message(format!("Failed to refresh comments: {}", e));
                        }
                    }
                    KeyCode::Char('o') | KeyCode::Enter => {
                        if let Some(fc) = app.comments.get(app.comments_selected) {
                            let url =
                                format!("https://news.ycombinator.com/item?id={}", fc.comment.id);
                            match open::that(&url) {
                                Ok(_) => {
                                    app.set_status_message("Opened comment in browser".to_string())
                                }
                                Err(_) => {
                                    app.set_status_message("Failed to open comment".to_string())
                                }
                            }
                        }
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
                                        app.set_status_message(format!(
                                            "Failed to refresh all sections: {}",
                                            e
                                        ));
                                    }
                                }
                                "Switch to Top" | "Switch to Ask" | "Switch to Show"
                                | "Switch to Jobs" => {
                                    if let Err(e) = (cmd.action)(&mut app) {
                                        app.set_status_message(format!(
                                            "Error switching section: {}",
                                            e
                                        ));
                                    }
                                    if let Err(e) = app.refresh_stories(&mut terminal).await {
                                        app.set_status_message(format!(
                                            "Failed to load stories: {}",
                                            e
                                        ));
                                    }
                                }
                                "Search" => {
                                    let _ = (cmd.action)(&mut app);
                                }
                                _ => {
                                    if let Err(e) = (cmd.action)(&mut app) {
                                        app.set_status_message(format!(
                                            "Error executing command: {}",
                                            e
                                        ));
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
                            app.selected_index =
                                (app.selected_index + 1) % app.filtered_stories.len();
                        }
                    }
                    KeyCode::Up => {
                        if !app.filtered_stories.is_empty() {
                            app.selected_index = app
                                .selected_index
                                .checked_sub(1)
                                .unwrap_or(app.filtered_stories.len() - 1);
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
