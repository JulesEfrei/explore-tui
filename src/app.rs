use std::error::Error;

use crate::{
    map::{Base, Map, Point, Terrain},
    point,
    state::{Action, Screen, State},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

pub struct App {
    pub state: State,
}

impl App {
    pub fn new() -> Self {
        App {
            state: State::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        ratatui::run(|terminal| {
            let mut should_render = true;

            loop {
                if should_render {
                    terminal.draw(|frame| self.render(frame))?;
                }

                let (exit, changed) = self.handle_events()?;
                if exit {
                    break Ok(());
                }

                should_render = changed
            }
        })
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        match self.state.current_screen() {
            Screen::Home => self.render_home_screen(frame),
            Screen::Game => self.render_game_screen(frame),
        }
    }

    fn handle_events(&mut self) -> std::io::Result<(bool, bool)> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok((true, false)),
                KeyCode::Char('g') => {
                    self.state.update(Action::StartGame);
                    return Ok((false, true));
                }
                KeyCode::Char('h') => {
                    self.state.update(Action::GoHome);
                    return Ok((false, true));
                }
                KeyCode::Char('r') => return Ok((false, true)),
                // handle other key events
                _ => {}
            },
            // handle other events
            _ => {}
        }
        Ok((false, false))
    }

    fn render_home_screen(&self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ])
            .flex(Flex::Center)
            .split(frame.area());

        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .flex(Flex::Center);

        let ascii_layout = row_layout.clone().split(layout[0]);
        let list_layout = row_layout.clone().split(layout[1]);
        let footer_layout = row_layout.clone().split(layout[2]);

        let slant_font = FIGlet::slant().unwrap();

        frame.render_widget(
            Paragraph::new(slant_font.convert("explore").unwrap().to_string())
                .alignment(Alignment::Center),
            ascii_layout[1],
        );

        let list_area = list_layout[1];
        let has_extra_vertical_space = list_area.height >= 9;

        let key_title_gap = "  ";
        let title_desc_gap = " - ";

        let actions = [
            ("h", "HOME", "go to home screen"),
            ("g", "GAME", "start a new game"),
        ];

        let mut action_lines = Vec::new();
        for (index, (key, title, description)) in actions.iter().enumerate() {
            action_lines.push(Line::from(vec![
                Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(key_title_gap),
                Span::styled(
                    *title,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(title_desc_gap),
                Span::raw(*description),
            ]));

            if has_extra_vertical_space && index + 1 < actions.len() {
                action_lines.push(Line::raw(""));
            }
        }

        let list_block = Block::new().padding(Padding::new(2, 2, 2, 2));
        let list_inner = list_block.inner(list_area);
        let content_height = action_lines.len() as u16;
        let top_spacer = list_inner.height.saturating_sub(content_height) / 2;

        let mut centered_lines = Vec::with_capacity(action_lines.len() + top_spacer as usize);
        for _ in 0..top_spacer {
            centered_lines.push(Line::raw(""));
        }
        centered_lines.extend(action_lines);

        frame.render_widget(Paragraph::new(centered_lines).block(list_block), list_area);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "press q to quit; ? for help;",
                    Style::default().fg(Color::Yellow),
                )),
                Line::raw(""),
                Line::raw(""),
                Line::from(Span::styled(
                    "Made by Jules B.",
                    Style::default().bold().fg(Color::Magenta),
                )),
            ])
            .block(Block::new())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
            footer_layout[1],
        );
    }

    fn render_game_screen(&self, frame: &mut ratatui::Frame) {
        let width = frame.area().width as usize;
        let height = frame.area().height as usize;
        let map = Map::new(width, height);
        let mut lines: Vec<Line> = Vec::with_capacity(height);

        for y in 0..height {
            let mut points = Vec::with_capacity(width);
            for x in 0..width {
                let terrain = map.terrain_at(point!(x, y));

                let point_char = match terrain {
                    Some(Terrain::DeepWater) => (String::from('≈'), Color::Blue),
                    Some(Terrain::ShallowWater) => (String::from('~'), Color::Cyan),
                    Some(Terrain::Plains) => (String::from('.'), Color::Green),
                    Some(Terrain::Hills) => (String::from('^'), Color::Gray),
                    Some(Terrain::Mountains) => (String::from('▲'), Color::DarkGray),
                    Some(Terrain::Base) => (String::from('B'), Color::Magenta),
                    None => (String::from('e'), Color::Red),
                };

                points.push(Span::styled(
                    point_char.0,
                    Style::default().fg(point_char.1),
                ));
            }

            lines.push(Line::from(points));
        }

        frame.render_widget(Paragraph::new(lines), frame.area());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    fn buffer_to_string(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let area = buffer.area;
        let mut out = String::new();

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }

        out
    }

    #[test]
    fn test_render_home() {
        let backend = TestBackend::new(110, 24);
        let mut terminal = Terminal::new(backend).expect("failed to create terminal");
        let app = App::new();

        terminal
            .draw(|frame| app.render_home_screen(frame))
            .expect("failed to draw home screen");

        assert_snapshot!("render_home_screen", buffer_to_string(&terminal));
    }
}
