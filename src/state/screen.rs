#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Game,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartGame,
    GoHome,
    GoOptions,
    ToggleMinerals,
    FocusMinerals,
    ScrollMineralsUp,
    ScrollMineralsDown,
    SelectPreviousOption,
    SelectNextOption,
    DecreaseOption,
    IncreaseOption,
}
