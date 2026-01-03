//! DismissableLayer (Radix-aligned outcomes).
//!
//! In the DOM, Radix's DismissableLayer composes Escape and outside-interaction dismissal hooks.
//! In Fret, the runtime substrate provides those mechanisms via:
//!
//! - Escape routing: `fret-ui` event dispatch.
//! - Outside-press observer pass: ADR 0069 (observer phase pointer events).
//!
//! This module provides a stable, Radix-named primitive surface for component-layer policy.

use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, Rect, UiServices};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost, UiTree};

pub use fret_ui::action::{ActionCx, DismissReason, OnDismissRequest, UiActionHost};
pub use fret_ui::action::{OnDismissiblePointerMove, PointerMoveCx};

/// Render a full-window dismissable root that provides Escape + outside-press dismissal hooks.
///
/// This is a Radix-aligned naming alias for `render_dismissible_root_with_hooks`.
#[allow(clippy::too_many_arguments)]
pub fn render_dismissable_root_with_hooks<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> fret_core::NodeId {
    crate::declarative::dismissible::render_dismissible_root_with_hooks(
        ui, app, services, window, bounds, root_name, render,
    )
}

/// Installs an `on_dismiss_request` handler for the current dismissable root.
///
/// This is a naming-aligned wrapper around `ElementContext::dismissible_on_dismiss_request`.
pub fn on_dismiss_request<H: UiHost>(cx: &mut ElementContext<'_, H>, handler: OnDismissRequest) {
    cx.dismissible_on_dismiss_request(handler);
}

/// Installs an `on_pointer_move` observer for the current dismissable root.
///
/// This is intended for overlay policy code (e.g. submenu safe-hover corridors) that needs pointer
/// movement even when the overlay content is click-through.
pub fn on_pointer_move<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    handler: OnDismissiblePointerMove,
) {
    cx.dismissible_on_pointer_move(handler);
}

/// Convenience builder for an `OnDismissRequest` handler.
pub fn handler(
    f: impl Fn(&mut dyn UiActionHost, ActionCx, DismissReason) + 'static,
) -> OnDismissRequest {
    Arc::new(f)
}

/// Convenience builder for an `OnDismissiblePointerMove` handler.
pub fn pointer_move_handler(
    f: impl Fn(&mut dyn UiActionHost, ActionCx, PointerMoveCx) -> bool + 'static,
) -> OnDismissiblePointerMove {
    Arc::new(f)
}

/// Resolve `DismissableLayerBranch` roots (Radix outcome) into `NodeId`s for the outside-press
/// observer pass (ADR 0069).
///
/// Notes:
/// - Missing nodes are ignored (e.g. branch element not mounted yet).
/// - Duplicates are removed while preserving first-seen order.
pub fn resolve_branch_nodes_for_trigger_and_elements<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    trigger: GlobalElementId,
    branches: &[GlobalElementId],
) -> Vec<NodeId> {
    let mut out: Vec<NodeId> = Vec::with_capacity(1 + branches.len());
    if let Some(node) = fret_ui::elements::node_for_element(app, window, trigger) {
        out.push(node);
    }
    out.extend(
        branches
            .iter()
            .filter_map(|branch| fret_ui::elements::node_for_element(app, window, *branch)),
    );
    let mut seen: HashSet<NodeId> = HashSet::with_capacity(out.len());
    out.retain(|id| seen.insert(*id));
    out
}

/// Returns true if `focus` is inside the dismissable layer subtree, or inside any branch subtree.
pub fn focus_is_inside_layer_or_branches<H: UiHost>(
    ui: &UiTree<H>,
    layer_root: NodeId,
    focus: NodeId,
    branch_roots: &[NodeId],
) -> bool {
    ui.is_descendant(layer_root, focus)
        || branch_roots
            .iter()
            .copied()
            .any(|branch| ui.is_descendant(branch, focus))
}

/// Returns true if focus changed since `last_focus` and is now outside the layer + branches.
///
/// This is the Radix `onFocusOutside` outcome, expressed using Fret overlay orchestration.
pub fn should_dismiss_on_focus_outside<H: UiHost>(
    ui: &UiTree<H>,
    layer_root: NodeId,
    focus_now: Option<NodeId>,
    last_focus: Option<NodeId>,
    branch_roots: &[NodeId],
) -> bool {
    let Some(focus) = focus_now else {
        return false;
    };
    if last_focus == Some(focus) {
        return false;
    }
    !focus_is_inside_layer_or_branches(ui, layer_root, focus, branch_roots)
}
