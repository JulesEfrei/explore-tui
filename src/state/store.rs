use crate::map::DefaultMap;
use crate::state::types::{Action, Screen};

pub struct State {
    pub current_screen: Screen,
    pub map: Option<DefaultMap>,
}

impl State {
    pub fn new() -> Self {
        State {
            current_screen: Screen::Home,
            map: None,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::StartGame => {
                self.current_screen = Screen::Game;
                self.map = None;
            }
            Action::GoHome => {
                self.current_screen = Screen::Home;
                self.map = None;
            }
        }
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
