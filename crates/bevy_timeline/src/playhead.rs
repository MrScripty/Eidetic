use bevy::prelude::Resource;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Serialize)]
pub struct TimelinePlayhead {
    pub position_ms: u64,
    pub duration_ms: u64,
}

impl Default for TimelinePlayhead {
    fn default() -> Self {
        Self {
            position_ms: 0,
            duration_ms: 1,
        }
    }
}

impl TimelinePlayhead {
    pub fn from_duration(duration_ms: u64) -> Self {
        Self {
            position_ms: 0,
            duration_ms: duration_ms.max(1),
        }
    }

    pub fn set_duration(&mut self, duration_ms: u64) {
        let duration_ms = duration_ms.max(1);
        self.duration_ms = duration_ms;
        self.position_ms = self.position_ms.min(duration_ms);
    }

    pub fn set_position(&mut self, position_ms: u64) {
        self.position_ms = position_ms.min(self.duration_ms);
    }
}
