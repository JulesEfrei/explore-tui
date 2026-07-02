use std::collections::HashMap;
use std::error::Error;

use crossterm::terminal;

use crate::{
    bots::{BotEvent, BotKind},
    map::Point,
    point,
    state::{Screen, State},
};

use figlet_rs::FIGlet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
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
        self.state.terminal_size = terminal::size()?;
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

    pub(crate) fn game_tick(&mut self) {
        let Some(world) = self.state.game_world.as_mut() else {
            return;
        };

        world.bot_manager.tick();
        self.drain_bot_events();
    }

    pub(crate) fn drain_bot_events(&mut self) {
        let Some(world) = self.state.game_world.as_mut() else {
            return;
        };

        while let Ok(event) = world.from_bots_rx.try_recv() {
            match event {
                BotEvent::MineralFound { pos, kind, .. } => {
                    world.record_known_mineral(pos, kind);
                }
                BotEvent::BotMoved(snapshot) => {
                    world.bot_snapshots.insert(snapshot.id, snapshot);
                }
                BotEvent::MinerArrivedAtMineral { miner_id, pos } => {
                    world.record_miner_arrival(miner_id, pos);
                }
                BotEvent::ResourcesDelivered {
                    miner_id,
                    pos,
                    amount,
                } => {
                    world.record_resource_delivery(miner_id, pos, amount);
                }
            }
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
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
            ("g", "GAME", "start a new game"),
            ("o", "OPTIONS", "set game options"),
            ("q", "QUIT", "exit the application"),
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
                    "g game  o options  q quit",
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
            (
                "Seed de génération",
                options
                    .seed
                    .map_or_else(String::new, |seed| seed.to_string()),
            ),
            (
                "Nombre de scouts",
                self.state.bot_config.scout_count.to_string(),
            ),
            (
                "Nombre de mineurs",
                self.state.bot_config.miner_count.to_string(),
            ),
            (
                "Algorithme des scouts",
                self.state.bot_config.scout_algorithm.label().to_string(),
            ),
            (
                "Algorithme des mineurs",
                self.state.bot_config.miner_algorithm.label().to_string(),
            ),
            (
                "Stratégie d'assignation",
                self.state
                    .bot_config
                    .assignment_strategy
                    .label()
                    .to_string(),
            ),
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
                "↑↓/jk naviguer  ←/h −  →/l +  seed: chiffres/⌫  enter jouer  esc accueil",
                Style::default().fg(Color::Yellow),
            )))
            .alignment(Alignment::Center),
            footer_layout[1],
        );
    }

    fn render_game_screen(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let side_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(1), Constraint::Length(38)])
            .split(layout[0]);
        let map_area = side_layout[0];
        self.render_minerals_dialog(frame, side_layout[1]);

        let status_area = layout[1];

        let world = self.state.game_world.as_ref().unwrap();
        let map = &*world.map;
        let mut bot_positions: HashMap<Point, Vec<_>> = HashMap::new();
        for snapshot in world.bot_snapshots.values() {
            bot_positions
                .entry(snapshot.pos)
                .or_default()
                .push(*snapshot);
        }
        let map_width = map.size().0;
        let map_height = map.size().1;
        let render_width = (map_area.width as usize).min(map_width);
        let render_height = (map_area.height as usize).min(map_height);

        let mut lines: Vec<Line> = Vec::with_capacity(render_height);

        for y in 0..render_height {
            let mut points = Vec::with_capacity(render_width);
            for x in 0..render_width {
                let pos = point!(x, y);
                let tile = if let Some(bots) = bot_positions.get(&pos) {
                    render_bots_tile(bots)
                } else if let Some(mineral) = map.mineral_at(pos) {
                    let (symbol, color) = map.render_tile_from_mineral(mineral.kind);
                    let is_empty = world.known_minerals.iter().any(|known_mineral| {
                        known_mineral.pos == pos && known_mineral.remaining == 0
                    });
                    (symbol, if is_empty { Color::Red } else { color })
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
        let scouts = world
            .bot_snapshots
            .values()
            .filter(|snapshot| snapshot.kind == BotKind::Scout)
            .count();
        let miners = world
            .bot_snapshots
            .values()
            .filter(|snapshot| snapshot.kind == BotKind::Miner)
            .count();
        let algorithms = format!(
            " Seed: {}   Scout: {}   Miner: {} ",
            map.seed(),
            self.state.bot_config.scout_algorithm.label(),
            self.state.bot_config.miner_algorithm.label()
        );
        let status = if world.clock.is_paused() {
            format!(
                " →/l +1s   ⏸ PAUSED   T + {}   Scouts: {}   Miners: {}   Resources: {} ",
                clock_str, scouts, miners, world.resources_at_base
            )
        } else {
            format!(
                " ▶ RUNNING   T + {}   Scouts: {}   Miners: {}   Resources: {} ",
                clock_str, scouts, miners, world.resources_at_base
            )
        };
        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length((algorithms.chars().count() as u16).min(status_area.width)),
                Constraint::Min(1),
            ])
            .split(status_area);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                algorithms,
                Style::default().fg(Color::Gray),
            )))
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Left),
            status_layout[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                status,
                Style::default().fg(Color::Gray),
            )))
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Right),
            status_layout[1],
        );
    }

    fn render_minerals_dialog(&self, frame: &mut ratatui::Frame, area: Rect) {
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
                    let known = world
                        .known_minerals
                        .iter()
                        .find(|known_mineral| known_mineral.pos == *coordinates);
                    let left = known.map_or(mineral.value, |known_mineral| known_mineral.remaining);
                    let assigned = known.map_or(0, |known_mineral| known_mineral.assigned_miners);
                    let (status, status_color) = match known {
                        Some(known_mineral) if known_mineral.remaining == 0 => {
                            ("empty", Color::Red)
                        }
                        Some(_) => ("known", Color::Green),
                        None => ("unknown", Color::Gray),
                    };
                    let symbol_color = if status == "empty" { Color::Red } else { color };

                    lines.push(Line::from(vec![
                        Span::styled(
                            symbol,
                            Style::default()
                                .fg(symbol_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(column_gap),
                        Span::raw(format!("({:>3}, {:>3})", coordinates.x, coordinates.y)),
                        Span::raw(column_gap),
                        Span::styled(status, Style::default().fg(status_color)),
                        Span::raw(column_gap),
                        Span::raw(format!("{:>2}/{}", left, mineral.max_value)),
                        Span::raw(column_gap),
                        Span::raw(format!("[{assigned}]")),
                    ]));
                }
            }
        }

        let dialog_block = Block::new()
            .borders(Borders::ALL)
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

        let scroll = if count > 0
            && let Some(focus) = self.state.game_render.minerals_focus
        {
            let focus = focus.min(count.saturating_sub(1)) as u16;

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

            if focus < self.state.game_render.minerals_scroll {
                focus
            } else if focus >= self.state.game_render.minerals_scroll + visible_height {
                focus + 1 - visible_height
            } else {
                self.state.game_render.minerals_scroll
            }
        } else {
            self.state.game_render.minerals_scroll
        }
        .min(max_scroll);

        frame.render_widget(
            Paragraph::new(lines)
                .block(dialog_block)
                .scroll((scroll, 0)),
            area,
        );
    }
}

fn render_bots_tile(bots: &[crate::bots::BotSnapshot]) -> (String, Color) {
    if bots.iter().any(|bot| bot.kind == BotKind::Miner) {
        return (String::from('M'), Color::LightRed);
    }

    (String::from('S'), Color::White)
}

