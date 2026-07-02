use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct GameClock {
    tick_rate: Duration,
    accumulated: Duration,
    previous: Instant,
    tick_count: u64,
    paused: bool,
}

const TICK_RATE_NANOS: u64 = 33_333_333; // ~30 Hz
const MAX_FRAME_TIME_MS: u64 = 100;

impl GameClock {
    pub fn new() -> Self {
        Self {
            tick_rate: Duration::from_nanos(TICK_RATE_NANOS),
            accumulated: Duration::ZERO,
            previous: Instant::now(),
            tick_count: 0,
            paused: false,
        }
    }

    pub fn advance(&mut self) -> u32 {
        if self.paused {
            return 0;
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.previous);
        self.previous = now;

        let capped = elapsed.min(Duration::from_millis(MAX_FRAME_TIME_MS));
        self.accumulated += capped;

        let ticks = (self.accumulated.as_nanos() / self.tick_rate.as_nanos()) as u32;
        if ticks > 0 {
            self.accumulated -= self.tick_rate * ticks;
            self.tick_count += ticks as u64;
        }

        ticks
    }

    pub fn elapsed(&self) -> Duration {
        self.tick_rate * self.tick_count as u32
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn advance_by(&mut self, duration: Duration) -> u32 {
        let ticks = (duration.as_nanos() / self.tick_rate.as_nanos()) as u64;
        self.tick_count += ticks;
        ticks as u32
    }

    pub fn elapsed_formatted(&self) -> String {
        let total_secs = self.elapsed().as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}
