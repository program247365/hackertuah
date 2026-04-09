use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::types::Mode;

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(if app.mode == Mode::Search { 3 } else { 0 }),
        ])
        .split(f.size());

    // Title bar
    let title = Paragraph::new(app.app_name.clone())
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Section menu
    let sections = ["Top", "Ask", "Show", "Jobs"];
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

    // Stories list
    let visible_height = (chunks[2].height as usize).saturating_sub(2);
    app.ensure_story_visible(visible_height);

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

    // Search box
    if app.mode == Mode::Search {
        let search_input = Paragraph::new(format!("/{}", app.search_query))
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(Style::default().fg(Color::Green)),
            );
        f.render_widget(search_input, chunks[3]);
    }

    if app.show_menu {
        draw_menu(f, app);
    }

    if let Some(summary) = &app.claude_summary {
        draw_summary(f, summary);
    }

    if app.mode == Mode::CommandPalette {
        draw_command_palette(f, app);
    }
}

fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
    let overlay = Block::default().style(Style::default());
    f.render_widget(overlay, f.size());

    let area = centered_rect(15, 12, f.size());

    let menu_items = [
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

    let search_input = Paragraph::new(app.command_palette.search_query.clone())
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Palette")
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(search_input, Rect::new(area.x, area.y, area.width, 3));

    let commands_area = Rect::new(area.x, area.y + 3, area.width, area.height - 3);
    let items: Vec<ListItem> = app
        .command_palette
        .filtered_commands
        .iter()
        .map(|&idx| {
            let cmd = &app.command_palette.commands[idx];
            let content = vec![Line::from(vec![
                Span::styled(cmd.name.clone(), Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(
                    cmd.description.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
            ])];
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
