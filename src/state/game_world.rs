use std::sync::Arc;

use crate::map::{Map, MapOptions};
use crate::state::clock::GameClock;

pub struct GameWorld {
    pub map: Arc<Map>,
    pub clock: GameClock,
    pub resources_at_base: u32,
}

impl GameWorld {
    pub fn new(width: usize, height: usize, options: MapOptions) -> Self {
        let mut map = Map::new(width, height);
        map.set_options(options);
        map.initialize();
        Self {
            map: Arc::new(map),
            clock: GameClock::new(),
            resources_at_base: 0,
        }
    }
}
