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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clock_is_not_paused() {
        let clock = GameClock::new();
        assert!(!clock.is_paused());
        assert_eq!(clock.elapsed(), Duration::ZERO);
    }

    #[test]
    fn toggle_pause_works() {
        let mut clock = GameClock::new();
        clock.toggle_pause();
        assert!(clock.is_paused());
        clock.toggle_pause();
        assert!(!clock.is_paused());
    }

    #[test]
    fn advance_returns_zero_when_paused() {
        let mut clock = GameClock::new();
        clock.toggle_pause();
        assert_eq!(clock.advance(), 0);
    }

    #[test]
    fn advance_by_produces_expected_ticks() {
        let mut clock = GameClock::new();
        let one_sec = Duration::from_secs(1);
        let ticks = clock.advance_by(one_sec);
        let nanos_per_tick = 33_333_333;
        assert_eq!(ticks, (1_000_000_000 / nanos_per_tick) as u32);
    }

    #[test]
    fn elapsed_matches_advanced_time() {
        let mut clock = GameClock::new();
        let one_sec = Duration::from_secs(1);
        let ticks = clock.advance_by(one_sec);
        assert_eq!(clock.elapsed().as_nanos() as u64, ticks as u64 * 33_333_333);
    }

    #[test]
    fn elapsed_formatted_shows_seconds() {
        let mut clock = GameClock::new();
        clock.advance_by(Duration::from_secs(65));
        assert_eq!(clock.elapsed_formatted(), "01:04");
    }

    #[test]
    fn elapsed_formatted_shows_hours() {
        let mut clock = GameClock::new();
        clock.advance_by(Duration::from_secs(3661));
        assert_eq!(clock.elapsed_formatted(), "01:01:00");
    }

    #[test]
    fn elapsed_formatted_zero() {
        let clock = GameClock::new();
        assert_eq!(clock.elapsed_formatted(), "00:00");
    }

    #[test]
    fn advance_by_30s_produces_900_ticks() {
        let mut clock = GameClock::new();
        assert_eq!(clock.advance_by(Duration::from_secs(30)), 900);
        assert_eq!(clock.advance_by(Duration::from_secs(30)), 900);
    }
}
