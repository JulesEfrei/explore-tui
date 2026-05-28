#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartGame,
    GoHome,
}
