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

pub use fret_ui::action::{
    ActionCx, DismissReason, DismissRequestCx, OnDismissRequest, UiActionHost,
};
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
    f: impl Fn(&mut dyn UiActionHost, ActionCx, &mut DismissRequestCx) + 'static,
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

/// Resolve `DismissableLayerBranch` roots (Radix outcome) into `NodeId`s for the outside-press
/// observer pass (ADR 0069), without implicitly treating a trigger as a branch.
///
/// This is useful for non-click-through overlays that also disable outside pointer interactions
/// (menu-like `modal=true` outcomes): the trigger should be treated as "outside" so a press on the
/// trigger can close the overlay without activating the underlay.
pub fn resolve_branch_nodes_for_elements<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    branches: &[GlobalElementId],
) -> Vec<NodeId> {
    let mut out: Vec<NodeId> = branches
        .iter()
        .filter_map(|branch| fret_ui::elements::node_for_element(app, window, *branch))
        .collect();
    let mut seen: HashSet<NodeId> = HashSet::with_capacity(out.len());
    out.retain(|id| seen.insert(*id));
    out
}

/// Resolve dismissable layer branch roots for a popover-like overlay request.
///
/// This matches Radix semantics used by menu/popover recipes:
///
/// - Click-through overlays treat the trigger as an implicit branch so a trigger click doesn't
///   first dismiss the overlay and then immediately re-open it.
/// - Menu-like overlays that disable outside pointer interactions should *not* treat the trigger
///   as a branch: the trigger press must be considered "outside" so it can close the overlay
///   without activating the underlay.
pub fn resolve_branch_nodes_for_popover_request<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    trigger: GlobalElementId,
    branches: &[GlobalElementId],
    disable_outside_pointer_events: bool,
) -> Vec<NodeId> {
    if disable_outside_pointer_events {
        resolve_branch_nodes_for_elements(app, window, branches)
    } else {
        resolve_branch_nodes_for_trigger_and_elements(app, window, trigger, branches)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_ui::element::{LayoutStyle, Length, PressableProps, SemanticsProps};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn resolve_branch_nodes_dedupes_and_preserves_order() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let b = bounds();

        let mut trigger: Option<GlobalElementId> = None;
        let mut branch_a: Option<GlobalElementId> = None;
        let mut branch_b: Option<GlobalElementId> = None;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(10.0));
                        layout.size.height = Length::Px(Px(10.0));
                        layout
                    },
                    focusable: true,
                    ..Default::default()
                };

                vec![
                    cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                        trigger = Some(id);
                        Vec::new()
                    }),
                    cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                        branch_a = Some(id);
                        Vec::new()
                    }),
                    cx.pressable_with_id(props, |_cx, _st, id| {
                        branch_b = Some(id);
                        Vec::new()
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let trigger = trigger.expect("trigger id");
        let branch_a = branch_a.expect("branch a id");
        let branch_b = branch_b.expect("branch b id");

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        let branch_a_node =
            fret_ui::elements::node_for_element(&mut app, window, branch_a).expect("branch a node");
        let branch_b_node =
            fret_ui::elements::node_for_element(&mut app, window, branch_b).expect("branch b node");

        let out = resolve_branch_nodes_for_trigger_and_elements(
            &mut app,
            window,
            trigger,
            &[branch_a, trigger, branch_b, branch_a],
        );

        assert_eq!(out, vec![trigger_node, branch_a_node, branch_b_node]);
    }

    #[test]
    fn focus_inside_layer_or_branch_is_treated_as_inside() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let b = bounds();

        let mut layer_root: Option<GlobalElementId> = None;
        let mut branch_root: Option<GlobalElementId> = None;
        let mut in_layer: Option<GlobalElementId> = None;
        let mut in_branch: Option<GlobalElementId> = None;
        let mut outside: Option<GlobalElementId> = None;

        let focusable = PressableProps {
            layout: LayoutStyle::default(),
            focusable: true,
            ..Default::default()
        };

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                vec![
                    cx.semantics_with_id(SemanticsProps::default(), |cx, id| {
                        layer_root = Some(id);
                        vec![cx.pressable_with_id(focusable.clone(), |_cx, _st, id| {
                            in_layer = Some(id);
                            Vec::new()
                        })]
                    }),
                    cx.semantics_with_id(SemanticsProps::default(), |cx, id| {
                        branch_root = Some(id);
                        vec![cx.pressable_with_id(focusable.clone(), |_cx, _st, id| {
                            in_branch = Some(id);
                            Vec::new()
                        })]
                    }),
                    cx.pressable_with_id(focusable, |_cx, _st, id| {
                        outside = Some(id);
                        Vec::new()
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let layer_root = layer_root.expect("layer root");
        let branch_root = branch_root.expect("branch root");
        let in_layer = in_layer.expect("in layer");
        let in_branch = in_branch.expect("in branch");
        let outside = outside.expect("outside");

        let layer_root_node =
            fret_ui::elements::node_for_element(&mut app, window, layer_root).expect("layer node");
        let branch_root_node = fret_ui::elements::node_for_element(&mut app, window, branch_root)
            .expect("branch node");
        let in_layer_node =
            fret_ui::elements::node_for_element(&mut app, window, in_layer).expect("in layer node");
        let in_branch_node = fret_ui::elements::node_for_element(&mut app, window, in_branch)
            .expect("in branch node");
        let outside_node =
            fret_ui::elements::node_for_element(&mut app, window, outside).expect("outside node");

        assert!(focus_is_inside_layer_or_branches(
            &ui,
            layer_root_node,
            in_layer_node,
            &[branch_root_node]
        ));
        assert!(focus_is_inside_layer_or_branches(
            &ui,
            layer_root_node,
            in_branch_node,
            &[branch_root_node]
        ));
        assert!(!focus_is_inside_layer_or_branches(
            &ui,
            layer_root_node,
            outside_node,
            &[branch_root_node]
        ));
    }
}
