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

use crate::{IntoUiElement, collect_children};

pub use fret_ui::element::FocusScopeProps;

/// Convenience helper for building a trapped focus scope (Tab/Shift+Tab loops within the subtree).
#[track_caller]
pub fn focus_trap<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    cx.focus_scope(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        move |cx| {
            let items = f(cx);
            collect_children(cx, items)
        },
    )
}

/// Like `focus_trap`, but also exposes the scope element ID.
#[track_caller]
pub fn focus_trap_with_id<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>, fret_ui::elements::GlobalElementId) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    cx.focus_scope_with_id(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        move |cx, id| {
            let items = f(cx, id);
            collect_children(cx, items)
        },
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
    if let Some(focus) = initial_focus {
        let focus_is_live = ui.live_attached_node_for_element(app, focus).is_some()
            || fret_ui::elements::element_identity_is_live_in_current_frame(app, window, focus);
        if focus_is_live {
            // Element-based initial focus must tolerate rebuild / barrier transitions in the same
            // way close-auto-focus hooks do: request now, then let the authoritative runtime
            // commit boundary resolve the target if direct focus is transiently rejected. A
            // deferred request alone does not satisfy the current-frame containment contract,
            // so keep the pending target but continue to fallback focus if focus did not
            // actually move yet.
            let before = ui.focus();
            ui.request_focus_element(app, focus);
            if ui.focus() != before {
                return true;
            }
            if let Some(node) = ui.live_attached_node_for_element(app, focus)
                && ui.focus() == Some(node)
            {
                return true;
            }
        }
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
        && let Some(trigger_node) = ui.live_attached_node_for_element(app, trigger)
    {
        let trigger_focusable =
            ui.first_focusable_ancestor_including_declarative(app, window, trigger_node)
                == Some(trigger_node);
        if trigger_focusable {
            return Some(trigger_node);
        }

        if let Some(descendant) =
            ui.first_focusable_descendant_including_declarative(app, window, trigger_node)
        {
            return Some(descendant);
        }
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
    use std::cell::Cell;
    use std::rc::Rc;

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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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

    #[test]
    fn resolve_restore_focus_skips_non_focusable_trigger() {
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

        let trigger_props = PressableProps {
            layout: LayoutStyle::default(),
            focusable: false,
            ..Default::default()
        };
        let other_props = PressableProps {
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
                    cx.pressable_with_id(trigger_props, |_cx, _st, id| {
                        trigger = Some(id);
                        Vec::new()
                    }),
                    cx.pressable_with_id(other_props, |_cx, _st, id| {
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
        let other_node =
            fret_ui::elements::node_for_element(&mut app, window, other).expect("other node");

        assert_eq!(
            resolve_restore_focus_node(&ui, &mut app, window, Some(trigger), Some(other_node)),
            Some(other_node)
        );
    }

    #[test]
    fn apply_initial_focus_ignores_removed_element_with_only_last_known_mapping() {
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

        let removed_focus: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let fallback_focus: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let show_removed = Cell::new(true);

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

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                let removed_focus = removed_focus.clone();
                let fallback_focus = fallback_focus.clone();
                fret_ui::declarative::render_root(ui, app, services, window, b, "test", |cx| {
                    let mut out = Vec::new();
                    if show_removed.get() {
                        out.push(cx.keyed("removed", |cx| {
                            cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                                removed_focus.set(Some(id));
                                Vec::new()
                            })
                        }));
                    }
                    out.push(cx.keyed("fallback", |cx| {
                        cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                            fallback_focus.set(Some(id));
                            Vec::new()
                        })
                    }));
                    out
                })
            };

        let root = render_frame(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let removed_focus = removed_focus.get().expect("removed focus id");

        show_removed.set(false);
        app.set_frame_id(fret_runtime::FrameId(app.frame_id().0.saturating_add(1)));
        let root = render_frame(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let fallback_focus = fallback_focus.get().expect("fallback focus id");
        let fallback_node = fret_ui::elements::node_for_element(&mut app, window, fallback_focus)
            .expect("fallback node");

        ui.set_focus(None);
        assert!(apply_initial_focus_for_overlay(
            &mut ui,
            &mut app,
            window,
            root,
            Some(removed_focus)
        ));
        assert_eq!(
            ui.focus(),
            Some(fallback_node),
            "expected initial-focus resolution to ignore a removed element that only still has a last-known node mapping"
        );
    }
}
