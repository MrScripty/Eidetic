use eidetic_core::contracts::{BibleGraphNodeId, BibleRenderGraphProjection};
use wasm_bindgen::prelude::*;

use crate::BibleGraphRendererApp;

#[wasm_bindgen]
pub struct WasmBibleGraphRenderer {
    renderer: BibleGraphRendererApp,
}

#[wasm_bindgen]
impl WasmBibleGraphRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            renderer: BibleGraphRendererApp::new(),
        }
    }

    pub fn set_projection(&mut self, projection: JsValue) -> Result<(), JsValue> {
        let projection: BibleRenderGraphProjection = serde_wasm_bindgen::from_value(projection)
            .map_err(|error| {
                JsValue::from_str(&format!("invalid bible graph projection: {error}"))
            })?;
        self.renderer.set_projection(projection);
        Ok(())
    }

    pub fn select_node(&mut self, node_id: String) -> Result<(), JsValue> {
        let node_id = parse_node_id(node_id)?;
        self.renderer
            .select_node(node_id)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn inspect_node(&mut self, node_id: String) -> Result<(), JsValue> {
        let node_id = parse_node_id(node_id)?;
        self.renderer
            .inspect_node(node_id)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn edge_ids_for_node(&self, node_id: String) -> Result<JsValue, JsValue> {
        let node_id = parse_node_id(node_id)?;
        let edge_ids = self
            .renderer
            .edge_ids_for_node(&node_id)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        serde_wasm_bindgen::to_value(&edge_ids)
            .map_err(|error| JsValue::from_str(&format!("invalid bible graph edges: {error}")))
    }

    pub fn drain_commands(&mut self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.renderer.drain_commands())
            .map_err(|error| JsValue::from_str(&format!("invalid renderer commands: {error}")))
    }

    pub fn node_count(&self) -> usize {
        self.renderer.projection_node_count()
    }
}

impl Default for WasmBibleGraphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_node_id(value: String) -> Result<BibleGraphNodeId, JsValue> {
    BibleGraphNodeId::new(value).map_err(|error| JsValue::from_str(&error.to_string()))
}
