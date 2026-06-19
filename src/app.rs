use std::error::Error;
use std::time::Duration;

use crate::{
    map::Point,
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
            loop {
                if self.handle_event()? {
                    return Ok(());
                }

                if self.state.current_screen() == Screen::Game
                    && let Some(ref mut world) = self.state.game_world
                {
                    let ticks = world.clock.advance();
                    for _ in 0..ticks {
                        self.game_tick();
                    }
                }

                terminal.draw(|frame| self.render(frame))?;
            }
        })
    }

    fn handle_event(&mut self) -> Result<bool, Box<dyn Error>> {
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('g') => self.state.update(Action::StartGame),
                    KeyCode::Char('h') => self.state.update(Action::GoHome),
                    KeyCode::Char('r') => self.state.update(Action::StartGame),
                    KeyCode::Char('m') => self.state.update(Action::ToggleMinerals),
                    KeyCode::Char('o') => self.state.update(Action::GoOptions),
                    KeyCode::Char('2') if self.state.current_screen() == Screen::Game => {
                        self.state.update(Action::FocusMinerals);
                    }
                    KeyCode::Up if self.state.current_screen() == Screen::Options => {
                        self.state.update(Action::SelectPreviousOption);
                    }
                    KeyCode::Down if self.state.current_screen() == Screen::Options => {
                        self.state.update(Action::SelectNextOption);
                    }
                    KeyCode::Up if self.state.current_screen() == Screen::Game => {
                        self.state.update(Action::ScrollMineralsUp);
                    }
                    KeyCode::Down if self.state.current_screen() == Screen::Game => {
                        self.state.update(Action::ScrollMineralsDown);
                    }
                    KeyCode::Left if self.state.current_screen() == Screen::Options => {
                        self.state.update(Action::DecreaseOption);
                    }
                    KeyCode::Right if self.state.current_screen() == Screen::Options => {
                        self.state.update(Action::IncreaseOption);
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }

    fn game_tick(&mut self) {
        // placeholder — bot logic, channel processing, etc.
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        match self.state.current_screen() {
            Screen::Home => self.render_home_screen(frame),
            Screen::Game => self.render_game_screen(frame),
            Screen::Options => self.render_options_screen(frame),
        }
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
        let area = frame.area();

        if self.state.game_world.is_none() {
            self.state
                .init_game_world(area.width as usize, area.height as usize);
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let map_area = if self.state.show_minerals {
            let side_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Min(0), Constraint::Length(38)])
                .split(layout[0]);

            self.render_minerals_dialog(frame, side_layout[1]);
            side_layout[0]
        } else {
            layout[0]
        };
        let status_area = layout[1];

        let world = self.state.game_world.as_ref().unwrap();
        let map = &*world.map;
        let map_width = map.size().0;
        let map_height = map.size().1;
        let render_width = (map_area.width as usize).min(map_width);
        let render_height = (map_area.height as usize).min(map_height);

        let mut lines: Vec<Line> = Vec::with_capacity(render_height);

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

        // Status bar
        let clock_str = world.clock.elapsed_formatted();
        let status = format!(" T + {} ", clock_str);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                status,
                Style::default().fg(Color::Gray),
            )))
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Right),
            status_area,
        );
    }

    fn render_minerals_dialog(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let column_gap = "  ";

        let mut lines = Vec::new();
        let count;
        {
            let world = self.state.game_world.as_ref().unwrap();
            let map = &*world.map;
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
