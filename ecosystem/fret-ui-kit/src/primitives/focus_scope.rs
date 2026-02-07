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
pub fn focus_trap<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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
pub fn focus_trap_with_id<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>, fret_ui::elements::GlobalElementId) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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

    if let Some(node) =
        ui.first_focusable_descendant_including_declarative_present_only(app, window, root)
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_ui::element::{LayoutStyle, Length, PressableProps};

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

    #[test]
    fn apply_initial_focus_prefers_explicit_element() {
        fn bounds() -> Rect {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(120.0)),
            )
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let b = bounds();

        let mut a: Option<GlobalElementId> = None;
        let mut b_id: Option<GlobalElementId> = None;

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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                vec![
                    cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                        a = Some(id);
                        Vec::new()
                    }),
                    cx.pressable_with_id(props, |_cx, _st, id| {
                        b_id = Some(id);
                        Vec::new()
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let a = a.expect("a id");
        let b_id = b_id.expect("b id");
        let a_node = fret_ui::elements::node_for_element(&mut app, window, a).expect("a node");
        let b_node = fret_ui::elements::node_for_element(&mut app, window, b_id).expect("b node");

        ui.set_focus(None);
        assert!(apply_initial_focus_for_overlay(
            &mut ui,
            &mut app,
            window,
            root,
            Some(b_id)
        ));
        assert_eq!(ui.focus(), Some(b_node));

        ui.set_focus(None);
        assert!(apply_initial_focus_for_overlay(
            &mut ui, &mut app, window, root, None
        ));
        assert_eq!(ui.focus(), Some(a_node));
    }

    #[test]
    fn resolve_restore_focus_prefers_trigger_and_falls_back_to_live_node() {
        fn bounds() -> Rect {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(120.0)),
            )
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let b = bounds();

        let mut trigger: Option<GlobalElementId> = None;
        let mut other: Option<GlobalElementId> = None;

        let props = PressableProps {
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
                    cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                        trigger = Some(id);
                        Vec::new()
                    }),
                    cx.pressable_with_id(props, |_cx, _st, id| {
                        other = Some(id);
                        Vec::new()
                    }),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let trigger = trigger.expect("trigger id");
        let other = other.expect("other id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        let other_node =
            fret_ui::elements::node_for_element(&mut app, window, other).expect("other node");

        assert_eq!(
            resolve_restore_focus_node(&ui, &mut app, window, Some(trigger), Some(other_node)),
            Some(trigger_node)
        );

        assert_eq!(
            resolve_restore_focus_node(&ui, &mut app, window, None, Some(other_node)),
            Some(other_node)
        );
    }
}
