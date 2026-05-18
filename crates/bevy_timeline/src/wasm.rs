use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;
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
