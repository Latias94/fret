use fret_core::{MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerUpCx, UiPointerActionHost};

use super::super::super::model_owner::ProofCollectionModelOwner;

pub(super) fn proof_collection_browser_scope_context_menu_anchor_from_up(
    up: &PointerUpCx,
) -> Option<Point> {
    if up.button != MouseButton::Right || !up.is_click {
        return None;
    }
    if up.down_hit_pressable_target.is_some() || up.down_hit_pressable_target_in_descendant_subtree
    {
        return None;
    }

    Some(up.position_window.unwrap_or(up.position))
}

pub(super) fn publish_collection_browser_scope_context_menu_anchor(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    context_menu_anchor_model: &Model<Option<Point>>,
    up: &PointerUpCx,
) -> bool {
    let Some(position) = proof_collection_browser_scope_context_menu_anchor_from_up(up) else {
        return false;
    };

    host.request_focus(acx.target);
    ProofCollectionModelOwner::new(host.models_mut())
        .publish_context_menu_anchor(context_menu_anchor_model, position);
    host.notify(acx);
    true
}

#[cfg(test)]
mod tests;
