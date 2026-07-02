use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    sync::mpsc,
};

use crate::bots::{AssignmentStrategy, BotConfig, BotEvent, BotManager, BotSnapshot};
use crate::map::{Map, MapOptions, MineralKind, Point};
use crate::state::clock::GameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownMineral {
    pub pos: Point,
    pub kind: MineralKind,
    pub remaining: u32,
    pub max_value: u32,
    pub assigned_miners: u32,
}

pub struct GameWorld {
    pub map: Arc<Map>,
    pub clock: GameClock,
    pub resources_at_base: u32,
    pub from_bots_rx: mpsc::Receiver<BotEvent>,
    pub bot_manager: BotManager,
    pub bot_snapshots: HashMap<u32, BotSnapshot>,
    pub known_minerals: Vec<KnownMineral>,
    assignment_strategy: AssignmentStrategy,
    assignment_cursor: usize,
    idle_miners: HashSet<u32>,
    miner_assignments: HashMap<u32, Point>,
}

impl GameWorld {
    pub fn new(width: usize, height: usize, options: MapOptions, bot_config: BotConfig) -> Self {
        let mut map = Map::new(width, height);
        map.set_options(options);
        map.initialize();
        let map = Arc::new(map);
        let (from_bots_tx, from_bots_rx) = mpsc::channel();
        let bot_manager = BotManager::spawn(Arc::clone(&map), from_bots_tx, bot_config);
        let idle_miners = bot_manager.miner_ids().into_iter().collect();

        Self {
            map,
            clock: GameClock::new(),
            resources_at_base: 0,
            from_bots_rx,
            bot_manager,
            bot_snapshots: HashMap::new(),
            known_minerals: Vec::new(),
            assignment_strategy: bot_config.assignment_strategy,
            assignment_cursor: 0,
            idle_miners,
            miner_assignments: HashMap::new(),
        }
    }

    pub fn record_known_mineral(&mut self, pos: Point, kind: MineralKind) {
        if !self
            .known_minerals
            .iter()
            .any(|known_mineral| known_mineral.pos == pos)
        {
            let Some(mineral) = self.map.mineral_at(pos) else {
                return;
            };

            self.known_minerals.push(KnownMineral {
                pos,
                kind,
                remaining: mineral.value,
                max_value: mineral.max_value,
                assigned_miners: 0,
            });
            self.assign_idle_miners();
        }
    }

    pub fn record_miner_arrival(&mut self, miner_id: u32, pos: Point) {
        if let Some(mineral) = self
            .known_minerals
            .iter_mut()
            .find(|known_mineral| known_mineral.pos == pos)
            && mineral.remaining > 0
        {
            mineral.remaining -= 1;
        }

        self.miner_assignments.insert(miner_id, pos);
    }

    pub fn record_resource_delivery(&mut self, miner_id: u32, pos: Point, amount: u32) {
        self.resources_at_base += amount;
        self.unassign_miner(miner_id, pos);
        self.idle_miners.insert(miner_id);
        self.assign_idle_miners();
    }

    fn assign_idle_miners(&mut self) {
        let idle_miners: Vec<u32> = self.idle_miners.iter().copied().collect();
        for miner_id in idle_miners {
            if let Some(pos) = self.choose_mineral_for_assignment()
                && self.send_mining_order(miner_id, pos)
            {
                self.idle_miners.remove(&miner_id);
            }
        }
    }

    fn choose_mineral_for_assignment(&self) -> Option<Point> {
        match self.assignment_strategy {
            AssignmentStrategy::LeastAssigned => self
                .known_minerals
                .iter()
                .filter(|mineral| mineral.remaining > mineral.assigned_miners)
                .min_by_key(|mineral| (mineral.assigned_miners, mineral.remaining))
                .map(|mineral| mineral.pos),
            AssignmentStrategy::RoundRobin => self.choose_round_robin_mineral(),
            AssignmentStrategy::WeightedByValue => self
                .known_minerals
                .iter()
                .filter(|mineral| mineral.remaining > mineral.assigned_miners)
                .max_by_key(|mineral| mineral.remaining.saturating_sub(mineral.assigned_miners))
                .map(|mineral| mineral.pos),
        }
    }

    fn choose_round_robin_mineral(&self) -> Option<Point> {
        if self.known_minerals.is_empty() {
            return None;
        }

        for offset in 0..self.known_minerals.len() {
            let index = (self.assignment_cursor + offset) % self.known_minerals.len();
            let mineral = self.known_minerals[index];
            if mineral.remaining > mineral.assigned_miners {
                return Some(mineral.pos);
            }
        }

        None
    }

    fn send_mining_order(&mut self, miner_id: u32, pos: Point) -> bool {
        let Some(mineral) = self
            .known_minerals
            .iter_mut()
            .find(|known_mineral| known_mineral.pos == pos)
        else {
            return false;
        };

        let order = crate::bots::MiningOrder {
            miner_id,
            pos,
            kind: mineral.kind,
        };

        if self.bot_manager.send_order(order) {
            mineral.assigned_miners += 1;
            self.miner_assignments.insert(miner_id, pos);
            if self.assignment_strategy == AssignmentStrategy::RoundRobin {
                self.assignment_cursor =
                    (self.assignment_cursor + 1).max(1) % self.known_minerals.len().max(1);
            }
            true
        } else {
            false
        }
    }

    fn unassign_miner(&mut self, miner_id: u32, fallback_pos: Point) {
        let pos = self
            .miner_assignments
            .remove(&miner_id)
            .unwrap_or(fallback_pos);

        if let Some(mineral) = self
            .known_minerals
            .iter_mut()
            .find(|known_mineral| known_mineral.pos == pos)
        {
            mineral.assigned_miners = mineral.assigned_miners.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bots::BotConfig,
        map::MapOptions,
        point,
    };

    fn test_world() -> GameWorld {
        GameWorld::new(
            10,
            10,
            MapOptions {
                seed: Some(42),
                energy_count: 2,
                diamond_count: 1,
                ..MapOptions::default()
            },
            BotConfig {
                scout_count: 0,
                miner_count: 0,
                ..BotConfig::default()
            },
        )
    }

    fn first_mineral_pos(world: &GameWorld) -> Point {
        world.map.minerals()[0].0
    }

    #[test]
    fn records_known_mineral() {
        let mut world = test_world();
        let pos = first_mineral_pos(&world);
        let kind = world.map.mineral_at(pos).unwrap().kind;

        world.record_known_mineral(pos, kind);

        assert_eq!(world.known_minerals.len(), 1);
        assert_eq!(world.known_minerals[0].pos, pos);
        assert_eq!(world.known_minerals[0].kind, kind);
        assert_eq!(world.known_minerals[0].assigned_miners, 0);
    }

    #[test]
    fn does_not_duplicate_known_mineral() {
        let mut world = test_world();
        let pos = first_mineral_pos(&world);
        let kind = world.map.mineral_at(pos).unwrap().kind;

        world.record_known_mineral(pos, kind);
        world.record_known_mineral(pos, kind);

        assert_eq!(world.known_minerals.len(), 1);
    }

    #[test]
    fn record_miner_arrival_decrements_remaining() {
        let mut world = test_world();
        let pos = first_mineral_pos(&world);
        let kind = world.map.mineral_at(pos).unwrap().kind;
        world.record_known_mineral(pos, kind);

        let initial = world.known_minerals[0].remaining;
        world.record_miner_arrival(0, pos);

        assert_eq!(world.known_minerals[0].remaining, initial - 1);
    }

    #[test]
    fn record_miner_arrival_does_not_go_below_zero() {
        let mut world = test_world();
        let pos = first_mineral_pos(&world);
        let kind = world.map.mineral_at(pos).unwrap().kind;
        world.record_known_mineral(pos, kind);

        for _ in 0..10 {
            world.record_miner_arrival(0, pos);
            world.record_miner_arrival(1, pos);
        }

        assert_eq!(world.known_minerals[0].remaining, 0);
    }

    #[test]
    fn record_resource_delivery_adds_to_base() {
        let mut world = test_world();
        world.record_resource_delivery(0, point!(0, 0), 5);

        assert_eq!(world.resources_at_base, 5);
    }

    #[test]
    fn record_resource_delivery_accumulates() {
        let mut world = test_world();
        world.record_resource_delivery(0, point!(0, 0), 3);
        world.record_resource_delivery(1, point!(0, 0), 7);

        assert_eq!(world.resources_at_base, 10);
    }

    #[test]
    fn resource_delivery_reinserts_miner_into_idle() {
        let mut world = test_world();
        world.record_resource_delivery(99, point!(0, 0), 1);
        assert!(world.idle_miners.contains(&99));
    }

    #[test]
    fn record_known_mineral_sets_remaining_from_map() {
        let mut world = test_world();
        let pos = first_mineral_pos(&world);
        let kind = world.map.mineral_at(pos).unwrap().kind;
        let map_value = world.map.mineral_at(pos).unwrap().value;

        world.record_known_mineral(pos, kind);

        assert_eq!(world.known_minerals[0].remaining, map_value);
        assert_eq!(world.known_minerals[0].max_value, map_value);
    }

    #[test]
    fn ignores_unknown_position() {
        let mut world = test_world();
        world.record_known_mineral(point!(99, 99), MineralKind::Energy);
        assert!(world.known_minerals.is_empty());
    }
}
