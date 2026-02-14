use serde::{Deserialize, Serialize};

use super::timing::TimeRange;

/// The episode's act structure (pre-placed, adjustable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeStructure {
    /// e.g., "Standard 30-Min Comedy"
    pub template_name: String,
    pub segments: Vec<StructureSegment>,
}

/// A single structural segment (act, cold open, commercial break, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSegment {
    pub segment_type: SegmentType,
    pub time_range: TimeRange,
    pub label: String,
}

/// The type of structural segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentType {
    ColdOpen,
    MainTitles,
    Act,
    CommercialBreak,
    Tag,
}

impl EpisodeStructure {
    /// Standard 30-minute TV episode structure (~22 min content).
    ///
    /// ```text
    ///  0:00 — Cold Open ——— ~2 min
    ///  2:00 — Main Titles —— ~0:30
    ///  2:30 — Act One ——————  ~7 min
    ///  9:30 — Commercial ———
    ///  9:30 — Act Two ——————  ~7 min
    /// 16:30 — Commercial ———
    /// 16:30 — Act Three ————  ~5 min
    /// 21:30 — Tag ——————————  ~0:30
    /// 22:00 — End
    /// ```
    pub fn standard_30_min() -> Self {
        Self {
            template_name: "Standard 30-Min Comedy".into(),
            segments: vec![
                StructureSegment {
                    segment_type: SegmentType::ColdOpen,
                    time_range: TimeRange { start_ms: 0, end_ms: 120_000 },
                    label: "Cold Open".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::MainTitles,
                    time_range: TimeRange { start_ms: 120_000, end_ms: 150_000 },
                    label: "Main Titles".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::Act,
                    time_range: TimeRange { start_ms: 150_000, end_ms: 570_000 },
                    label: "Act One".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::CommercialBreak,
                    time_range: TimeRange { start_ms: 570_000, end_ms: 570_000 },
                    label: "Commercial Break".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::Act,
                    time_range: TimeRange { start_ms: 570_000, end_ms: 990_000 },
                    label: "Act Two".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::CommercialBreak,
                    time_range: TimeRange { start_ms: 990_000, end_ms: 990_000 },
                    label: "Commercial Break".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::Act,
                    time_range: TimeRange { start_ms: 990_000, end_ms: 1_290_000 },
                    label: "Act Three".into(),
                },
                StructureSegment {
                    segment_type: SegmentType::Tag,
                    time_range: TimeRange { start_ms: 1_290_000, end_ms: 1_320_000 },
                    label: "Tag".into(),
                },
            ],
        }
    }
}
