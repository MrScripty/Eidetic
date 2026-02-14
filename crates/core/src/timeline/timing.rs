use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A time range on the timeline, stored as milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

impl TimeRange {
    /// Create a new time range, validating that start < end.
    pub fn new(start_ms: u64, end_ms: u64) -> Result<Self> {
        let range = Self { start_ms, end_ms };
        range.validate()?;
        Ok(range)
    }

    /// Validate that start < end.
    pub fn validate(&self) -> Result<()> {
        if self.start_ms >= self.end_ms {
            return Err(Error::InvalidTimeRange {
                start_ms: self.start_ms,
                end_ms: self.end_ms,
            });
        }
        Ok(())
    }

    /// Duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.end_ms - self.start_ms
    }

    /// Check if a time point falls within this range (inclusive start, exclusive end).
    pub fn contains(&self, time_ms: u64) -> bool {
        time_ms >= self.start_ms && time_ms < self.end_ms
    }

    /// Check if two ranges overlap.
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start_ms < other.end_ms && other.start_ms < self.end_ms
    }

    /// Approximate page count (1 page ≈ 1 minute of screen time).
    pub fn estimated_pages(&self) -> f64 {
        self.duration_ms() as f64 / 60_000.0
    }
}

/// Format milliseconds as MM:SS for display.
pub fn format_time(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_range_succeeds() {
        let range = TimeRange::new(0, 60_000).unwrap();
        assert_eq!(range.start_ms, 0);
        assert_eq!(range.end_ms, 60_000);
    }

    #[test]
    fn test_new_invalid_range_returns_error() {
        assert!(TimeRange::new(60_000, 0).is_err());
        assert!(TimeRange::new(1000, 1000).is_err());
    }

    #[test]
    fn test_duration_ms_returns_difference() {
        let range = TimeRange::new(10_000, 70_000).unwrap();
        assert_eq!(range.duration_ms(), 60_000);
    }

    #[test]
    fn test_contains_point_inside_returns_true() {
        let range = TimeRange::new(10_000, 20_000).unwrap();
        assert!(range.contains(15_000));
        assert!(range.contains(10_000)); // inclusive start
    }

    #[test]
    fn test_contains_point_at_end_returns_false() {
        let range = TimeRange::new(10_000, 20_000).unwrap();
        assert!(!range.contains(20_000)); // exclusive end
    }

    #[test]
    fn test_overlaps_overlapping_ranges_returns_true() {
        let a = TimeRange::new(0, 10_000).unwrap();
        let b = TimeRange::new(5_000, 15_000).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn test_overlaps_adjacent_ranges_returns_false() {
        let a = TimeRange::new(0, 10_000).unwrap();
        let b = TimeRange::new(10_000, 20_000).unwrap();
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_estimated_pages_one_minute_equals_one_page() {
        let range = TimeRange::new(0, 60_000).unwrap();
        assert!((range.estimated_pages() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_time_displays_correctly() {
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(150_000), "2:30");
        assert_eq!(format_time(1_320_000), "22:00");
    }
}
