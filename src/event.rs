use std::error::Error;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    app::App,
    state::{Action, GameFocus, Screen},
};

impl App {
    pub fn handle_event(&mut self) -> Result<bool, Box<dyn Error>> {
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if matches!(key.code, KeyCode::Char('q')) {
                    return Ok(true);
                }
                return self.handle_screen_key(key.code);
            }
        }
        Ok(false)
    }

    fn handle_screen_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match self.state.current_screen() {
            Screen::Home => self.handle_home_key(code),
            Screen::Options => self.handle_options_key(code),
            Screen::Game => self.handle_game_key(code),
        }
    }

    fn handle_home_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match code {
            KeyCode::Char('g') => self.state.update(Action::StartGame),
            KeyCode::Char('o') => self.state.update(Action::GoOptions),
            _ => {}
        }
        Ok(false)
    }

    fn handle_options_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match code {
            KeyCode::Esc | KeyCode::Backspace => self.state.update(Action::GoHome),
            KeyCode::Up | KeyCode::Char('k') => self.state.update(Action::SelectPreviousOption),
            KeyCode::Down | KeyCode::Char('j') => self.state.update(Action::SelectNextOption),
            KeyCode::Left | KeyCode::Char('h') => self.state.update(Action::DecreaseOption),
            KeyCode::Right | KeyCode::Char('l') => self.state.update(Action::IncreaseOption),
            KeyCode::Enter => self.state.update(Action::StartGame),
            _ => {}
        }
        Ok(false)
    }

    fn handle_game_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match code {
            KeyCode::Char('h') => self.state.update(Action::GoHome),
            KeyCode::Char('r') => self.state.update(Action::StartGame),
            KeyCode::Char(' ') => self.state.update(Action::TogglePause),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if self
                    .state
                    .game_world
                    .as_ref()
                    .is_some_and(|w| w.clock.is_paused())
                {
                    self.advance_paused_game(Duration::from_secs(1));
                }
            }
            _ => {
                match self.state.game_focus() {
                    GameFocus::Map => self.handle_game_map_key(code)?,
                    GameFocus::Minerals => self.handle_game_minerals_key(code)?,
                };
            }
        }
        Ok(false)
    }

    fn advance_paused_game(&mut self, duration: Duration) {
        let Some(world) = self.state.game_world.as_mut() else {
            return;
        };

        let ticks = world.clock.advance_by(duration);
        for _ in 0..ticks {
            self.game_tick();
        }
        std::thread::sleep(Duration::from_millis(20));
        self.drain_bot_events();
    }

    fn handle_game_map_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match code {
            KeyCode::Char('1') => self.state.update(Action::FocusMap),
            KeyCode::Char('2') => self.state.update(Action::FocusMinerals),
            KeyCode::Up | KeyCode::Char('k') => self.state.update(Action::ScrollMineralsUp),
            KeyCode::Down | KeyCode::Char('j') => self.state.update(Action::ScrollMineralsDown),
            _ => {}
        }
        Ok(false)
    }

    fn handle_game_minerals_key(&mut self, code: KeyCode) -> Result<bool, Box<dyn Error>> {
        match code {
            KeyCode::Char('1') => self.state.update(Action::FocusMap),
            KeyCode::Up | KeyCode::Char('k') => self.state.update(Action::ScrollMineralsUp),
            KeyCode::Down | KeyCode::Char('j') => self.state.update(Action::ScrollMineralsDown),
            _ => {}
        }
        Ok(false)
    }
}
