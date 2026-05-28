use std::error::Error;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::Paragraph;

use crate::state::{Action, Screen, State};

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
                terminal.draw(|frame| self.render(frame))?;
                if self.handle_events()? {
                    break Ok(());
                }
            }
        })
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        match self.state.current_screen() {
            Screen::Home => self.render_home_screen(frame),
            Screen::Game => self.render_game_screen(frame),
        }
    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('g') => self.state.update(Action::StartGame),
                KeyCode::Char('h') => self.state.update(Action::GoHome),
                // handle other key events
                _ => {}
            },
            // handle other events
            _ => {}
        }
        Ok(false)
    }

    fn render_home_screen(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(Paragraph::new(String::from("Home screen")), frame.area());
    }

    fn render_game_screen(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(Paragraph::new(String::from("Game screen")), frame.area());
    }
}
