use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    DeepWater,
    ShallowWater,
    Plains,
    Hills,
    Mountains,
    Base,
}

#[derive(Debug, Clone)]
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
    coordinates: (Point, Point, Point, Point),
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    elevation: Vec<f64>,
    base: Option<Base>,
}

/// Interface for any terrain generation strategy.
///
/// Implement this trait for each technique (Perlin, Simplex, Worley, etc.)
/// and call `Map::from_generator`.
pub trait MapGenerator {
    /// Returns an elevation value for one tile.
    /// Implementations can use any algorithm as long as they return [0.0, 1.0].
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

impl MapGenerator for PerlinGenerator {
    fn elevation_at(&self, coordinates: Point, _width: usize, _height: usize) -> f64 {
        let value = self
            .perlin
            .get([coordinates.x as f64, coordinates.y as f64]);

        // Normalize to [0, 1].
        ((value + 1.0) * 0.5).clamp(0.0, 1.0)
    }
}

impl Map {
    pub fn from_generator<G: MapGenerator>(width: usize, height: usize, generator: &G) -> Self {
        let mut elevation = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                elevation.push(
                    generator
                        .elevation_at(point!(x, y), width, height)
                        .clamp(0.0, 1.0),
                );
            }
        }

        Self {
            width,
            height,
            elevation,
            base: None,
        }
    }

    pub fn find_base_location(&self) -> Option<Base> {
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

    fn is_valid_2x2_plains_block(&self, x: usize, y: usize) -> bool {
        if x + 1 >= self.width || y + 1 >= self.height {
            return false;
        }

        self.terrain_at(point!(x, y)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x + 1, y)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x, y + 1)) == Some(Terrain::Plains)
            && self.terrain_at(point!(x + 1, y + 1)) == Some(Terrain::Plains)
    }

    /// Convenience constructor using default Perlin settings.
    pub fn new(width: usize, height: usize) -> Self {
        let generator = PerlinGenerator::default();
        // Self::from_generator(width, height, &generator)

        let mut base: Option<Base> = None;
        let mut map: Option<Map> = None;

        while base.is_none() {
            map = Some(Self::from_generator(width, height, &generator));
            base = map.as_ref().unwrap().find_base_location();
        }

        map.as_mut()
            .unwrap()
            .set_terrain(base.clone().unwrap().coordinates.0, Terrain::Base);
        map.as_mut()
            .unwrap()
            .set_terrain(base.clone().unwrap().coordinates.1, Terrain::Base);
        map.as_mut()
            .unwrap()
            .set_terrain(base.clone().unwrap().coordinates.2, Terrain::Base);
        map.as_mut()
            .unwrap()
            .set_terrain(base.clone().unwrap().coordinates.3, Terrain::Base);

        map.unwrap()
    }

    fn set_terrain(&mut self, coordinates: Point, terrain: Terrain) {
        let index = coordinates.y * self.width + coordinates.x;
        self.elevation[index] = match terrain {
            Terrain::DeepWater => 0.30,
            Terrain::ShallowWater => 0.42,
            Terrain::Plains => 0.63,
            Terrain::Hills => 0.82,
            Terrain::Mountains => 1.0,
            Terrain::Base => 4.0,
        };
    }

    pub fn terrain_at(&self, coordinates: Point) -> Option<Terrain> {
        self.elevation_at(coordinates).map(|h| match h {
            h if h < 0.30 => Terrain::DeepWater,
            h if h < 0.42 => Terrain::ShallowWater,
            h if h < 0.63 => Terrain::Plains,
            h if h < 0.82 => Terrain::Hills,
            h if h == 4.0 => Terrain::Base,
            _ => Terrain::Mountains,
        })
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn elevation_at(&self, coordinates: Point) -> Option<f64> {
        self.index(coordinates).map(|idx| self.elevation[idx])
    }

    fn index(&self, coordinates: Point) -> Option<usize> {
        if coordinates.x < self.width && coordinates.y < self.height {
            Some(coordinates.y * self.width + coordinates.x)
        } else {
            None
        }
    }
}
