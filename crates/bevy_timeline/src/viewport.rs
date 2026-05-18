use bevy::prelude::Resource;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Serialize)]
pub struct TimelineViewport {
    pub start_ms: u64,
    pub end_ms: u64,
    pub duration_ms: u64,
}

impl Default for TimelineViewport {
    fn default() -> Self {
        Self {
            start_ms: 0,
            end_ms: 1,
            duration_ms: 1,
        }
    }
}

impl TimelineViewport {
    pub fn from_duration(duration_ms: u64) -> Self {
        let duration_ms = duration_ms.max(1);
        Self {
            start_ms: 0,
            end_ms: duration_ms,
            duration_ms,
        }
    }

    pub fn set_duration(&mut self, duration_ms: u64) {
        let duration_ms = duration_ms.max(1);
        if *self == Self::default() {
            *self = Self::from_duration(duration_ms);
            return;
        }
        self.duration_ms = duration_ms;
        self.start_ms = self.start_ms.min(duration_ms.saturating_sub(1));
        self.end_ms = self.end_ms.clamp(self.start_ms + 1, duration_ms);
    }

    pub fn set_range(&mut self, start_ms: u64, end_ms: u64) {
        self.start_ms = start_ms;
        self.end_ms = end_ms;
    }

    pub fn pan_by(&mut self, delta_ms: i64) {
        let width = self.width_ms();
        let max_start = self.duration_ms.saturating_sub(width);
        let start = if delta_ms.is_negative() {
            self.start_ms.saturating_sub(delta_ms.unsigned_abs())
        } else {
            self.start_ms.saturating_add(delta_ms as u64).min(max_start)
        };
        self.start_ms = start;
        self.end_ms = start + width;
    }

    pub fn zoom_around(&mut self, center_ms: u64, factor: f32) {
        let factor = factor.max(f32::EPSILON);
        let current_width = self.width_ms() as f32;
        let next_width = (current_width / factor)
            .round()
            .clamp(1.0, self.duration_ms as f32) as u64;
        let half_width = next_width / 2;
        let center_ms = center_ms.min(self.duration_ms);
        let start = center_ms.saturating_sub(half_width);
        let max_start = self.duration_ms.saturating_sub(next_width);
        self.start_ms = start.min(max_start);
        self.end_ms = self.start_ms + next_width;
    }

    pub fn width_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms).max(1)
    }
}
