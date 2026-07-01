use crate::bots::BotConfig;
use crate::map::MapOptions;
use crate::state::game_world::GameWorld;
use crate::state::screen::{Action, GameFocus, Screen};

pub const OPTION_COUNT: usize = 9;

#[derive(Debug, Clone, Copy)]
pub struct GameRenderState {
    pub minerals_scroll: u16,
    pub minerals_focus: Option<usize>,
}

impl GameRenderState {
    pub fn new() -> Self {
        GameRenderState {
            minerals_scroll: 0,
            minerals_focus: None,
        }
    }
}

pub struct State {
    pub current_screen: Screen,
    pub game_focus: GameFocus,
    pub game_world: Option<GameWorld>,
    pub game_render: GameRenderState,
    pub options: MapOptions,
    pub bot_config: BotConfig,
    pub selected_option: usize,
    pub terminal_size: (u16, u16),
}

impl State {
    pub fn new() -> Self {
        State {
            current_screen: Screen::Home,
            game_focus: GameFocus::Map,
            game_world: None,
            game_render: GameRenderState::new(),
            options: MapOptions::default(),
            bot_config: BotConfig::default(),
            selected_option: 0,
            terminal_size: (0, 0),
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::StartGame => {
                self.current_screen = Screen::Game;
                self.game_focus = GameFocus::Map;
                self.game_render = GameRenderState::new();
                self.init_game_world(self.terminal_size.0 as usize, self.terminal_size.1 as usize);
            }
            Action::TogglePause => {
                if let Some(ref mut world) = self.game_world {
                    world.clock.toggle_pause();
                }
            }
            Action::GoHome => {
                self.current_screen = Screen::Home;
                self.game_focus = GameFocus::Map;
                self.game_world = None;
            }
            Action::GoOptions => {
                self.current_screen = Screen::Options;
                self.game_focus = GameFocus::Map;
                self.selected_option = 0;
            }
            Action::FocusMinerals => {
                self.game_focus = GameFocus::Minerals;
                if self.game_render.minerals_focus.is_none() && self.minerals_count() > 0 {
                    self.game_render.minerals_focus = Some(0);
                }
            }
            Action::FocusMap => {
                self.game_focus = GameFocus::Map;
                self.game_render.minerals_focus = None;
            }
            Action::ScrollMineralsUp => {
                if let Some(focus) = self.game_render.minerals_focus {
                    self.game_render.minerals_focus = Some(focus.saturating_sub(1));
                } else {
                    self.game_render.minerals_scroll =
                        self.game_render.minerals_scroll.saturating_sub(1);
                }
            }
            Action::ScrollMineralsDown => {
                if let Some(focus) = self.game_render.minerals_focus {
                    self.game_render.minerals_focus = Some(focus + 1);
                } else {
                    self.game_render.minerals_scroll =
                        self.game_render.minerals_scroll.saturating_add(1);
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
            3 => {
                self.options.frequency = if increase {
                    (self.options.frequency + 0.005).min(0.05)
                } else {
                    (self.options.frequency - 0.005).max(0.005)
                };
            }
            4 => {
                self.bot_config.scout_count = if increase {
                    (self.bot_config.scout_count + 1).min(8)
                } else {
                    self.bot_config.scout_count.saturating_sub(1).max(1)
                };
            }
            5 => {
                self.bot_config.miner_count = if increase {
                    (self.bot_config.miner_count + 1).min(8)
                } else {
                    self.bot_config.miner_count.saturating_sub(1).max(1)
                };
            }
            6 => {
                self.bot_config.scout_algorithm = if increase {
                    self.bot_config.scout_algorithm.next()
                } else {
                    self.bot_config.scout_algorithm.previous()
                };
            }
            7 => {
                self.bot_config.miner_algorithm = if increase {
                    self.bot_config.miner_algorithm.next()
                } else {
                    self.bot_config.miner_algorithm.previous()
                };
            }
            _ => {
                self.bot_config.assignment_strategy = if increase {
                    self.bot_config.assignment_strategy.next()
                } else {
                    self.bot_config.assignment_strategy.previous()
                };
            }
        }
    }

    fn minerals_count(&self) -> usize {
        self.game_world
            .as_ref()
            .map_or(0, |world| world.map.minerals().len())
    }

    pub fn init_game_world(&mut self, width: usize, height: usize) {
        self.game_world = Some(GameWorld::new(width, height, self.options, self.bot_config));
    }

    pub fn current_screen(&self) -> Screen {
        self.current_screen
    }

    pub fn game_focus(&self) -> GameFocus {
        self.game_focus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> State {
        let mut state = State::new();
        state.terminal_size = (80, 24);
        state
    }

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
        let mut state = make_state();
        state.update(Action::StartGame);
        assert_eq!(state.current_screen, Screen::Game);
    }

    #[test]
    fn test_update_with_go_home_action() {
        let mut state = make_state();
        state.update(Action::StartGame);
        assert_eq!(state.current_screen, Screen::Game);
        state.update(Action::GoHome);
        assert_eq!(state.current_screen, Screen::Home);
    }

    #[test]
    fn test_multiple_updates() {
        let mut state = make_state();
        state.update(Action::StartGame);
        assert_eq!(state.current_screen(), Screen::Game);
        state.update(Action::GoHome);
        assert_eq!(state.current_screen(), Screen::Home);
        state.update(Action::StartGame);
        assert_eq!(state.current_screen(), Screen::Game);
    }

    #[test]
    fn test_adjusts_bot_counts_from_options() {
        let mut state = make_state();
        state.update(Action::GoOptions);

        state.selected_option = 4;
        state.update(Action::IncreaseOption);
        assert_eq!(state.bot_config.scout_count, 4);
        state.update(Action::DecreaseOption);
        assert_eq!(state.bot_config.scout_count, 3);

        state.selected_option = 5;
        state.update(Action::IncreaseOption);
        assert_eq!(state.bot_config.miner_count, 3);
        state.update(Action::DecreaseOption);
        assert_eq!(state.bot_config.miner_count, 2);
    }
}
