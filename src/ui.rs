use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::types::Mode;

fn draw_help_bar(f: &mut Frame, area: Rect, shortcuts: &[(&str, &str)]) {
    let spans: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::DarkGray)),
            ];
            if i < shortcuts.len() - 1 {
                s.push(Span::styled(" ", Style::default().fg(Color::DarkGray)));
            }
            s
        })
        .collect();

    let help = Paragraph::new(Line::from(spans))
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(help, area);
}

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    if app.mode == Mode::Comments {
        draw_comments(f, app);
        return;
    }

    let help_height = 3;
    let search_height = if app.mode == Mode::Search { 3 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),             // Title bar
            Constraint::Length(3),             // Section menu
            Constraint::Min(0),                // Main content
            Constraint::Length(search_height), // Search box
            Constraint::Length(help_height),   // Help bar
        ])
        .split(f.area());

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
            let comment_str = if story.descendants > 0 {
                format!(" | {} comments", story.descendants)
            } else {
                String::new()
            };
            let content = Line::from(vec![Span::raw(format!(
                "{:2}. {} [{}] ({}){}",
                i + 1,
                story.title,
                story.score,
                story.by,
                comment_str
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

    // Help bar
    if app.mode == Mode::Search {
        draw_help_bar(
            f,
            chunks[4],
            &[("↑↓", "navigate"), ("Enter", "open"), ("Esc", "cancel")],
        );
    } else {
        draw_help_bar(
            f,
            chunks[4],
            &[
                ("j/k", "navigate"),
                ("h/l", "sections"),
                ("Enter", "open"),
                ("c", "comments"),
                ("o", "options"),
                ("/", "search"),
                ("Ctrl+K", "palette"),
                ("r", "refresh"),
                ("q", "quit"),
            ],
        );
    }

    // Overlays (drawn on top)
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

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let rest: String = chars.clone().take(3).collect();
            if rest.starts_with("p>") || rest.starts_with("p ") || rest.starts_with("br") {
                result.push('\n');
            }
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            if c == '&' {
                let entity: String = chars.clone().take_while(|&ch| ch != ';').collect();
                match entity.as_str() {
                    "amp" => {
                        result.push('&');
                        for _ in 0..=entity.len() {
                            chars.next();
                        }
                    }
                    "lt" => {
                        result.push('<');
                        for _ in 0..=entity.len() {
                            chars.next();
                        }
                    }
                    "gt" => {
                        result.push('>');
                        for _ in 0..=entity.len() {
                            chars.next();
                        }
                    }
                    "quot" => {
                        result.push('"');
                        for _ in 0..=entity.len() {
                            chars.next();
                        }
                    }
                    "apos" | "#x27" | "#39" => {
                        result.push('\'');
                        for _ in 0..=entity.len() {
                            chars.next();
                        }
                    }
                    _ => result.push('&'),
                }
            } else {
                result.push(c);
            }
        }
    }
    result
}

fn draw_comments(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(0),    // Comments
            Constraint::Length(3), // Help bar
        ])
        .split(f.area());

    // Title bar with story title
    let title = Paragraph::new(format!("Comments: {}", app.comments_story_title))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if app.comments.is_empty() {
        let empty = Paragraph::new("No comments yet.")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let visible_height = (chunks[1].height as usize).saturating_sub(2);
        let available_width = chunks[1].width.saturating_sub(2) as usize;

        // Pass 1: build all lines and track line ranges per comment
        let mut all_lines: Vec<Line> = Vec::new();
        // (start_line, end_line) for each comment index
        let mut comment_ranges: Vec<(usize, usize)> = Vec::new();

        for fc in app.comments.iter() {
            let start = all_lines.len();
            let indent = "  ".repeat(fc.depth);
            let tree_char = if fc.depth > 0 { "| " } else { "" };
            let prefix = format!("{}{}", indent, tree_char);
            let prefix_len = prefix.len();

            let text = fc
                .comment
                .text
                .as_deref()
                .map(strip_html)
                .unwrap_or_default();

            // Header line
            all_lines.push(Line::from(vec![
                Span::styled(prefix.clone(), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} :", fc.comment.by),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            // Wrapped text lines
            let wrap_width = available_width.saturating_sub(prefix_len).max(20);
            for text_line in text.lines() {
                if text_line.is_empty() {
                    all_lines.push(Line::from(Span::raw("")));
                    continue;
                }
                let mut remaining = text_line;
                while !remaining.is_empty() {
                    let end = if remaining.len() <= wrap_width {
                        remaining.len()
                    } else {
                        remaining[..wrap_width]
                            .rfind(' ')
                            .map(|p| p + 1)
                            .unwrap_or(wrap_width)
                    };
                    let (chunk, rest) = remaining.split_at(end);
                    all_lines.push(Line::from(vec![
                        Span::styled(
                            "  ".repeat(fc.depth).to_string()
                                + if fc.depth > 0 { "| " } else { "" },
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(chunk.to_string(), Style::default().fg(Color::Green)),
                    ]));
                    remaining = rest;
                }
            }

            // Blank separator
            all_lines.push(Line::from(Span::raw("")));
            comment_ranges.push((start, all_lines.len()));
        }

        // Pass 2: adjust scroll so selected comment is visible
        if let Some(&(sel_start, sel_end)) = comment_ranges.get(app.comments_selected) {
            // If selected comment starts above viewport, scroll up to it
            if sel_start < app.comments_scroll {
                app.comments_scroll = sel_start;
            }
            // If selected comment ends below viewport, scroll down
            if sel_end > app.comments_scroll + visible_height {
                // Try to show the start of the comment at top, but if the comment
                // itself is taller than the viewport, show as much as possible
                let ideal = sel_start;
                let min_to_show_end = sel_end.saturating_sub(visible_height);
                app.comments_scroll = ideal.max(min_to_show_end);
            }
        }

        // Pass 3: slice visible lines and apply selection highlight
        let visible_lines: Vec<Line> = all_lines
            .iter()
            .enumerate()
            .skip(app.comments_scroll)
            .take(visible_height)
            .map(|(line_idx, line)| {
                let is_selected = comment_ranges
                    .get(app.comments_selected)
                    .is_some_and(|&(s, e)| line_idx >= s && line_idx < e);
                if is_selected {
                    line.clone()
                        .patch_style(Style::default().bg(Color::Rgb(0, 40, 0)))
                } else {
                    line.clone()
                }
            })
            .collect();

        let comments_widget = Paragraph::new(visible_lines)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default());
        f.render_widget(comments_widget, chunks[1]);
    }

    // Help bar
    draw_help_bar(
        f,
        chunks[2],
        &[
            ("j/k", "navigate"),
            ("o", "open"),
            ("r", "reply"),
            ("R", "refresh"),
            ("Esc", "back"),
        ],
    );
}

fn draw_menu(f: &mut Frame, app: &App) {
    let overlay = Block::default().style(Style::default());
    f.render_widget(overlay, f.area());

    let area = centered_rect(15, 12, f.area());

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

fn draw_summary(f: &mut Frame, summary: &str) {
    let area = centered_rect(80, 60, f.area());

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

fn draw_command_palette(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, f.area());

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
