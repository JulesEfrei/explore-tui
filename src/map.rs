use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    DeepWater,
    ShallowWater,
    Plains,
    Hills,
    Mountains,
    Base,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineralKind {
    Energy,
    Diamond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mineral {
    pub kind: MineralKind,
    pub value: u32,
    pub max_value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y }
    };

    (x: $x:expr, y: $y:expr) => {
        Point { x: $x, y: $y }
    };
}

#[derive(Debug, Clone)]
pub struct Base {
    pub coordinates: (Point, Point, Point, Point),
}

/// Interface for any elevatoin generation strategy.
///
/// Implement this trait for each technique (Perlin, Simplex, Worley, etc.)
pub trait ElevationMapGenerator {
    /// Returns an elevation value for one tile.
    fn elevation_at(&self, coordinates: Point, width: usize, height: usize) -> f64;
}

#[derive(Debug, Clone)]
pub struct PerlinGenerator {
    perlin: Fbm<Perlin>,
}

#[derive(Debug, Clone, Copy)]
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u32) -> Self {
        Self { state: seed as u64 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
        let len = range.end.saturating_sub(range.start);
        range.start + (self.next_u32() as usize % len)
    }

    fn range_u32_inclusive(&mut self, range: std::ops::RangeInclusive<u32>) -> u32 {
        let start = *range.start();
        let len = range.end() - start + 1;
        start + (self.next_u32() % len)
    }
}

impl PerlinGenerator {
    pub fn new(seed: u32, octaves: usize, frequency: f64) -> Self {
        let noise = Fbm::<Perlin>::new(seed)
            .set_octaves(octaves)
            .set_frequency(frequency);
        Self { perlin: noise }
    }

    pub fn random_seed() -> u32 {
        rand::random::<u32>()
    }
}

impl ElevationMapGenerator for PerlinGenerator {
    fn elevation_at(&self, coordinates: Point, _width: usize, _height: usize) -> f64 {
        let value = self
            .perlin
            .get([coordinates.x as f64, coordinates.y as f64]);

        // Normalize to [0, 1].
        ((value + 1.0) * 0.5).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapOptions {
    pub energy_count: u32,
    pub diamond_count: u32,
    pub octaves: usize,
    pub frequency: f64,
    pub seed: Option<u32>,
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            energy_count: 12,
            diamond_count: 6,
            octaves: 2,
            frequency: 0.02,
            seed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    options: MapOptions,
    base: Option<Base>,
    pub elevation_map: Vec<f64>,
    terrain_map: Vec<Terrain>,
    mineral_map: Vec<Option<Mineral>>,
}

impl Map {
    fn get_index_from_coordinates(&self, coordinates: Point) -> Option<usize> {
        if coordinates.x < self.width && coordinates.y < self.height {
            Some(coordinates.y * self.width + coordinates.x)
        } else {
            None
        }
    }

    fn is_valid_2x2_plains_block(&self, x: usize, y: usize) -> bool {
        if x + 1 >= self.width || y + 1 >= self.height {
            return false;
        }

        self.terrain_at(point!(x, y)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x + 1, y)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x, y + 1)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x + 1, y + 1)) == Some(Terrain::Plains)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn points(&self) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                points.push(point!(x, y));
            }
        }
        points
    }

    pub fn base_center(&self) -> Option<Point> {
        let base = self.base.as_ref()?;
        Some(point!(
            (base.coordinates.0.x + base.coordinates.1.x) / 2,
            (base.coordinates.0.y + base.coordinates.2.y) / 2
        ))
    }

    pub fn base_tiles(&self) -> Option<[Point; 4]> {
        let base = self.base.as_ref()?;
        Some([
            base.coordinates.0,
            base.coordinates.1,
            base.coordinates.2,
            base.coordinates.3,
        ])
    }

    pub fn set_options(&mut self, options: MapOptions) {
        self.options = options;
    }

    pub fn seed(&self) -> u32 {
        self.options
            .seed
            .expect("map seed should be resolved during initialization")
    }

    pub fn minerals(&self) -> Vec<(Point, Mineral)> {
        let mut minerals = Vec::new();

        for (idx, mineral) in self.mineral_map.iter().enumerate() {
            if let Some(mineral) = mineral {
                let coordinates = point!(idx % self.width, idx / self.width);
                minerals.push((coordinates, *mineral));
            }
        }

        minerals
    }

    fn try_place_mineral(&mut self, kind: MineralKind, value: u32, rng: &mut SeededRng) {
        for _ in 0..10 {
            let x = rng.range_usize(0..self.width);
            let y = rng.range_usize(0..self.height);
            let idx = y * self.width + x;

            let terrain = self.terrain_map[idx];
            if matches!(
                terrain,
                Terrain::Plains | Terrain::Hills | Terrain::ShallowWater
            ) && self.mineral_map[idx].is_none()
            {
                self.mineral_map[idx] = Some(Mineral {
                    kind,
                    value,
                    max_value: value,
                });
                return;
            }
        }
    }

    pub fn initialize(&mut self) {
        let seed = self
            .options
            .seed
            .unwrap_or_else(PerlinGenerator::random_seed);
        let mut attempt = 0;
        loop {
            let generation_seed = seed.wrapping_add(attempt);
            self.create_elevation_map(generation_seed);
            self.create_terrain_from_elevation();
            if let Some(base) = self.find_base_location() {
                self.options.seed = Some(generation_seed);
                let b = base.coordinates;
                self.set_terrain_at(b.0, Terrain::Base);
                self.set_terrain_at(b.1, Terrain::Base);
                self.set_terrain_at(b.2, Terrain::Base);
                self.set_terrain_at(b.3, Terrain::Base);
                self.base = Some(base);
                break;
            }
            attempt = attempt.wrapping_add(1);
        }
        self.create_minerals();
    }

    fn generate_elevation_map<G: ElevationMapGenerator>(
        width: usize,
        height: usize,
        generator: &G,
    ) -> Vec<f64> {
        let mut elevation_map = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                elevation_map.push(
                    generator
                        .elevation_at(point!(x, y), width, height)
                        .clamp(0.0, 1.0),
                );
            }
        }

        elevation_map
    }

    fn set_terrain_from_elevation(&self, elevation_point: &f64) -> Terrain {
        match elevation_point {
            elevation_point if *elevation_point < 0.30 => Terrain::DeepWater,
            elevation_point if *elevation_point < 0.42 => Terrain::ShallowWater,
            elevation_point if *elevation_point < 0.63 => Terrain::Plains,
            elevation_point if *elevation_point < 0.82 => Terrain::Hills,
            elevation_point if *elevation_point == 4.0 => Terrain::Base,
            _ => Terrain::Mountains,
        }
    }

    pub fn render_tile_from_mineral(&self, mineral: MineralKind) -> (String, Color) {
        match mineral {
            MineralKind::Energy => (String::from('E'), Color::Yellow),
            MineralKind::Diamond => (String::from('D'), Color::Cyan),
        }
    }

    pub fn render_tile_from_terrain(&self, terrain: Terrain) -> (String, Color) {
        match terrain {
            Terrain::DeepWater => (String::from('≈'), Color::Blue),
            Terrain::ShallowWater => (String::from('~'), Color::Cyan),
            Terrain::Plains => (String::from('.'), Color::Green),
            Terrain::Hills => (String::from('^'), Color::Gray),
            Terrain::Mountains => (String::from('▲'), Color::DarkGray),
            Terrain::Base => (String::from('B'), Color::Magenta),
        }
    }

    /// Convenience constructor using default Perlin settings.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            options: MapOptions::default(),
            base: None,
            elevation_map: Vec::with_capacity(width * height),
            terrain_map: Vec::with_capacity(width * height),
            mineral_map: vec![None; width * height],
        }
    }

    #[cfg(test)]
    pub fn from_terrain_for_tests(width: usize, height: usize, terrain_map: Vec<Terrain>) -> Self {
        assert_eq!(terrain_map.len(), width * height);
        Self {
            width,
            height,
            options: MapOptions::default(),
            base: None,
            elevation_map: vec![0.5; width * height],
            terrain_map,
            mineral_map: vec![None; width * height],
        }
    }

    fn create_elevation_map(&mut self, seed: u32) {
        let generator = PerlinGenerator::new(seed, self.options.octaves, self.options.frequency);
        self.elevation_map = Self::generate_elevation_map(self.width, self.height, &generator);
    }

    fn create_terrain_from_elevation(&mut self) {
        let mut terrain_map: Vec<Terrain> = Vec::new();

        for elevation_point in self.elevation_map.iter() {
            terrain_map.push(self.set_terrain_from_elevation(elevation_point));
        }

        self.terrain_map = terrain_map;
    }

    fn create_minerals(&mut self) {
        self.mineral_map = vec![None; self.width * self.height];
        let mut rng = SeededRng::new(self.seed().wrapping_add(0x9E37_79B9));

        for _ in 0..self.options.energy_count {
            let value = rng.range_u32_inclusive(3..=8);
            self.try_place_mineral(MineralKind::Energy, value, &mut rng);
        }

        for _ in 0..self.options.diamond_count {
            let value = rng.range_u32_inclusive(2..=5);
            self.try_place_mineral(MineralKind::Diamond, value, &mut rng);
        }
    }

    pub fn mineral_at(&self, coordinates: Point) -> Option<Mineral> {
        let idx = self.get_index_from_coordinates(coordinates)?;
        self.mineral_map[idx]
    }

    pub fn terrain_at(&self, coordinates: Point) -> Option<Terrain> {
        let index = self.get_index_from_coordinates(coordinates)?;

        if index >= self.terrain_map.len() {
            return None;
        }

        Some(self.terrain_map[index])
    }

    pub fn is_walkable(&self, coordinates: Point) -> bool {
        !matches!(
            self.terrain_at(coordinates),
            None | Some(Terrain::DeepWater) | Some(Terrain::Mountains)
        )
    }

    pub fn terrain_cost(&self, coordinates: Point) -> Option<u32> {
        match self.terrain_at(coordinates)? {
            Terrain::DeepWater | Terrain::Mountains => None,
            Terrain::Plains | Terrain::Base => Some(1),
            Terrain::Hills => Some(2),
            Terrain::ShallowWater => Some(2),
        }
    }

    pub fn neighbors(&self, coordinates: Point) -> Vec<Point> {
        let mut neighbors = Vec::with_capacity(4);
        let candidates = [
            (coordinates.x.checked_sub(1), Some(coordinates.y)),
            (Some(coordinates.x + 1), Some(coordinates.y)),
            (Some(coordinates.x), coordinates.y.checked_sub(1)),
            (Some(coordinates.x), Some(coordinates.y + 1)),
        ];

        for (x, y) in candidates {
            if let (Some(x), Some(y)) = (x, y) {
                let point = point!(x, y);
                if self.is_walkable(point) {
                    neighbors.push(point);
                }
            }
        }

        neighbors
    }

    fn set_terrain_at(&mut self, coordinates: Point, terrain: Terrain) {
        let index = self.get_index_from_coordinates(coordinates).unwrap();
        self.terrain_map[index] = terrain;
    }

    fn find_base_location(&self) -> Option<Base> {
        let center_x = self.width / 2;
        let center_y = self.height / 2;

        let k_max = std::cmp::max(
            std::cmp::max(center_x, center_y),
            std::cmp::max(self.width - center_x - 1, self.height - center_y - 1),
        );

        for k in 0..=k_max {
            // Top edge: left to right
            for x in (center_x.saturating_sub(k))..=(center_x + k) {
                let y = center_y.saturating_sub(k);
                if self.is_valid_2x2_plains_block(x, y) {
                    return Some(Base {
                        coordinates: (
                            point!(x, y),
                            point!(x + 1, y),
                            point!(x, y + 1),
                            point!(x + 1, y + 1),
                        ),
                    });
                }
            }

            // Right edge: top to bottom
            for y in (center_y.saturating_sub(k) + 1)..=(center_y + k) {
                let x = center_x + k;
                if self.is_valid_2x2_plains_block(x, y) {
                    return Some(Base {
                        coordinates: (
                            point!(x, y),
                            point!(x + 1, y),
                            point!(x, y + 1),
                            point!(x + 1, y + 1),
                        ),
                    });
                }
            }

            // Bottom edge: right to left
            for x in ((center_x + k).saturating_sub(1))..=(center_x.saturating_sub(k) + 1) {
                let y = center_y + k;
                if x <= center_x + k && self.is_valid_2x2_plains_block(x, y) {
                    return Some(Base {
                        coordinates: (
                            point!(x, y),
                            point!(x + 1, y),
                            point!(x, y + 1),
                            point!(x + 1, y + 1),
                        ),
                    });
                }
            }

            // Left edge: bottom to top
            for y in ((center_y + k).saturating_sub(1))..=(center_y.saturating_sub(k) + 1) {
                let x = center_x.saturating_sub(k);
                if y <= center_y + k && self.is_valid_2x2_plains_block(x, y) {
                    return Some(Base {
                        coordinates: (
                            point!(x, y),
                            point!(x + 1, y),
                            point!(x, y + 1),
                            point!(x + 1, y + 1),
                        ),
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialized_map(seed: u32) -> Map {
        let mut map = Map::new(80, 24);
        map.set_options(MapOptions {
            seed: Some(seed),
            ..MapOptions::default()
        });
        map.initialize();
        map
    }

    #[test]
    fn same_seed_replays_same_map() {
        let first = initialized_map(12345);
        let second = initialized_map(12345);

        assert_eq!(first.terrain_map, second.terrain_map);
        assert_eq!(first.minerals(), second.minerals());
        assert_eq!(first.base_center(), second.base_center());
    }

    #[test]
    fn points_returns_all_coordinates() {
        let map = Map::from_terrain_for_tests(3, 2, vec![Terrain::Plains; 6]);
        let points = map.points();
        assert_eq!(points.len(), 6);
        assert!(points.contains(&point!(0, 0)));
        assert!(points.contains(&point!(2, 1)));
    }

    #[test]
    fn size_returns_dimensions() {
        let map = Map::from_terrain_for_tests(5, 7, vec![Terrain::Plains; 35]);
        assert_eq!(map.size(), (5, 7));
    }

    #[test]
    fn terrain_at_valid_point() {
        let map = Map::from_terrain_for_tests(
            2,
            2,
            vec![
                Terrain::Plains,
                Terrain::Hills,
                Terrain::Mountains,
                Terrain::DeepWater,
            ],
        );
        assert_eq!(map.terrain_at(point!(0, 0)), Some(Terrain::Plains));
        assert_eq!(map.terrain_at(point!(1, 1)), Some(Terrain::DeepWater));
    }

    #[test]
    fn terrain_at_out_of_bounds() {
        let map = Map::from_terrain_for_tests(2, 2, vec![Terrain::Plains; 4]);
        assert_eq!(map.terrain_at(point!(5, 5)), None);
    }

    #[test]
    fn is_walkable_returns_false_for_deep_water() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::DeepWater]);
        assert!(!map.is_walkable(point!(0, 0)));
    }

    #[test]
    fn is_walkable_returns_false_for_mountains() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Mountains]);
        assert!(!map.is_walkable(point!(0, 0)));
    }

    #[test]
    fn is_walkable_returns_true_for_plains() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert!(map.is_walkable(point!(0, 0)));
    }

    #[test]
    fn is_walkable_returns_true_for_hills_and_shallow_water_and_base() {
        for terrain in [Terrain::Hills, Terrain::ShallowWater, Terrain::Base] {
            let map = Map::from_terrain_for_tests(1, 1, vec![terrain]);
            assert!(
                map.is_walkable(point!(0, 0)),
                "{terrain:?} should be walkable"
            );
        }
    }

    #[test]
    fn is_walkable_out_of_bounds() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert!(!map.is_walkable(point!(5, 5)));
    }

    #[test]
    fn terrain_cost_plains_and_base() {
        for terrain in [Terrain::Plains, Terrain::Base] {
            let map = Map::from_terrain_for_tests(1, 1, vec![terrain]);
            assert_eq!(map.terrain_cost(point!(0, 0)), Some(1));
        }
    }

    #[test]
    fn terrain_cost_hills_and_shallow_water() {
        for terrain in [Terrain::Hills, Terrain::ShallowWater] {
            let map = Map::from_terrain_for_tests(1, 1, vec![terrain]);
            assert_eq!(map.terrain_cost(point!(0, 0)), Some(2));
        }
    }

    #[test]
    fn terrain_cost_impassable() {
        for terrain in [Terrain::DeepWater, Terrain::Mountains] {
            let map = Map::from_terrain_for_tests(1, 1, vec![terrain]);
            assert_eq!(map.terrain_cost(point!(0, 0)), None);
        }
    }

    #[test]
    fn terrain_cost_out_of_bounds() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert_eq!(map.terrain_cost(point!(5, 5)), None);
    }

    #[test]
    fn neighbors_includes_cardinal_walkable() {
        let map = Map::from_terrain_for_tests(3, 3, vec![Terrain::Plains; 9]);
        let neighbors = map.neighbors(point!(1, 1));
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&point!(0, 1)));
        assert!(neighbors.contains(&point!(2, 1)));
        assert!(neighbors.contains(&point!(1, 0)));
        assert!(neighbors.contains(&point!(1, 2)));
    }

    #[test]
    fn neighbors_excludes_impassable_terrain() {
        let map = Map::from_terrain_for_tests(
            3,
            1,
            vec![Terrain::Mountains, Terrain::Plains, Terrain::DeepWater],
        );
        let neighbors = map.neighbors(point!(1, 0));
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn neighbors_at_top_left_corner() {
        let map = Map::from_terrain_for_tests(2, 2, vec![Terrain::Plains; 4]);
        let neighbors = map.neighbors(point!(0, 0));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&point!(1, 0)));
        assert!(neighbors.contains(&point!(0, 1)));
    }

    #[test]
    fn set_terrain_from_elevation_boundaries() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert_eq!(map.set_terrain_from_elevation(&0.29), Terrain::DeepWater);
        assert_eq!(map.set_terrain_from_elevation(&0.30), Terrain::ShallowWater);
        assert_eq!(map.set_terrain_from_elevation(&0.41), Terrain::ShallowWater);
        assert_eq!(map.set_terrain_from_elevation(&0.42), Terrain::Plains);
        assert_eq!(map.set_terrain_from_elevation(&0.62), Terrain::Plains);
        assert_eq!(map.set_terrain_from_elevation(&0.63), Terrain::Hills);
        assert_eq!(map.set_terrain_from_elevation(&0.81), Terrain::Hills);
        assert_eq!(map.set_terrain_from_elevation(&0.82), Terrain::Mountains);
        assert_eq!(map.set_terrain_from_elevation(&1.0), Terrain::Mountains);
    }

    #[test]
    fn mineral_at_returns_none_without_minerals() {
        let map = Map::from_terrain_for_tests(2, 2, vec![Terrain::Plains; 4]);
        assert_eq!(map.mineral_at(point!(0, 0)), None);
    }

    #[test]
    fn render_tile_from_terrain_all_variants() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert_eq!(map.render_tile_from_terrain(Terrain::DeepWater).0, "≈");
        assert_eq!(map.render_tile_from_terrain(Terrain::ShallowWater).0, "~");
        assert_eq!(map.render_tile_from_terrain(Terrain::Plains).0, ".");
        assert_eq!(map.render_tile_from_terrain(Terrain::Hills).0, "^");
        assert_eq!(map.render_tile_from_terrain(Terrain::Mountains).0, "▲");
        assert_eq!(map.render_tile_from_terrain(Terrain::Base).0, "B");
    }

    #[test]
    fn render_tile_from_mineral_all_variants() {
        let map = Map::from_terrain_for_tests(1, 1, vec![Terrain::Plains]);
        assert_eq!(map.render_tile_from_mineral(MineralKind::Energy).0, "E");
        assert_eq!(map.render_tile_from_mineral(MineralKind::Diamond).0, "D");
    }

    #[test]
    fn seeded_rng_determinism() {
        let mut rng_a = SeededRng::new(42);
        let mut rng_b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(rng_a.next_u32(), rng_b.next_u32());
        }
    }

    #[test]
    fn seeded_rng_range_usize() {
        let mut rng = SeededRng::new(42);
        for _ in 0..100 {
            let v = rng.range_usize(5..10);
            assert!((5..10).contains(&v));
        }
    }

    #[test]
    fn seeded_rng_range_u32_inclusive() {
        let mut rng = SeededRng::new(42);
        for _ in 0..100 {
            let v = rng.range_u32_inclusive(3..=8);
            assert!((3..=8).contains(&v));
        }
    }

    #[test]
    fn find_base_location_on_all_plains() {
        let map = Map::from_terrain_for_tests(10, 10, vec![Terrain::Plains; 100]);
        let base = map.find_base_location();
        assert!(base.is_some(), "should find base on all-plains map");
        if let Some(base) = base {
            let (a, b, c, _) = base.coordinates;
            assert_eq!(a.x + 1, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.x, c.x);
            assert_eq!(a.y + 1, c.y);
        }
    }

    #[test]
    fn base_center_returns_none_when_no_base() {
        let map = Map::from_terrain_for_tests(5, 5, vec![Terrain::Plains; 25]);
        assert_eq!(map.base_center(), None);
    }

    #[test]
    fn base_tiles_returns_none_when_no_base() {
        let map = Map::from_terrain_for_tests(5, 5, vec![Terrain::Plains; 25]);
        assert_eq!(map.base_tiles(), None);
    }

    #[test]
    fn initialized_map_has_base() {
        let map = initialized_map(42);
        assert!(map.base_center().is_some());
        assert!(map.base_tiles().is_some());
    }

    #[test]
    fn initialized_map_has_minerals() {
        let map = initialized_map(42);
        let minerals = map.minerals();
        assert!(!minerals.is_empty());
        assert_eq!(
            minerals.len() as u32,
            map.options.energy_count + map.options.diamond_count,
        );
    }
}
