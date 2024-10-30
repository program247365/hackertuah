use rand::{thread_rng, Rng};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Instant;

pub struct MatrixRain {
    chars: Vec<Vec<char>>,
    speeds: Vec<f32>,
    positions: Vec<f32>,
    last_update: Instant,
    blink_state: bool,
    blink_timer: Instant,
}

impl MatrixRain {
    pub fn new(width: usize) -> Self {
        let mut rng = thread_rng();
        let matrix_chars = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890".chars().collect::<Vec<char>>();
        
        MatrixRain {
            chars: (0..width)
                .map(|_| {
                    (0..20)
                        .map(|_| {
                            // Safely get a random character from the collection
                            matrix_chars[rng.gen_range(0..matrix_chars.len())]
                        })
                        .collect()
                })
                .collect(),
            speeds: (0..width).map(|_| rng.gen_range(0.1..1.0)).collect(),
            positions: (0..width).map(|_| rng.gen_range(-20.0..0.0)).collect(),
            last_update: Instant::now(),
            blink_state: true,
            blink_timer: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f32();
        self.last_update = Instant::now();

        // Update positions
        for i in 0..self.positions.len() {
            self.positions[i] += self.speeds[i] * elapsed * 10.0;
            if self.positions[i] > 20.0 {
                self.positions[i] = -20.0;
            }
        }

        // Update blink state every 500ms
        if self.blink_timer.elapsed().as_millis() > 500 {
            self.blink_state = !self.blink_state;
            self.blink_timer = Instant::now();
        }
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Draw the matrix rain
        let mut lines = Vec::new();
        for y in 0..area.height.saturating_sub(2) as usize {
            let mut line = Vec::new();
            for x in 0..self.chars.len() {
                let pos = self.positions[x] as i32;
                let char_index = (y as i32 - pos).rem_euclid(self.chars[x].len() as i32) as usize;
                let intensity = ((y as i32 - pos) as f32 * 0.5).min(1.0).max(0.0);
                let color = if intensity > 0.0 {
                    Color::Rgb(0, (intensity * 255.0) as u8, 0)
                } else {
                    Color::Black
                };
                line.push(Span::styled(
                    self.chars[x][char_index].to_string(),
                    Style::default().fg(color),
                ));
            }
            lines.push(Line::from(line));
        }

        // Draw the background and matrix rain
        let background = Paragraph::new(lines)
            .style(Style::default().bg(Color::Black));
        f.render_widget(background, area);

        // Draw the loading text in the center
        let loading_text = if self.blink_state {
            "Loading..."
        } else {
            "         "
        };
        
        let loading_block = Paragraph::new(loading_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));

        let loading_area = centered_rect(20, 10, area);
        f.render_widget(loading_block, loading_area);
    }
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