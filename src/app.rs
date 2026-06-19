use std::error::Error;

use crate::{
    map::{DefaultMap, Map, Point},
    point,
    state::{Action, Screen, State},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
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

    fn render(&mut self, frame: &mut ratatui::Frame) {
        match self.state.current_screen() {
            Screen::Home => self.render_home_screen(frame),
            Screen::Game => self.render_game_screen(frame),
            Screen::Options => self.render_options_screen(frame),
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
                //Reload helper
                KeyCode::Char('r') => {
                    self.state.update(Action::StartGame);
                    return Ok((false, true));
                }
                KeyCode::Char('m') => {
                    self.state.update(Action::ToggleMinerals);
                    return Ok((false, true));
                }
                KeyCode::Char('o') => {
                    self.state.update(Action::GoOptions);
                    return Ok((false, true));
                }
                KeyCode::Char('2') if self.state.current_screen() == Screen::Game => {
                    self.state.update(Action::FocusMinerals);
                    return Ok((false, true));
                }
                KeyCode::Up if self.state.current_screen() == Screen::Options => {
                    self.state.update(Action::SelectPreviousOption);
                    return Ok((false, true));
                }
                KeyCode::Down if self.state.current_screen() == Screen::Options => {
                    self.state.update(Action::SelectNextOption);
                    return Ok((false, true));
                }
                KeyCode::Up if self.state.current_screen() == Screen::Game => {
                    self.state.update(Action::ScrollMineralsUp);
                    return Ok((false, true));
                }
                KeyCode::Down if self.state.current_screen() == Screen::Game => {
                    self.state.update(Action::ScrollMineralsDown);
                    return Ok((false, true));
                }
                KeyCode::Left if self.state.current_screen() == Screen::Options => {
                    self.state.update(Action::DecreaseOption);
                    return Ok((false, true));
                }
                KeyCode::Right if self.state.current_screen() == Screen::Options => {
                    self.state.update(Action::IncreaseOption);
                    return Ok((false, true));
                }
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
            ("o", "OPTIONS", "set game options"),
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
                    "Made by Jules B. - Younes E. - Mathias D.",
                    Style::default().bold().fg(Color::Magenta),
                )),
            ])
            .block(Block::new())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
            footer_layout[1],
        );
    }

    fn render_options_screen(&self, frame: &mut ratatui::Frame) {
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
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ])
            .flex(Flex::Center);

        let title_layout = row_layout.clone().split(layout[0]);
        let list_layout = row_layout.clone().split(layout[1]);
        let footer_layout = row_layout.clone().split(layout[2]);

        let slant_font = FIGlet::slant().unwrap();

        frame.render_widget(
            Paragraph::new(slant_font.convert("options").unwrap().to_string())
                .alignment(Alignment::Center),
            title_layout[1],
        );

        let options = self.state.options;
        let entries = [
            ("Quantité d'énergie", options.energy_count.to_string()),
            ("Quantité de diamants", options.diamond_count.to_string()),
            ("Niveau de détail du relief", options.octaves.to_string()),
            ("Taille des reliefs", format!("{:.3}", options.frequency)),
        ];

        let mut option_lines = Vec::new();
        for (index, (label, value)) in entries.iter().enumerate() {
            let is_selected = index == self.state.selected_option;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let value = if is_selected {
                format!("‹ {value} ›")
            } else {
                format!("  {value}  ")
            };

            option_lines.push(Line::from(vec![
                Span::styled(if is_selected { "› " } else { "  " }, style),
                Span::styled(format!("{label:<28}"), style),
                Span::styled(value, style),
            ]));
            option_lines.push(Line::raw(""));
        }

        let list_block = Block::new().padding(Padding::new(2, 2, 2, 2));
        let list_inner = list_block.inner(list_layout[1]);
        let content_height = option_lines.len() as u16;
        let top_spacer = list_inner.height.saturating_sub(content_height) / 2;

        let mut centered_lines = Vec::with_capacity(option_lines.len() + top_spacer as usize);
        for _ in 0..top_spacer {
            centered_lines.push(Line::raw(""));
        }
        centered_lines.extend(option_lines);

        frame.render_widget(
            Paragraph::new(centered_lines).block(list_block),
            list_layout[1],
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "↑↓ naviguer   ←→ ajuster   g jouer   h accueil",
                Style::default().fg(Color::Yellow),
            )))
            .alignment(Alignment::Center),
            footer_layout[1],
        );
    }

    fn render_game_screen(&mut self, frame: &mut ratatui::Frame) {
        let width = frame.area().width as usize;
        let height = frame.area().height as usize;

        if self.state.map.is_none() {
            let mut map = DefaultMap::new(width, height);
            map.set_options(self.state.options);
            map.initialize();
            self.state.map = Some(map);
        }

        let map_area = if self.state.show_minerals {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Min(0), Constraint::Length(38)])
                .split(frame.area());

            self.render_minerals_dialog(frame, layout[1]);
            layout[0]
        } else {
            frame.area()
        };

        let map = self.state.map.as_ref().unwrap();
        let map_width = map.size().0;
        let map_height = map.size().1;
        let render_width = (map_area.width as usize).min(map_width);
        let render_height = (map_area.height as usize).min(map_height);

        let mut lines: Vec<Line> = Vec::with_capacity(map_area.height as usize);

        for y in 0..render_height {
            let mut points = Vec::with_capacity(render_width);
            for x in 0..render_width {
                let pos = point!(x, y);
                let tile = if let Some(mineral) = map.mineral_at(pos) {
                    map.render_tile_from_mineral(mineral.kind)
                } else if let Some(terrain) = map.terrain_at(pos) {
                    map.render_tile_from_terrain(terrain)
                } else {
                    (String::from(' '), Color::Reset)
                };

                points.push(Span::styled(tile.0, Style::default().fg(tile.1)));
            }

            lines.push(Line::from(points));
        }

        frame.render_widget(Paragraph::new(lines), map_area);
    }

    fn render_minerals_dialog(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let column_gap = "  ";

        let mut lines = Vec::new();
        let count;
        {
            let map = self.state.map.as_ref().unwrap();
            let minerals = map.minerals();
            count = minerals.len();

            if minerals.is_empty() {
                lines.push(Line::from(Span::styled(
                    "no minerals on the map",
                    Style::default().fg(Color::Yellow),
                )));
            } else {
                for (coordinates, mineral) in &minerals {
                    let (symbol, color) = map.render_tile_from_mineral(mineral.kind);

                    lines.push(Line::from(vec![
                        Span::styled(
                            symbol,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(column_gap),
                        Span::raw(format!("({:>3}, {:>3})", coordinates.x, coordinates.y)),
                        Span::raw(column_gap),
                        Span::raw(format!(
                            "mined {:>2}",
                            mineral.max_value.saturating_sub(mineral.value)
                        )),
                        Span::raw(column_gap),
                        Span::raw(format!("left {:>2}", mineral.value)),
                    ]));
                }
            }
        }

        let dialog_block = Block::new()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::Magenta))
            .title(Line::from(Span::styled(
                format!(" Minerals ({count}) "),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )))
            .padding(Padding::new(2, 1, 1, 1));

        let inner = dialog_block.inner(area);
        let visible_height = inner.height;
        let max_scroll = (lines.len() as u16).saturating_sub(visible_height);

        let scroll = if count > 0 && self.state.minerals_focus.is_some() {
            let focus = self.state.minerals_focus.unwrap().min(count - 1) as u16;
            self.state.minerals_focus = Some(focus as usize);

            let highlight = Style::default()
                .bg(Color::Magenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
            let line = &mut lines[focus as usize];
            let used: usize = line
                .spans
                .iter()
                .map(|span| span.content.chars().count())
                .sum();
            let padding = (inner.width as usize).saturating_sub(used);
            line.spans.push(Span::raw(" ".repeat(padding)));
            line.style = highlight;

            if focus < self.state.minerals_scroll {
                focus
            } else if focus >= self.state.minerals_scroll + visible_height {
                focus + 1 - visible_height
            } else {
                self.state.minerals_scroll
            }
        } else {
            self.state.minerals_scroll
        }
        .min(max_scroll);
        self.state.minerals_scroll = scroll;

        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(lines)
                .block(dialog_block)
                .scroll((scroll, 0)),
            area,
        );
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
