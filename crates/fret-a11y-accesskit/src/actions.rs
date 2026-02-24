use accesskit::{Action, ActionRequest, TextPosition, TreeId};
use fret_core::{SemanticsRole, SemanticsSnapshot};

use crate::ids::{from_accesskit_id, parent_from_synthetic_id};

pub fn focus_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Focus {
        return None;
    }
    if req.target_tree != TreeId::ROOT {
        return None;
    }
    parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))
}

pub fn invoke_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Click {
        return None;
    }
    if req.target_tree != TreeId::ROOT {
        return None;
    }
    parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepperAction {
    Decrement,
    Increment,
}

pub fn stepper_target_from_action(
    req: &ActionRequest,
) -> Option<(fret_core::NodeId, StepperAction)> {
    if req.target_tree != TreeId::ROOT {
        return None;
    }

    let action = match req.action {
        Action::Decrement => StepperAction::Decrement,
        Action::Increment => StepperAction::Increment,
        _ => return None,
    };

    let target =
        parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))?;
    Some((target, action))
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetValueData {
    Text(String),
    Numeric(f64),
}

pub fn set_value_from_action(req: &ActionRequest) -> Option<(fret_core::NodeId, SetValueData)> {
    if req.action != Action::SetValue {
        return None;
    }
    if req.target_tree != TreeId::ROOT {
        return None;
    }

    let target =
        parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))?;
    let data = req.data.as_ref()?;
    match data {
        accesskit::ActionData::Value(v) => Some((target, SetValueData::Text(v.to_string()))),
        accesskit::ActionData::NumericValue(v) => Some((target, SetValueData::Numeric(*v))),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollByData {
    pub dx: f64,
    pub dy: f64,
}

fn scroll_delta_for_unit(unit: accesskit::ScrollUnit, viewport_extent: f32) -> f64 {
    match unit {
        accesskit::ScrollUnit::Item => 40.0,
        accesskit::ScrollUnit::Page => {
            // Best-effort approximation: a page scroll is usually close to the viewport size.
            //
            // Use a small overlap so repeated PageDown/PageUp keeps context.
            ((viewport_extent as f64) * 0.9).max(40.0)
        }
    }
}

pub fn scroll_by_from_action(
    req: &ActionRequest,
    snapshot: &SemanticsSnapshot,
) -> Option<(fret_core::NodeId, ScrollByData)> {
    if req.target_tree != TreeId::ROOT {
        return None;
    }

    let target =
        parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))?;
    let node = snapshot.nodes.iter().find(|n| n.id == target)?;

    let unit = match &req.data {
        Some(accesskit::ActionData::ScrollUnit(unit)) => *unit,
        _ => accesskit::ScrollUnit::Item,
    };

    match req.action {
        Action::ScrollDown => {
            let dy = scroll_delta_for_unit(unit, node.bounds.size.height.0);
            Some((target, ScrollByData { dx: 0.0, dy }))
        }
        Action::ScrollUp => {
            let dy = -scroll_delta_for_unit(unit, node.bounds.size.height.0);
            Some((target, ScrollByData { dx: 0.0, dy }))
        }
        Action::ScrollLeft => {
            let dx = -scroll_delta_for_unit(unit, node.bounds.size.width.0);
            Some((target, ScrollByData { dx, dy: 0.0 }))
        }
        Action::ScrollRight => {
            let dx = scroll_delta_for_unit(unit, node.bounds.size.width.0);
            Some((target, ScrollByData { dx, dy: 0.0 }))
        }
        Action::SetScrollOffset => {
            let accesskit::ActionData::SetScrollOffset(p) = req.data.as_ref()? else {
                return None;
            };

            let mut dx = 0.0;
            let mut dy = 0.0;

            if node.extra.scroll.x.is_some() || node.extra.scroll.x_max.is_some() {
                let cur = node.extra.scroll.x.unwrap_or(0.0);
                dx = p.x - cur;
            }
            if node.extra.scroll.y.is_some() || node.extra.scroll.y_max.is_some() {
                let cur = node.extra.scroll.y.unwrap_or(0.0);
                dy = p.y - cur;
            }

            Some((target, ScrollByData { dx, dy }))
        }
        _ => None,
    }
}

pub fn replace_selected_text_from_action(
    req: &ActionRequest,
    snapshot: &SemanticsSnapshot,
) -> Option<(fret_core::NodeId, String)> {
    if req.action != Action::ReplaceSelectedText {
        return None;
    }
    if req.target_tree != TreeId::ROOT {
        return None;
    }

    let target =
        parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))?;
    let node = snapshot.nodes.iter().find(|n| n.id == target)?;
    if node.role != SemanticsRole::TextField || node.value.is_none() {
        return None;
    }
    if node.text_composition.is_some() {
        return None;
    }

    let data = req.data.as_ref()?;
    match data {
        accesskit::ActionData::Value(v) => Some((target, v.to_string())),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTextSelectionData {
    pub anchor: u32,
    pub focus: u32,
}

fn character_index_to_byte_offset(value: &str, character_index: usize) -> u32 {
    value
        .chars()
        .take(character_index)
        .fold(0u32, |acc, ch| acc.saturating_add(ch.len_utf8() as u32))
        .min(value.len() as u32)
}

fn text_selection_target_from_position(pos: &TextPosition) -> Option<fret_core::NodeId> {
    parent_from_synthetic_id(pos.node).or_else(|| from_accesskit_id(pos.node))
}

pub fn set_text_selection_from_action(
    req: &ActionRequest,
    snapshot: &SemanticsSnapshot,
) -> Option<(fret_core::NodeId, SetTextSelectionData)> {
    if req.action != Action::SetTextSelection {
        return None;
    }
    if req.target_tree != TreeId::ROOT {
        return None;
    }

    let target =
        parent_from_synthetic_id(req.target_node).or_else(|| from_accesskit_id(req.target_node))?;
    let data = req.data.as_ref()?;
    let accesskit::ActionData::SetTextSelection(sel) = data else {
        return None;
    };

    let node = snapshot.nodes.iter().find(|n| n.id == target)?;
    let value = node.value.as_deref()?;
    if node.text_composition.is_some() {
        return None;
    }

    let anchor_target = text_selection_target_from_position(&sel.anchor)?;
    let focus_target = text_selection_target_from_position(&sel.focus)?;
    if anchor_target != target || focus_target != target {
        return None;
    }

    let anchor = character_index_to_byte_offset(value, sel.anchor.character_index);
    let focus = character_index_to_byte_offset(value, sel.focus.character_index);

    Some((target, SetTextSelectionData { anchor, focus }))
}
