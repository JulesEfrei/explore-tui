use crate::map::{Map, MapOptions};
use crate::state::game_world::GameWorld;
use crate::state::screen::{Action, Screen};

pub const OPTION_COUNT: usize = 4;

pub struct State {
    pub current_screen: Screen,
    pub game_world: Option<GameWorld>,
    pub map: Option<Map>,
    pub show_minerals: bool,
    pub minerals_scroll: u16,
    pub minerals_focus: Option<usize>,
    pub options: MapOptions,
    pub selected_option: usize,
}

impl State {
    pub fn new() -> Self {
        State {
            current_screen: Screen::Home,
            game_world: None,
            map: None,
            show_minerals: true,
            minerals_scroll: 0,
            minerals_focus: None,
            options: MapOptions::default(),
            selected_option: 0,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::StartGame => {
                self.current_screen = Screen::Game;
                self.game_world = None;
                self.map = None;
                self.show_minerals = true;
                self.minerals_scroll = 0;
                self.minerals_focus = None;
            }
            Action::GoHome => {
                self.current_screen = Screen::Home;
                self.game_world = None;
                self.map = None;
                self.show_minerals = false;
            }
            Action::GoOptions => {
                self.current_screen = Screen::Options;
                self.selected_option = 0;
            }
            Action::ToggleMinerals => {
                self.show_minerals = !self.show_minerals;
                if !self.show_minerals {
                    self.minerals_focus = None;
                }
            }
            Action::FocusMinerals => {
                if self.minerals_focus.is_some() {
                    self.minerals_focus = None;
                } else if self.minerals_count() > 0 {
                    self.show_minerals = true;
                    self.minerals_focus = Some(0);
                }
            }
            Action::ScrollMineralsUp => {
                if let Some(focus) = self.minerals_focus {
                    self.minerals_focus = Some(focus.saturating_sub(1));
                } else {
                    self.minerals_scroll = self.minerals_scroll.saturating_sub(1);
                }
            }
            Action::ScrollMineralsDown => {
                if let Some(focus) = self.minerals_focus {
                    self.minerals_focus = Some(focus + 1);
                } else {
                    self.minerals_scroll = self.minerals_scroll.saturating_add(1);
                }
            }
            Action::SelectPreviousOption => {
                self.selected_option = (self.selected_option + OPTION_COUNT - 1) % OPTION_COUNT;
            }
            Action::SelectNextOption => {
                self.selected_option = (self.selected_option + 1) % OPTION_COUNT;
            }
            Action::DecreaseOption => self.adjust_option(false),
            Action::IncreaseOption => self.adjust_option(true),
        }
    }

    fn adjust_option(&mut self, increase: bool) {
        match self.selected_option {
            0 => {
                self.options.energy_count = if increase {
                    (self.options.energy_count + 1).min(40)
                } else {
                    self.options.energy_count.saturating_sub(1)
                };
            }
            1 => {
                self.options.diamond_count = if increase {
                    (self.options.diamond_count + 1).min(40)
                } else {
                    self.options.diamond_count.saturating_sub(1)
                };
            }
            2 => {
                self.options.octaves = if increase {
                    (self.options.octaves + 1).min(6)
                } else {
                    self.options.octaves.saturating_sub(1).max(1)
                };
            }
            _ => {
                self.options.frequency = if increase {
                    (self.options.frequency + 0.005).min(0.05)
                } else {
                    (self.options.frequency - 0.005).max(0.005)
                };
            }
        }
    }

    fn minerals_count(&self) -> usize {
        self.map.as_ref().map_or(0, |map| map.minerals().len())
    }

    pub fn init_game_world(&mut self, width: usize, height: usize) {
        self.game_world = Some(GameWorld::new(width, height));
    }

    pub fn current_screen(&self) -> Screen {
        self.current_screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_initializes_with_home_screen() {
        let state = State::new();
        assert_eq!(state.current_screen, Screen::Home);
    }

    #[test]
    fn test_current_screen_returns_current_screen() {
        let state = State::new();
        assert_eq!(state.current_screen(), Screen::Home);
    }

    #[test]
    fn test_update_with_start_game_action() {
        let mut state = State::new();
        state.update(Action::StartGame);
        assert_eq!(state.current_screen, Screen::Game);
    }

    #[test]
    fn test_update_with_go_home_action() {
        let mut state = State::new();
        // First navigate to game screen
        state.update(Action::StartGame);
        assert_eq!(state.current_screen, Screen::Game);

        // Then go back home
        state.update(Action::GoHome);
        assert_eq!(state.current_screen, Screen::Home);
    }

    #[test]
    fn test_multiple_updates() {
        let mut state = State::new();

        // Start game
        state.update(Action::StartGame);
        assert_eq!(state.current_screen(), Screen::Game);

        // Go home
        state.update(Action::GoHome);
        assert_eq!(state.current_screen(), Screen::Home);

        // Start game again
        state.update(Action::StartGame);
        assert_eq!(state.current_screen(), Screen::Game);
    }
}
