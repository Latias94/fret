//! Menu radio group helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuRadioGroup` stores the selected `value` and `MenuRadioItem` writes to it on
//! select. In Fret we model the group value as `Model<Option<Arc<str>>>`.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::{ElementContext, UiHost};

/// Returns whether `value` is the currently selected radio group value.
pub fn is_selected(selected: Option<&Arc<str>>, value: &Arc<str>) -> bool {
    selected.is_some_and(|cur| cur.as_ref() == value.as_ref())
}

/// Wire radio-item activation to set the radio group's selected value.
///
/// Intended to be called inside a pressable closure for the radio item.
#[track_caller]
pub fn wire_select_on_activate<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group_value: Model<Option<Arc<str>>>,
    value: Arc<str>,
) {
    let handler: OnActivate = Arc::new(move |host, _acx, _reason| {
        let next = Some(value.clone());
        let _ = host.models_mut().update(&group_value, |v| *v = next);
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        group: Model<Option<Arc<str>>>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let selected = cx.watch_model(&group).cloned().flatten();
                let a = Arc::from("a");
                let b = Arc::from("b");
                let group_for_a = group.clone();
                let group_for_b = group.clone();

                let a_checked = is_selected(selected.as_ref(), &a);
                let b_checked = is_selected(selected.as_ref(), &b);

                let item_layout = |y: Px| LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(Px(20.0)),
                        top: Some(y),
                        ..Default::default()
                    },
                    size: SizeStyle {
                        width: Length::Px(Px(120.0)),
                        height: Length::Px(Px(32.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };

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
                            layout: item_layout(Px(20.0)),
                            enabled: true,
                            focusable: true,
                            a11y: item::menu_item_radio_a11y(Some(Arc::from("A")), a_checked),
                            ..Default::default()
                        },
                        move |cx, _st| {
                            wire_select_on_activate(cx, group_for_a.clone(), a.clone());
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    ),
                    cx.pressable(
                        PressableProps {
                            layout: item_layout(Px(60.0)),
                            enabled: true,
                            focusable: true,
                            a11y: item::menu_item_radio_a11y(Some(Arc::from("B")), b_checked),
                            ..Default::default()
                        },
                        move |cx, _st| {
                            wire_select_on_activate(cx, group_for_b.clone(), b.clone());
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
    fn radio_item_sets_group_value_and_emits_checked_flag() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let group = app.models_mut().insert(None::<Arc<str>>);

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            group.clone(),
        );

        let pressable_a = ui.children(root)[1];
        let bounds_a = ui.debug_node_bounds(pressable_a).expect("A bounds");
        assert!(
            bounds_a.size.width.0 > 0.0 && bounds_a.size.height.0 > 0.0,
            "expected non-zero A bounds, got {bounds_a:?}"
        );
        let click_a = Point::new(Px(bounds_a.origin.x.0 + 1.0), Px(bounds_a.origin.y.0 + 1.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click_a,
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
                position: click_a,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        assert_eq!(
            app.models()
                .read(&group, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("a")
        );

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            group.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let a_node = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItemRadio && n.label.as_deref() == Some("A"))
            .expect("radio A node");
        assert_eq!(a_node.flags.checked, Some(true));

        let pressable_b = ui.children(root)[2];
        let bounds_b = ui.debug_node_bounds(pressable_b).expect("B bounds");
        assert!(
            bounds_b.size.width.0 > 0.0 && bounds_b.size.height.0 > 0.0,
            "expected non-zero B bounds, got {bounds_b:?}"
        );
        let click_b = Point::new(Px(bounds_b.origin.x.0 + 1.0), Px(bounds_b.origin.y.0 + 1.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click_b,
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
                position: click_b,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        assert_eq!(
            app.models()
                .read(&group, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("b")
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            group.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let b_node = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItemRadio && n.label.as_deref() == Some("B"))
            .expect("radio B node");
        assert_eq!(b_node.flags.checked, Some(true));
    }
}
