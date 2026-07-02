use crate::map::{MineralKind, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorationBias {
    West,
    East,
    North,
    South,
}

impl ExplorationBias {
    const ALL: [Self; 4] = [Self::West, Self::East, Self::North, Self::South];

    pub fn for_scout(id: u32) -> Self {
        Self::ALL[id as usize % Self::ALL.len()]
    }

    pub fn penalty(self, point: Point, base: Point) -> u32 {
        match self {
            Self::West => point.x.saturating_sub(base.x) as u32,
            Self::East => base.x.saturating_sub(point.x) as u32,
            Self::North => point.y.saturating_sub(base.y) as u32,
            Self::South => base.y.saturating_sub(point.y) as u32,
        }
    }

    pub fn step_bonus(self, from: Point, to: Point, base: Point) -> u32 {
        let from_penalty = self.penalty(from, base);
        let to_penalty = self.penalty(to, base);
        if to_penalty < from_penalty {
            4
        } else if to_penalty == from_penalty {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoutAlgorithm {
    FrontierWavefront,
    AStarExploration,
    RandomWalkCostBias,
    Bfs,
    Dfs,
}

impl ScoutAlgorithm {
    pub const ALL: [Self; 5] = [
        Self::FrontierWavefront,
        Self::AStarExploration,
        Self::RandomWalkCostBias,
        Self::Bfs,
        Self::Dfs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::FrontierWavefront => "Frontier",
            Self::AStarExploration => "A* explore",
            Self::RandomWalkCostBias => "Random cost",
            Self::Bfs => "BFS",
            Self::Dfs => "DFS",
        }
    }

    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinerAlgorithm {
    AStar,
    Dijkstra,
    Bidirectional,
}

impl MinerAlgorithm {
    pub const ALL: [Self; 3] = [Self::AStar, Self::Dijkstra, Self::Bidirectional];

    pub fn label(self) -> &'static str {
        match self {
            Self::AStar => "A*",
            Self::Dijkstra => "Dijkstra",
            Self::Bidirectional => "Bidirectional",
        }
    }

    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentStrategy {
    LeastAssigned,
    RoundRobin,
    WeightedByValue,
}

impl AssignmentStrategy {
    pub const ALL: [Self; 3] = [Self::LeastAssigned, Self::RoundRobin, Self::WeightedByValue];

    pub fn label(self) -> &'static str {
        match self {
            Self::LeastAssigned => "Least assigned",
            Self::RoundRobin => "Round robin",
            Self::WeightedByValue => "Weighted value",
        }
    }

    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotKind {
    Scout,
    Miner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotStatus {
    Idle,
    Exploring,
    Moving,
    Mining,
    ReturningToBase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningOrder {
    pub miner_id: u32,
    pub pos: Point,
    pub kind: MineralKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotSnapshot {
    pub id: u32,
    pub kind: BotKind,
    pub pos: Point,
    pub status: BotStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotEvent {
    MineralFound {
        scout_id: u32,
        pos: Point,
        kind: MineralKind,
    },
    MinerArrivedAtMineral {
        miner_id: u32,
        pos: Point,
    },
    ResourcesDelivered {
        miner_id: u32,
        pos: Point,
        amount: u32,
    },
    BotMoved(BotSnapshot),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn for_scout_cycles_through_biases() {
        let biases: Vec<_> = (0..8).map(ExplorationBias::for_scout).collect();
        assert_eq!(biases[0], ExplorationBias::West);
        assert_eq!(biases[1], ExplorationBias::East);
        assert_eq!(biases[2], ExplorationBias::North);
        assert_eq!(biases[3], ExplorationBias::South);
        assert_eq!(biases[4], ExplorationBias::West);
        assert_eq!(biases[5], ExplorationBias::East);
    }

    #[test]
    fn penalty_west_increases_with_x() {
        let base = point!(5, 5);
        assert_eq!(ExplorationBias::West.penalty(point!(7, 5), base), 2);
        assert_eq!(ExplorationBias::West.penalty(point!(3, 5), base), 0);
    }

    #[test]
    fn penalty_east_increases_when_x_is_smaller() {
        let base = point!(5, 5);
        assert_eq!(ExplorationBias::East.penalty(point!(3, 5), base), 2);
        assert_eq!(ExplorationBias::East.penalty(point!(7, 5), base), 0);
    }

    #[test]
    fn penalty_north_increases_with_y() {
        let base = point!(5, 5);
        assert_eq!(ExplorationBias::North.penalty(point!(5, 7), base), 2);
        assert_eq!(ExplorationBias::North.penalty(point!(5, 3), base), 0);
    }

    #[test]
    fn penalty_south_increases_when_y_is_smaller() {
        let base = point!(5, 5);
        assert_eq!(ExplorationBias::South.penalty(point!(5, 3), base), 2);
        assert_eq!(ExplorationBias::South.penalty(point!(5, 7), base), 0);
    }

    #[test]
    fn step_bonus_rewards_moving_toward_bias() {
        let base = point!(5, 5);
        let from = point!(6, 5);
        let toward = point!(5, 5);
        let away = point!(7, 5);
        assert_eq!(ExplorationBias::West.step_bonus(from, toward, base), 4);
        assert_eq!(ExplorationBias::West.step_bonus(from, from, base), 1);
        assert_eq!(ExplorationBias::West.step_bonus(toward, away, base), 0);
    }

    #[test]
    fn scout_algorithm_previous_next_cycles() {
        let mut a = ScoutAlgorithm::FrontierWavefront;
        assert_eq!(a.previous(), ScoutAlgorithm::Dfs);
        assert_eq!(a.next(), ScoutAlgorithm::AStarExploration);
        a = ScoutAlgorithm::Dfs;
        assert_eq!(a.next(), ScoutAlgorithm::FrontierWavefront);
    }

    #[test]
    fn scout_algorithm_labels_are_distinct() {
        let labels: std::collections::HashSet<&str> =
            ScoutAlgorithm::ALL.iter().map(|a| a.label()).collect();
        assert_eq!(labels.len(), ScoutAlgorithm::ALL.len());
    }

    #[test]
    fn miner_algorithm_previous_next_cycles() {
        let a = MinerAlgorithm::Bidirectional;
        assert_eq!(a.next(), MinerAlgorithm::AStar);
        assert_eq!(a.previous(), MinerAlgorithm::Dijkstra);
    }

    #[test]
    fn miner_algorithm_labels_are_distinct() {
        let labels: std::collections::HashSet<&str> =
            MinerAlgorithm::ALL.iter().map(|a| a.label()).collect();
        assert_eq!(labels.len(), MinerAlgorithm::ALL.len());
    }

    #[test]
    fn assignment_strategy_previous_next_cycles() {
        let s = AssignmentStrategy::WeightedByValue;
        assert_eq!(s.next(), AssignmentStrategy::LeastAssigned);
        assert_eq!(s.previous(), AssignmentStrategy::RoundRobin);
    }

    #[test]
    fn assignment_strategy_labels_are_distinct() {
        let labels: std::collections::HashSet<&str> =
            AssignmentStrategy::ALL.iter().map(|a| a.label()).collect();
        assert_eq!(labels.len(), AssignmentStrategy::ALL.len());
    }
}
