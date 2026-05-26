use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TimelineViewportGeometry {
    pub width_px: u32,
    pub height_px: u32,
    pub track_height_px: u32,
}

impl TimelineViewportGeometry {
    pub fn new(width_px: u32, height_px: u32, track_height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
            track_height_px,
        }
    }

    pub fn validate(&self) -> bool {
        self.width_px > 0 && self.height_px > 0 && self.track_height_px > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TimelineViewportPoint {
    pub x_px: u32,
    pub y_px: u32,
}

impl TimelineViewportPoint {
    pub fn new(x_px: u32, y_px: u32) -> Self {
        Self { x_px, y_px }
    }
}
