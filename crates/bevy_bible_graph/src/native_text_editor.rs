use bevy::prelude::Vec2;

pub(crate) const NATIVE_NODE_TEXT_EDITOR_TOP_PX: f32 = 64.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_RIGHT_PX: f32 = 16.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_WIDTH_PX: f32 = 340.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_HEIGHT_RATIO: f32 = 0.72;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_FONT_SIZE_PX: f32 = 15.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_LINE_HEIGHT_PX: f32 = 20.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_CHARACTER_WIDTH_PX: f32 = 8.5;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_CARET_WIDTH_PX: f32 = 2.0;
pub(crate) const NATIVE_NODE_TEXT_EDITOR_CARET_HEIGHT_PX: f32 = 18.0;

pub(crate) fn bible_graph_native_text_editor_caret_position(
    text: &str,
    cursor_byte_index: usize,
    _scroll_y: f32,
) -> Vec2 {
    let (line_index, column_index) =
        bible_graph_native_text_editor_line_column(text, cursor_byte_index);
    Vec2::new(
        column_index as f32 * NATIVE_NODE_TEXT_EDITOR_CHARACTER_WIDTH_PX,
        line_index as f32 * NATIVE_NODE_TEXT_EDITOR_LINE_HEIGHT_PX,
    )
}

pub(crate) fn bible_graph_native_text_editor_index_from_position(
    text: &str,
    local_position: Vec2,
) -> usize {
    let target_line = (local_position.y / NATIVE_NODE_TEXT_EDITOR_LINE_HEIGHT_PX)
        .floor()
        .max(0.0) as usize;
    let target_column = (local_position.x / NATIVE_NODE_TEXT_EDITOR_CHARACTER_WIDTH_PX)
        .round()
        .max(0.0) as usize;
    bible_graph_native_text_editor_index_for_line_column(text, target_line, target_column)
}

pub(crate) fn bible_graph_native_text_editor_local_position(
    cursor_position: Vec2,
    viewport_size: Vec2,
    scroll_y: f32,
    padding_px: f32,
) -> Option<Vec2> {
    let editor_height = viewport_size.y * NATIVE_NODE_TEXT_EDITOR_HEIGHT_RATIO;
    let editor_left =
        viewport_size.x - NATIVE_NODE_TEXT_EDITOR_RIGHT_PX - NATIVE_NODE_TEXT_EDITOR_WIDTH_PX;
    let editor_right = viewport_size.x - NATIVE_NODE_TEXT_EDITOR_RIGHT_PX;
    let editor_top = NATIVE_NODE_TEXT_EDITOR_TOP_PX;
    let editor_bottom = editor_top + editor_height;

    if cursor_position.x < editor_left
        || cursor_position.x > editor_right
        || cursor_position.y < editor_top
        || cursor_position.y > editor_bottom
    {
        return None;
    }

    Some(Vec2::new(
        (cursor_position.x - editor_left - padding_px.max(0.0)).max(0.0),
        (cursor_position.y - editor_top - padding_px.max(0.0) + scroll_y).max(0.0),
    ))
}

pub(crate) fn bible_graph_native_text_editor_delete_backward(
    text: &mut String,
    cursor_byte_index: usize,
) -> usize {
    if cursor_byte_index == 0 {
        return 0;
    }
    let delete_from = bible_graph_native_text_editor_move_left(text, cursor_byte_index);
    text.replace_range(delete_from..cursor_byte_index, "");
    delete_from
}

pub(crate) fn bible_graph_native_text_editor_move_left(
    text: &str,
    cursor_byte_index: usize,
) -> usize {
    text[..cursor_byte_index.min(text.len())]
        .char_indices()
        .last()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

pub(crate) fn bible_graph_native_text_editor_move_right(
    text: &str,
    cursor_byte_index: usize,
) -> usize {
    let cursor_byte_index = cursor_byte_index.min(text.len());
    if cursor_byte_index >= text.len() {
        return text.len();
    }
    text[cursor_byte_index..]
        .chars()
        .next()
        .map(|character| cursor_byte_index + character.len_utf8())
        .unwrap_or(text.len())
}

pub(crate) fn bible_graph_native_text_editor_move_vertical(
    text: &str,
    cursor_byte_index: usize,
    direction: isize,
) -> usize {
    let (line_index, column_index) =
        bible_graph_native_text_editor_line_column(text, cursor_byte_index);
    let target_line = if direction < 0 {
        line_index.saturating_sub(1)
    } else {
        line_index.saturating_add(1)
    };
    bible_graph_native_text_editor_index_for_line_column(text, target_line, column_index)
}

fn bible_graph_native_text_editor_line_column(
    text: &str,
    cursor_byte_index: usize,
) -> (usize, usize) {
    let cursor_byte_index = cursor_byte_index.min(text.len());
    let mut line_index = 0;
    let mut column_index = 0;
    for character in text[..cursor_byte_index].chars() {
        if character == '\n' {
            line_index += 1;
            column_index = 0;
        } else {
            column_index += 1;
        }
    }
    (line_index, column_index)
}

fn bible_graph_native_text_editor_index_for_line_column(
    text: &str,
    target_line: usize,
    target_column: usize,
) -> usize {
    let mut line_index = 0;
    let mut column_index = 0;
    for (byte_index, character) in text.char_indices() {
        if line_index == target_line && column_index >= target_column {
            return byte_index;
        }
        if character == '\n' {
            if line_index == target_line {
                return byte_index;
            }
            line_index += 1;
            column_index = 0;
        } else {
            column_index += 1;
        }
    }
    text.len()
}
