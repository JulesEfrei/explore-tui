#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Game,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameFocus {
    Map,
    Minerals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartGame,
    GoHome,
    GoOptions,
    ToggleMinerals,
    FocusMinerals,
    FocusMap,
    ScrollMineralsUp,
    ScrollMineralsDown,
    SelectPreviousOption,
    SelectNextOption,
    DecreaseOption,
    IncreaseOption,
    TogglePause,
    AdvanceClock,
}
