use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    DeepWater,
    ShallowWater,
    Plains,
    Hills,
    Mountains,
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    elevation: Vec<f64>,
}

/// Interface for any terrain generation strategy.
///
/// Implement this trait for each technique (Perlin, Simplex, Worley, etc.)
/// and call `Map::from_generator`.
pub trait MapGenerator {
    /// Returns an elevation value for one tile.
    /// Implementations can use any algorithm as long as they return [0.0, 1.0].
    fn elevation_at(&self, x: usize, y: usize, width: usize, height: usize) -> f64;
}

#[derive(Debug, Clone)]
pub struct PerlinGenerator {
    frequency: f64,
    perlin: Perlin,
}

impl PerlinGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            frequency: 0.035,
            perlin: Perlin::new(seed),
        }
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
    fn elevation_at(&self, x: usize, y: usize, _width: usize, _height: usize) -> f64 {
        let nx = x as f64 * self.frequency;
        let ny = y as f64 * self.frequency;

        // Multi-octave Perlin sampling for richer terrain.
        let base = self.perlin.get([nx, ny]);
        let detail = self.perlin.get([nx * 2.0, ny * 2.0]) * 0.5;
        let micro = self.perlin.get([nx * 4.0, ny * 4.0]) * 0.25;

        let value = (base + detail + micro) / 1.75;

        // Normalize to [0, 1].
        ((value + 1.0) * 0.5).clamp(0.0, 1.0)
    }
}

impl Map {
    pub fn from_generator<G: MapGenerator>(width: usize, height: usize, generator: &G) -> Self {
        let mut elevation = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                elevation.push(generator.elevation_at(x, y, width, height).clamp(0.0, 1.0));
            }
        }

        Self {
            width,
            height,
            elevation,
        }
    }

    /// Convenience constructor using default Perlin settings.
    pub fn new(width: usize, height: usize) -> Self {
        let generator = PerlinGenerator::default();
        Self::from_generator(width, height, &generator)
    }

    pub fn terrain_at(&self, x: usize, y: usize) -> Option<Terrain> {
        self.elevation_at(x, y).map(|h| match h {
            h if h < 0.30 => Terrain::DeepWater,
            h if h < 0.42 => Terrain::ShallowWater,
            h if h < 0.63 => Terrain::Plains,
            h if h < 0.82 => Terrain::Hills,
            _ => Terrain::Mountains,
        })
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn elevation_at(&self, x: usize, y: usize) -> Option<f64> {
        self.index(x, y).map(|idx| self.elevation[idx])
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }
}
