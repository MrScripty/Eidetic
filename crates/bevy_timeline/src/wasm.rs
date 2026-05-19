use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;
use wasm_bindgen::prelude::*;

use crate::TimelineRendererApp;

#[wasm_bindgen]
pub struct WasmTimelineRenderer {
    renderer: TimelineRendererApp,
}

#[wasm_bindgen]
impl WasmTimelineRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            renderer: TimelineRendererApp::new(),
        }
    }

    pub fn set_projection(&mut self, projection: JsValue) -> Result<(), JsValue> {
        let projection: TimelineRenderProjection = serde_wasm_bindgen::from_value(projection)
            .map_err(|error| JsValue::from_str(&format!("invalid timeline projection: {error}")))?;
        self.renderer.set_projection(projection);
        Ok(())
    }

    pub fn select_node(&mut self, node_id: String) -> Result<(), JsValue> {
        let node_id = NodeId(
            uuid::Uuid::parse_str(&node_id)
                .map_err(|error| JsValue::from_str(&format!("invalid node id: {error}")))?,
        );
        self.renderer
            .select_node(node_id)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn select_clip_at_time(&mut self, track_id: String, time_ms: u64) -> Result<(), JsValue> {
        let track_id = TrackId(
            uuid::Uuid::parse_str(&track_id)
                .map_err(|error| JsValue::from_str(&format!("invalid track id: {error}")))?,
        );
        self.renderer
            .select_clip_at_time(track_id, time_ms)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn request_node_range(
        &mut self,
        node_id: String,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<(), JsValue> {
        let node_id = NodeId(
            uuid::Uuid::parse_str(&node_id)
                .map_err(|error| JsValue::from_str(&format!("invalid node id: {error}")))?,
        );
        self.renderer
            .request_node_range(node_id, start_ms, end_ms)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn request_create_node(
        &mut self,
        node_id: String,
        parent_id: Option<String>,
        level: String,
        name: String,
        start_ms: u64,
        end_ms: u64,
        beat_type: Option<String>,
    ) -> Result<(), JsValue> {
        let node_id = parse_node_id(&node_id)?;
        let parent_id = parent_id.as_deref().map(parse_node_id).transpose()?;
        let level = parse_story_level(&level)?;
        let beat_type = beat_type.as_deref().map(parse_beat_type).transpose()?;

        self.renderer
            .request_create_node(node_id, parent_id, level, name, start_ms, end_ms, beat_type)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn request_split_node(&mut self, node_id: String, at_ms: u64) -> Result<(), JsValue> {
        let node_id = parse_node_id(&node_id)?;
        self.renderer
            .request_split_node(node_id, at_ms)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn request_delete_node(&mut self, node_id: String) -> Result<(), JsValue> {
        let node_id = parse_node_id(&node_id)?;
        self.renderer
            .request_delete_node(node_id)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn set_viewport(&mut self, start_ms: u64, end_ms: u64) -> Result<(), JsValue> {
        self.renderer
            .set_viewport(start_ms, end_ms)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn set_playhead(&mut self, position_ms: u64) -> Result<(), JsValue> {
        self.renderer
            .set_playhead(position_ms)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn playhead(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.renderer.playhead())
            .map_err(|error| JsValue::from_str(&format!("invalid renderer playhead: {error}")))
    }

    pub fn relationship_curves(&self) -> Result<JsValue, JsValue> {
        let curves = self
            .renderer
            .relationship_curves()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        serde_wasm_bindgen::to_value(&curves).map_err(|error| {
            JsValue::from_str(&format!("invalid relationship curve projection: {error}"))
        })
    }

    pub fn pan_viewport(&mut self, delta_ms: i64) {
        self.renderer.pan_viewport(delta_ms);
    }

    pub fn zoom_viewport_around(&mut self, center_ms: u64, factor: f32) -> Result<(), JsValue> {
        self.renderer
            .zoom_viewport_around(center_ms, factor)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn viewport(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.renderer.viewport())
            .map_err(|error| JsValue::from_str(&format!("invalid renderer viewport: {error}")))
    }

    pub fn drain_commands(&mut self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.renderer.drain_commands())
            .map_err(|error| JsValue::from_str(&format!("invalid renderer commands: {error}")))
    }

    pub fn clip_count(&self) -> usize {
        self.renderer.projection_clip_count()
    }
}

impl Default for WasmTimelineRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_node_id(value: &str) -> Result<NodeId, JsValue> {
    Ok(NodeId(uuid::Uuid::parse_str(value).map_err(|error| {
        JsValue::from_str(&format!("invalid node id: {error}"))
    })?))
}

fn parse_story_level(value: &str) -> Result<StoryLevel, JsValue> {
    match value {
        "Premise" | "premise" => Ok(StoryLevel::Premise),
        "Act" | "act" => Ok(StoryLevel::Act),
        "Sequence" | "sequence" => Ok(StoryLevel::Sequence),
        "Scene" | "scene" => Ok(StoryLevel::Scene),
        "Beat" | "beat" => Ok(StoryLevel::Beat),
        _ => Err(JsValue::from_str("invalid story level")),
    }
}

fn parse_beat_type(value: &str) -> Result<BeatType, JsValue> {
    match value {
        "Setup" | "setup" => Ok(BeatType::Setup),
        "Complication" | "complication" => Ok(BeatType::Complication),
        "Escalation" | "escalation" => Ok(BeatType::Escalation),
        "Climax" | "climax" => Ok(BeatType::Climax),
        "Resolution" | "resolution" => Ok(BeatType::Resolution),
        "Payoff" | "payoff" => Ok(BeatType::Payoff),
        "Callback" | "callback" => Ok(BeatType::Callback),
        custom if !custom.is_empty() => Ok(BeatType::Custom(custom.to_string())),
        _ => Err(JsValue::from_str("invalid beat type")),
    }
}
