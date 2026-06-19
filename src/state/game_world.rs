use std::sync::Arc;

use crate::map::Map;
use crate::state::clock::GameClock;

pub struct GameWorld {
    pub map: Arc<Map>,
    pub clock: GameClock,
    pub resources_at_base: u32,
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        let mut map = Map::new(width, height);
        map.initialize();
        Self {
            map: Arc::new(map),
            clock: GameClock::new(),
            resources_at_base: 0,
        }
    }
}
