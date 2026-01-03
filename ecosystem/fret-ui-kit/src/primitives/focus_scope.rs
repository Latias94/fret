//! FocusScope (Radix-aligned outcomes).
//!
//! In Radix, FocusScope composes focus trapping/looping and (optionally) auto-focus / restore.
//! In Fret, the runtime provides the focus traversal mechanism, and this primitive provides a
//! stable, Radix-named entry point for component-layer policy.

use fret_core::{AppWindowId, NodeId};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{ElementContext, UiHost, UiTree};

pub use fret_ui::element::FocusScopeProps;

/// Convenience helper for building a trapped focus scope (Tab/Shift+Tab loops within the subtree).
#[track_caller]
pub fn focus_trap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.focus_scope(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        f,
    )
}

/// Like `focus_trap`, but also exposes the scope element ID.
#[track_caller]
pub fn focus_trap_with_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>, fret_ui::elements::GlobalElementId) -> Vec<AnyElement>,
) -> AnyElement {
    cx.focus_scope_with_id(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        f,
    )
}

/// Applies a Radix-style "initial focus" policy for an overlay-like focus scope.
///
/// - If `initial_focus` is provided and still resolves to a live node, we focus it.
/// - Otherwise, we fall back to the first focusable descendant within the root.
///
/// Returns `true` when it updates focus.
pub fn apply_initial_focus_for_overlay<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    root: NodeId,
    initial_focus: Option<GlobalElementId>,
) -> bool {
    if let Some(focus) = initial_focus
        && let Some(node) = fret_ui::elements::node_for_element(app, window, focus)
    {
        ui.set_focus(Some(node));
        return true;
    }

    if let Some(node) = ui.first_focusable_descendant_including_declarative(app, window, root) {
        ui.set_focus(Some(node));
        return true;
    }

    false
}

/// Whether focus restoration is allowed for a non-modal overlay closing in a click-through world.
///
/// For non-modal overlays, focus restoration must be conditional (ADR 0069): if focus already moved
/// to an underlay target due to the click-through outside press, we must not override it.
pub fn should_restore_focus_for_non_modal_overlay<H: UiHost>(
    ui: &UiTree<H>,
    layer: UiLayerId,
) -> bool {
    let focus = ui.focus();
    focus.is_none() || focus.is_some_and(|n| ui.node_layer(n) == Some(layer))
}

/// Resolve which node to restore focus to, preferring a trigger element when possible.
///
/// - We prefer resolving `trigger` at restore time to avoid stale `NodeId`s across frames.
/// - If `trigger` is missing or no longer resolves, we can fall back to `restore_focus` as long as
///   it still belongs to some live layer.
pub fn resolve_restore_focus_node<H: UiHost>(
    ui: &UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    trigger: Option<GlobalElementId>,
    restore_focus: Option<NodeId>,
) -> Option<NodeId> {
    if let Some(trigger) = trigger
        && let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger)
    {
        return Some(trigger_node);
    }

    if let Some(node) = restore_focus
        && ui.node_layer(node).is_some()
    {
        return Some(node);
    }

    None
}
