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
}

#[derive(Debug, Clone, Copy)]
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

impl PerlinGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = Fbm::<Perlin>::new(seed).set_octaves(2).set_frequency(0.02);
        Self { perlin: noise }
    }

    pub fn random_seed() -> u32 {
        rand::random::<u32>()
    }
}

impl Default for PerlinGenerator {
    fn default() -> Self {
        Self::new(Self::random_seed())
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

#[derive(Debug, Clone)]
pub struct DefaultMap {
    width: usize,
    height: usize,
    pub elevation_map: Vec<f64>,
    terrain_map: Vec<Terrain>,
    mineral_map: Vec<Option<Mineral>>,
}

/// Interface for any map generation strategy
pub trait Map {
    fn new(width: usize, height: usize) -> Self;

    fn create_elevation_map(&mut self) -> ();
    fn create_terrain_from_elevation(&mut self) -> ();
    fn create_minerals(&mut self) -> ();

    fn initialize(&mut self) {
        loop {
            self.create_elevation_map();
            self.create_terrain_from_elevation();
            if let Some(base) = self.find_base_location() {
                let b = base.coordinates;
                self.set_terrain_at(b.0, Terrain::Base);
                self.set_terrain_at(b.1, Terrain::Base);
                self.set_terrain_at(b.2, Terrain::Base);
                self.set_terrain_at(b.3, Terrain::Base);
                break;
            }
        }
        self.create_minerals();
    }

    fn mineral_at(&self, coordinates: Point) -> Option<Mineral>;
    fn mine_at(&mut self, coordinates: Point) -> Option<MineralKind>;

    // Map a terrain from an elevation value.
    fn terrain_at(&self, coordinates: Point) -> Option<Terrain>;

    // Override terrain at coordinates
    fn set_terrain_at(&mut self, coordinates: Point, terrain: Terrain) -> ();

    fn find_base_location(&self) -> Option<Base>;

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

    fn render_tile_from_mineral(&self, mineral: MineralKind) -> (String, Color) {
        match mineral {
            MineralKind::Energy => (String::from('E'), Color::Yellow),
            MineralKind::Diamond => (String::from('D'), Color::Cyan),
        }
    }

    // Render a tile from a terrain
    fn render_tile_from_terrain(&self, terrain: Terrain) -> (String, Color) {
        match terrain {
            Terrain::DeepWater => (String::from('≈'), Color::Blue),
            Terrain::ShallowWater => (String::from('~'), Color::Cyan),
            Terrain::Plains => (String::from('.'), Color::Green),
            Terrain::Hills => (String::from('^'), Color::Gray),
            Terrain::Mountains => (String::from('▲'), Color::DarkGray),
            Terrain::Base => (String::from('B'), Color::Magenta),
        }
    }
}

impl DefaultMap {
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

    fn try_place_mineral(&mut self, kind: MineralKind, value: u32) {
        for _ in 0..10 {
            let x = rand::random_range(0..self.width);
            let y = rand::random_range(0..self.height);
            let idx = y * self.width + x;

            let terrain = self.terrain_map[idx];
            if matches!(
                terrain,
                Terrain::Plains | Terrain::Hills | Terrain::ShallowWater
            ) && self.mineral_map[idx].is_none()
            {
                self.mineral_map[idx] = Some(Mineral { kind, value });
                return;
            }
        }
    }
}

impl Map for DefaultMap {
    /// Convenience constructor using default Perlin settings.
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            elevation_map: Vec::with_capacity(width * height),
            terrain_map: Vec::with_capacity(width * height),
            mineral_map: vec![None; width * height],
        }
    }

    fn create_elevation_map(&mut self) {
        let generator = PerlinGenerator::default();
        self.elevation_map = Self::generate_elevation_map(self.width, self.height, &generator);
    }

    fn create_terrain_from_elevation(&mut self) {
        let mut terrain_map: Vec<Terrain> = Vec::new();

        for elevation_point in self.elevation_map.iter() {
            terrain_map.push(self.set_terrain_from_elevation(elevation_point));
        }

        self.terrain_map = terrain_map;
    }

    fn create_minerals(&mut self) -> () {
        self.mineral_map = vec![None; self.width * self.height];

        let nb_energy = rand::random_range(8..=15);
        let nb_diamond = rand::random_range(4..=8);

        for _ in 0..nb_energy {
            self.try_place_mineral(MineralKind::Energy, rand::random_range(3..=8));
        }

        for _ in 0..nb_diamond {
            self.try_place_mineral(MineralKind::Diamond, rand::random_range(2..=5));
        }
    }

    fn mineral_at(&self, coordinates: Point) -> Option<Mineral> {
        let idx = self.get_index_from_coordinates(coordinates)?;
        self.mineral_map[idx]
    }

    fn mine_at(&mut self, coordinates: Point) -> Option<MineralKind> {
        let idx = self.get_index_from_coordinates(coordinates)?;
        let mineral = self.mineral_map[idx].as_mut()?;

        mineral.value = mineral.value.saturating_sub(1);
        let kind = mineral.kind;

        if mineral.value == 0 {
            self.mineral_map[idx] = None;
        }

        Some(kind)
    }

    fn terrain_at(&self, coordinates: Point) -> Option<Terrain> {
        let index = self.get_index_from_coordinates(coordinates).unwrap();

        if index >= self.terrain_map.len() {
            return None;
        }

        Some(self.terrain_map[index])
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
