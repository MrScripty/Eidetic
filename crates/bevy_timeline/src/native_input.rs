use bevy::prelude::{ButtonInput, KeyCode, MouseButton, Query, Res, Window, World};
use bevy::window::PrimaryWindow;
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};

use crate::native_render::{
    TimelineNativeProjectionState, TimelineNativeWindowControl, native_track_height_px,
    nudge_timeline_native_playhead, pan_timeline_native_viewport, zoom_timeline_native_viewport,
};
use crate::{TimelineViewportGeometry, TimelineViewportPoint};

pub(crate) fn emit_timeline_native_click_selection(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, bevy::prelude::With<PrimaryWindow>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(buttons) = buttons else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let geometry = TimelineViewportGeometry::new(
        window.width().max(1.0) as u32,
        window.height().max(1.0) as u32,
        native_track_height_px(),
    );
    let point = TimelineViewportPoint::new(
        cursor_position.x.max(0.0) as u32,
        (window.height() - cursor_position.y).max(0.0) as u32,
    );
    let _ = crate::native_command::emit_timeline_native_clip_selection(
        &control,
        projection,
        projection_state.viewport,
        geometry,
        point,
    );
}

pub(crate) fn emit_timeline_native_selected_relationship_create(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, bevy::prelude::With<PrimaryWindow>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(buttons) = buttons else {
        return;
    };
    if !timeline_native_control_modifier_pressed(&keys)
        || !timeline_native_shift_modifier_pressed(&keys)
        || !buttons.just_pressed(MouseButton::Left)
    {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let geometry = TimelineViewportGeometry::new(
        window.width().max(1.0) as u32,
        window.height().max(1.0) as u32,
        native_track_height_px(),
    );
    let point = TimelineViewportPoint::new(
        cursor_position.x.max(0.0) as u32,
        (window.height() - cursor_position.y).max(0.0) as u32,
    );
    let Ok(Some(to_node_id)) = crate::hit_test_projection_clip_at_point(
        projection,
        projection_state.viewport,
        geometry,
        point,
    ) else {
        return;
    };
    let _ = crate::native_command::emit_timeline_native_selected_create_relationship_request(
        &control,
        projection,
        RelationshipId::new(),
        to_node_id,
        RelationshipType::Thematic,
    );
}

pub(crate) fn emit_timeline_native_selected_delete(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !keys.just_pressed(KeyCode::Delete) && !keys.just_pressed(KeyCode::Backspace) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let _ =
        crate::native_command::emit_timeline_native_selected_delete_request(&control, projection);
}

pub(crate) fn emit_timeline_native_selected_split(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let _ = crate::native_command::emit_timeline_native_selected_split_request(
        &control,
        projection,
        projection_state.playhead.position_ms,
        NodeId::new(),
        NodeId::new(),
    );
}

pub(crate) fn emit_timeline_native_selected_create_child(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !timeline_native_control_modifier_pressed(&keys) || !keys.just_pressed(KeyCode::KeyN) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let _ = crate::native_command::emit_timeline_native_selected_create_child_from_parent_request(
        &control,
        projection,
        NodeId::new(),
    );
}

pub(crate) fn emit_timeline_native_selected_nudge(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !timeline_native_control_modifier_pressed(&keys) {
        return;
    }
    if timeline_native_shift_modifier_pressed(&keys) || timeline_native_alt_modifier_pressed(&keys)
    {
        return;
    }
    let nudge_left = keys.just_pressed(KeyCode::ArrowLeft);
    let nudge_right = keys.just_pressed(KeyCode::ArrowRight);
    if !nudge_left && !nudge_right {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let nudge_step_ms = (projection_state.viewport.width_ms() / 100).max(1) as i64;
    let delta_ms = if nudge_left {
        -nudge_step_ms
    } else {
        nudge_step_ms
    };
    let _ = crate::native_command::emit_timeline_native_selected_nudge_request(
        &control, projection, delta_ms,
    );
}

pub(crate) fn emit_timeline_native_selected_resize(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !timeline_native_control_modifier_pressed(&keys) {
        return;
    }
    let resize_left = keys.just_pressed(KeyCode::ArrowLeft);
    let resize_right = keys.just_pressed(KeyCode::ArrowRight);
    if !resize_left && !resize_right {
        return;
    }
    let edge = if timeline_native_shift_modifier_pressed(&keys) {
        crate::native_command::TimelineNativeResizeEdge::Start
    } else if timeline_native_alt_modifier_pressed(&keys) {
        crate::native_command::TimelineNativeResizeEdge::End
    } else {
        return;
    };
    let Some(control) = control else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let resize_step_ms = (projection_state.viewport.width_ms() / 100).max(1) as i64;
    let delta_ms = if resize_left {
        -resize_step_ms
    } else {
        resize_step_ms
    };
    let _ = crate::native_command::emit_timeline_native_selected_resize_request(
        &control, projection, edge, delta_ms,
    );
}

pub(crate) fn navigate_timeline_native_viewport(world: &mut World) {
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    if timeline_native_control_modifier_pressed(keys) {
        return;
    }
    let pan_left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft);
    let pan_right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight);
    let zoom_out = keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Minus);
    let zoom_in = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Equal);

    let viewport_width_ms = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.viewport.width_ms())
        .unwrap_or(1);
    let pan_step_ms = (viewport_width_ms / 10).max(1) as i64;

    if pan_left {
        let _ = pan_timeline_native_viewport(world, -pan_step_ms);
    }
    if pan_right {
        let _ = pan_timeline_native_viewport(world, pan_step_ms);
    }
    if zoom_out {
        let _ = zoom_timeline_native_viewport(world, 0.8);
    }
    if zoom_in {
        let _ = zoom_timeline_native_viewport(world, 1.25);
    }
}

pub(crate) fn navigate_timeline_native_playhead(world: &mut World) {
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    let nudge_left = keys.just_pressed(KeyCode::KeyJ);
    let nudge_right = keys.just_pressed(KeyCode::KeyL);

    if !nudge_left && !nudge_right {
        return;
    }

    let (viewport_width_ms, projection, control) = {
        let Some(projection_state) = world.get_resource::<TimelineNativeProjectionState>() else {
            return;
        };
        let projection = projection_state.projection.clone();
        let control = world
            .get_resource::<TimelineNativeWindowControl>()
            .map(|control| control.clone());
        (projection_state.viewport.width_ms(), projection, control)
    };
    let Some(projection) = projection else {
        return;
    };
    let Some(control) = control else {
        return;
    };

    let nudge_step_ms = (viewport_width_ms / 100).max(1) as i64;
    let playhead = if nudge_left {
        nudge_timeline_native_playhead(world, -nudge_step_ms)
    } else {
        nudge_timeline_native_playhead(world, nudge_step_ms)
    };
    let _ = crate::native_command::emit_timeline_native_playhead_request(
        &control,
        &projection,
        playhead.position_ms,
    );
}

fn timeline_native_control_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}

fn timeline_native_shift_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn timeline_native_alt_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
}
