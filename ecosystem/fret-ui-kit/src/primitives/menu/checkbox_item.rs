//! Menu checkbox item helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuCheckboxItem` is a checkable menu item. In Fret we model the checked state as a
//! `Model<bool>` and provide small wiring helpers that wrappers can call from within a pressable
//! closure.

use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::{ElementContext, UiHost};
use std::sync::Arc;

/// Wire checkbox-item activation to toggle `checked`.
///
/// Intended to be called inside a pressable closure for the checkbox item.
#[track_caller]
pub fn wire_toggle_on_activate<H: UiHost>(cx: &mut ElementContext<'_, H>, checked: Model<bool>) {
    let handler: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&checked, |v| *v = !*v);
    });
    cx.pressable_add_on_activate(handler);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, MouseButton, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId,
        SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService, UiServices,
    };
    use fret_runtime::FrameId;
    use fret_ui::element::{
        ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps, SizeStyle,
    };
    use fret_ui::tree::UiTree;

    use crate::declarative::model_watch::ModelWatchExt as _;
    use crate::primitives::menu::item;

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
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        checked: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let checked_now = cx.watch_model(&checked).copied().unwrap_or(false);
                vec![
                    cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    ),
                    cx.pressable(
                        PressableProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(20.0)),
                                    top: Some(Px(20.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(120.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            enabled: true,
                            focusable: true,
                            a11y: item::menu_item_checkbox_a11y(
                                Some(Arc::from("Bold")),
                                checked_now,
                            ),
                            ..Default::default()
                        },
                        move |cx, _st| {
                            wire_toggle_on_activate(cx, checked.clone());
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    ),
                ]
            });

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn checkbox_item_toggles_model_and_emits_checked_flag() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let checked = app.models_mut().insert(false);

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            checked.clone(),
        );

        let pressable_node = ui.children(root)[1];
        let pressable_bounds = ui
            .debug_node_bounds(pressable_node)
            .expect("pressable bounds");
        assert!(
            pressable_bounds.size.width.0 > 0.0 && pressable_bounds.size.height.0 > 0.0,
            "expected non-zero pressable bounds, got {pressable_bounds:?}"
        );
        let click = Point::new(
            Px(pressable_bounds.origin.x.0 + 1.0),
            Px(pressable_bounds.origin.y.0 + 1.0),
        );
        let hit = ui.debug_hit_test(click);
        let hit_node = hit.hit.expect("expected click to hit a node");
        let path = ui.debug_node_path(hit_node);
        assert!(
            path.contains(&pressable_node),
            "expected hit to be inside pressable subtree; hit={hit:?} path={path:?} pressable={pressable_node:?}"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );

        assert_eq!(app.models().get_copied(&checked), Some(true));

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            checked.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItemCheckbox)
            .expect("checkbox item node");
        assert_eq!(node.flags.checked, Some(true));
    }
}
