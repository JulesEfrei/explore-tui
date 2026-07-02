use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
};

use crate::map::{Map, Point};

const BOT_TICK_WAIT_MS: u64 = 50;

#[derive(Debug, Default)]
pub(super) struct MovementProgress {
    target: Option<Point>,
    ticks_remaining: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MovementStep {
    Arrived(Point),
    Waiting,
    Idle,
}

impl MovementProgress {
    pub(super) fn advance(&mut self, map: &Map, path: &mut VecDeque<Point>) -> MovementStep {
        if self.target.is_none() {
            let Some(next) = path.pop_front() else {
                return MovementStep::Idle;
            };
            let Some(ticks) = map.terrain_cost(next) else {
                return MovementStep::Idle;
            };
            self.target = Some(next);
            self.ticks_remaining = ticks;
        }

        self.ticks_remaining = self.ticks_remaining.saturating_sub(1);
        if self.ticks_remaining == 0 {
            return MovementStep::Arrived(self.target.take().unwrap());
        }

        MovementStep::Waiting
    }
}

pub(super) fn wait_for_tick(tick_rx: &mpsc::Receiver<()>, stop_flag: &AtomicBool) -> bool {
    loop {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }

        match tick_rx.recv_timeout(Duration::from_millis(BOT_TICK_WAIT_MS)) {
            Ok(()) => return true,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{map::Terrain, point};

    #[test]
    fn movement_waits_for_terrain_cost() {
        let map = Map::from_terrain_for_tests(
            3,
            1,
            vec![Terrain::Plains, Terrain::Hills, Terrain::ShallowWater],
        );
        let mut movement = MovementProgress::default();
        let mut path = VecDeque::from(vec![point!(1, 0), point!(2, 0)]);
        let hill_cost = map.terrain_cost(point!(1, 0)).expect("hill cost");
        let shallow_water_cost = map.terrain_cost(point!(2, 0)).expect("shallow water cost");

        for _ in 1..hill_cost {
            assert_eq!(movement.advance(&map, &mut path), MovementStep::Waiting);
        }
        assert_eq!(
            movement.advance(&map, &mut path),
            MovementStep::Arrived(point!(1, 0))
        );

        for _ in 1..shallow_water_cost {
            assert_eq!(movement.advance(&map, &mut path), MovementStep::Waiting);
        }
        assert_eq!(
            movement.advance(&map, &mut path),
            MovementStep::Arrived(point!(2, 0))
        );
    }

    #[test]
    fn movement_distinguishes_waiting_from_idle() {
        let map = Map::from_terrain_for_tests(2, 1, vec![Terrain::Plains, Terrain::Hills]);
        let mut movement = MovementProgress::default();
        let mut path = VecDeque::from(vec![point!(1, 0)]);

        assert_eq!(movement.advance(&map, &mut path), MovementStep::Waiting);
        assert_eq!(
            movement.advance(&map, &mut path),
            MovementStep::Arrived(point!(1, 0))
        );
        assert_eq!(movement.advance(&map, &mut path), MovementStep::Idle);
    }
}
