use std::collections::HashSet;

use pathfinding::prelude::{astar, bfs, bfs_bidirectional, dfs, dijkstra};

use crate::{
    bots::{ExplorationBias, MinerAlgorithm, ScoutAlgorithm},
    map::{Map, Point},
};

#[derive(Debug, Clone)]
pub struct ScoutMemory {
    visited: HashSet<Point>,
}

impl ScoutMemory {
    pub fn new(start: Point) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);
        Self { visited }
    }

    pub fn mark_visited(&mut self, point: Point) {
        self.visited.insert(point);
    }

    pub fn has_visited(&self, point: Point) -> bool {
        self.visited.contains(&point)
    }
}

pub fn scout_path(
    map: &Map,
    memory: &ScoutMemory,
    from: Point,
    base: Point,
    algorithm: ScoutAlgorithm,
    bias: ExplorationBias,
) -> Option<Vec<Point>> {
    match algorithm {
        ScoutAlgorithm::FrontierWavefront => frontier_wavefront_path(map, memory, from, base, bias),
        ScoutAlgorithm::AStarExploration => {
            let target = exploration_target(map, memory, from, base, bias)?;
            astar_path(map, from, target)
        }
        ScoutAlgorithm::RandomWalkCostBias => {
            random_cost_biased_step(map, from, base, bias).map(|step| vec![step])
        }
        ScoutAlgorithm::Bfs => bfs_path_to_unvisited(map, memory, from, base, bias),
        ScoutAlgorithm::Dfs => dfs_path_to_unvisited(map, memory, from, base, bias),
    }
}

pub fn miner_path(
    map: &Map,
    from: Point,
    goal: Point,
    algorithm: MinerAlgorithm,
) -> Option<Vec<Point>> {
    match algorithm {
        MinerAlgorithm::AStar => astar_path(map, from, goal),
        MinerAlgorithm::Dijkstra => dijkstra_path(map, from, goal),
        MinerAlgorithm::Bidirectional => bidirectional_path(map, from, goal),
    }
}

fn frontier_wavefront_path(
    map: &Map,
    memory: &ScoutMemory,
    from: Point,
    base: Point,
    bias: ExplorationBias,
) -> Option<Vec<Point>> {
    bfs(
        &from,
        |point| biased_neighbors(map, *point, base, bias),
        |point| *point != from && is_unvisited_frontier(map, memory, *point),
    )
    .map(path_without_start)
    .filter(|path| !path.is_empty())
    .or_else(|| bfs_path_to_unvisited(map, memory, from, base, bias))
}

fn bfs_path_to_unvisited(
    map: &Map,
    memory: &ScoutMemory,
    from: Point,
    base: Point,
    bias: ExplorationBias,
) -> Option<Vec<Point>> {
    bfs(
        &from,
        |point| biased_neighbors(map, *point, base, bias),
        |point| *point != from && !memory.has_visited(*point),
    )
    .map(path_without_start)
    .filter(|path| !path.is_empty())
}

fn dfs_path_to_unvisited(
    map: &Map,
    memory: &ScoutMemory,
    from: Point,
    base: Point,
    bias: ExplorationBias,
) -> Option<Vec<Point>> {
    dfs(
        from,
        |point| biased_neighbors(map, *point, base, bias),
        |point| *point != from && !memory.has_visited(*point),
    )
    .map(path_without_start)
    .filter(|path| !path.is_empty())
}

fn astar_path(map: &Map, from: Point, goal: Point) -> Option<Vec<Point>> {
    astar(
        &from,
        |point| weighted_neighbors(map, *point),
        |point| manhattan_distance(*point, goal),
        |point| *point == goal,
    )
    .map(|(path, _)| path_without_start(path))
    .filter(|path| !path.is_empty())
}

fn dijkstra_path(map: &Map, from: Point, goal: Point) -> Option<Vec<Point>> {
    dijkstra(
        &from,
        |point| weighted_neighbors(map, *point),
        |point| *point == goal,
    )
    .map(|(path, _)| path_without_start(path))
    .filter(|path| !path.is_empty())
}

fn bidirectional_path(map: &Map, from: Point, goal: Point) -> Option<Vec<Point>> {
    bfs_bidirectional(
        &from,
        &goal,
        |point| map.neighbors(*point),
        |point| map.neighbors(*point),
    )
    .map(path_without_start)
    .filter(|path| !path.is_empty())
}

fn random_cost_biased_step(
    map: &Map,
    from: Point,
    base: Point,
    bias: ExplorationBias,
) -> Option<Point> {
    let weighted: Vec<(Point, u32)> = map
        .neighbors(from)
        .into_iter()
        .filter_map(|point| {
            let cost = map.terrain_cost(point)?;
            let terrain_weight = 11_u32.saturating_sub(cost).max(1);
            Some((point, terrain_weight + bias.step_bonus(from, point, base)))
        })
        .collect();

    let total_weight: u32 = weighted.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return None;
    }

    let mut choice = rand::random_range(0..total_weight);
    for (point, weight) in weighted {
        if choice < weight {
            return Some(point);
        }
        choice -= weight;
    }

    None
}

fn exploration_target(
    map: &Map,
    memory: &ScoutMemory,
    from: Point,
    base: Point,
    bias: ExplorationBias,
) -> Option<Point> {
    map.points()
        .into_iter()
        .filter(|point| map.is_walkable(*point) && !memory.has_visited(*point))
        .min_by_key(|point| {
            let novelty_bonus = unvisited_neighbor_count(map, memory, *point) * 4;
            let distance_from_base = manhattan_distance(*point, base) / 3;
            let travel_cost =
                manhattan_distance(from, *point) + map.terrain_cost(*point).unwrap_or(10);
            travel_cost.saturating_sub(novelty_bonus)
                + distance_from_base
                + bias.penalty(*point, base)
        })
}

fn weighted_neighbors(map: &Map, point: Point) -> Vec<(Point, u32)> {
    map.neighbors(point)
        .into_iter()
        .filter_map(|neighbor| map.terrain_cost(neighbor).map(|cost| (neighbor, cost)))
        .collect()
}

fn biased_neighbors(map: &Map, point: Point, base: Point, bias: ExplorationBias) -> Vec<Point> {
    let mut neighbors = map.neighbors(point);
    neighbors.sort_by_key(|neighbor| {
        (
            bias.penalty(*neighbor, base),
            map.terrain_cost(*neighbor).unwrap_or(u32::MAX),
            manhattan_distance(point, *neighbor),
        )
    });
    neighbors
}

fn is_unvisited_frontier(map: &Map, memory: &ScoutMemory, point: Point) -> bool {
    map.is_walkable(point)
        && !memory.has_visited(point)
        && map
            .neighbors(point)
            .into_iter()
            .any(|neighbor| memory.has_visited(neighbor))
}

fn unvisited_neighbor_count(map: &Map, memory: &ScoutMemory, point: Point) -> u32 {
    map.neighbors(point)
        .into_iter()
        .filter(|neighbor| !memory.has_visited(*neighbor))
        .count() as u32
}

fn manhattan_distance(a: Point, b: Point) -> u32 {
    a.x.abs_diff(b.x) as u32 + a.y.abs_diff(b.y) as u32
}

fn path_without_start(path: Vec<Point>) -> Vec<Point> {
    path.into_iter().skip(1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{map::Terrain, point};

    fn plains_map() -> Map {
        Map::from_terrain_for_tests(5, 5, vec![Terrain::Plains; 25])
    }

    #[test]
    fn miner_astar_returns_steps_without_start() {
        let map = plains_map();
        let path = miner_path(&map, point!(0, 0), point!(2, 0), MinerAlgorithm::AStar)
            .expect("expected path");

        assert_eq!(path, vec![point!(1, 0), point!(2, 0)]);
    }

    #[test]
    fn miner_dijkstra_reaches_goal() {
        let map = plains_map();
        let path = miner_path(&map, point!(0, 0), point!(2, 2), MinerAlgorithm::Dijkstra)
            .expect("expected path");

        assert_eq!(path.last(), Some(&point!(2, 2)));
    }

    #[test]
    fn scout_bfs_finds_unvisited_neighbor() {
        let map = plains_map();
        let memory = ScoutMemory::new(point!(2, 2));
        let path = scout_path(
            &map,
            &memory,
            point!(2, 2),
            point!(0, 0),
            ScoutAlgorithm::Bfs,
            ExplorationBias::West,
        )
        .expect("expected path");

        assert_eq!(path.len(), 1);
        assert!(!memory.has_visited(path[0]));
    }

    #[test]
    fn frontier_wavefront_does_not_bounce_to_base() {
        let map = plains_map();
        let mut memory = ScoutMemory::new(point!(2, 2));
        memory.mark_visited(point!(3, 2));

        let path = scout_path(
            &map,
            &memory,
            point!(3, 2),
            point!(2, 2),
            ScoutAlgorithm::FrontierWavefront,
            ExplorationBias::West,
        )
        .expect("expected path");

        assert_ne!(path[0], point!(2, 2));
        assert!(!memory.has_visited(path[0]));
    }

    #[test]
    fn scout_bias_changes_bfs_first_step() {
        let map = plains_map();
        let memory = ScoutMemory::new(point!(2, 2));

        let west_path = scout_path(
            &map,
            &memory,
            point!(2, 2),
            point!(2, 2),
            ScoutAlgorithm::Bfs,
            ExplorationBias::West,
        )
        .expect("expected west-biased path");
        let east_path = scout_path(
            &map,
            &memory,
            point!(2, 2),
            point!(2, 2),
            ScoutAlgorithm::Bfs,
            ExplorationBias::East,
        )
        .expect("expected east-biased path");

        assert_eq!(west_path[0], point!(1, 2));
        assert_eq!(east_path[0], point!(3, 2));
    }

    #[test]
    fn paths_do_not_cross_mountains_or_deep_water() {
        let map = Map::from_terrain_for_tests(
            3,
            3,
            vec![
                Terrain::Plains,
                Terrain::Mountains,
                Terrain::Plains,
                Terrain::Plains,
                Terrain::DeepWater,
                Terrain::Plains,
                Terrain::Plains,
                Terrain::Plains,
                Terrain::Plains,
            ],
        );

        let path = miner_path(&map, point!(0, 0), point!(2, 0), MinerAlgorithm::AStar)
            .expect("expected path around barriers");

        assert!(!path.contains(&point!(1, 0)));
        assert!(!path.contains(&point!(1, 1)));
    }
}
