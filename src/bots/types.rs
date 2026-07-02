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
