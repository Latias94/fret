use super::*;

use std::cell::Cell;

use crate::element::{AnchoredProps, ContainerProps, LayoutStyle, PressableProps, SizeStyle};
use crate::overlay_placement::{Align, AnchoredPanelLayout, AnchoredPanelOptions, Side};
use fret_core::{Event, MouseButton, Point, PointerEvent, Rect, Size};

#[test]
fn anchored_places_child_via_render_transform_and_updates_layout_out() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );

    let clicks = app.models_mut().insert(0u32);
    let layout_out = app.models_mut().insert(AnchoredPanelLayout {
        rect: Rect::default(),
        side: Side::Bottom,
        align: Align::Start,
        arrow: None,
    });

    let anchor = Rect::new(
        Point::new(Px(50.0), Px(40.0)),
        Size::new(Px(10.0), Px(10.0)),
    );

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "anchored",
        |cx| {
            let clicks = clicks.clone();
            let layout_out = layout_out.clone();
            vec![cx.anchored_props(
                AnchoredProps {
                    anchor,
                    side: Side::Bottom,
                    align: Align::Start,
                    side_offset: Px(0.0),
                    options: AnchoredPanelOptions::default(),
                    layout_out: Some(layout_out),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.pressable(PressableProps::default(), move |cx, _st| {
                        let clicks = clicks.clone();
                        #[allow(clippy::arc_with_non_send_sync)]
                        cx.pressable_add_on_activate(Arc::new(move |host, _action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&clicks, |v| *v = v.saturating_add(1));
                        }));

                        vec![cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: crate::element::Length::Px(Px(100.0)),
                                        height: crate::element::Length::Px(Px(20.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )]
                    })]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let expected = crate::overlay_placement::anchored_panel_layout_sized_ex(
        bounds,
        anchor,
        Size::new(Px(100.0), Px(20.0)),
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions::default(),
    );
    assert_eq!(app.models().get_copied(&layout_out), Some(expected));

    // Click at the untransformed origin; should not hit the translated pressable.
    let origin_point = Point::new(Px(1.0), Px(1.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: origin_point,
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
        &Event::Pointer(PointerEvent::Up {
            position: origin_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&clicks), Some(0));

    // Click within the expected translated rect; should activate the pressable.
    let translated_point = Point::new(
        Px(expected.rect.origin.x.0 + 1.0),
        Px(expected.rect.origin.y.0 + 1.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: translated_point,
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
        &Event::Pointer(PointerEvent::Up {
            position: translated_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&clicks), Some(1));
}

#[test]
fn anchored_can_resolve_anchor_element_bounds_in_layout() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );

    let layout_out = app.models_mut().insert(AnchoredPanelLayout {
        rect: Rect::default(),
        side: Side::Bottom,
        align: Align::Start,
        arrow: None,
    });

    let anchor_element: Cell<Option<u64>> = Cell::new(None);
    let anchor_element = &anchor_element;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "anchored-anchor-element",
        |cx| {
            let layout_out = layout_out.clone();
            // The declarative root is a `Stack` (see `render_root`), which stretches static
            // children to the stack's base bounds. Wrap the scenario in a `Container` so the
            // anchor element gets its intrinsic bounds (10x10) during layout.
            vec![cx.container(ContainerProps::default(), |cx| {
                let anchor = cx.pressable_with_id(PressableProps::default(), |cx, _st, id| {
                    anchor_element.set(Some(id.0));
                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: crate::element::Length::Px(Px(10.0)),
                                    height: crate::element::Length::Px(Px(10.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                });

                let anchored = cx.anchored_props(
                    AnchoredProps {
                        // Intentionally wrong fallback rect: if the element resolution path is broken,
                        // this would place the panel at the origin.
                        anchor: Rect::new(
                            Point::new(Px(0.0), Px(0.0)),
                            Size::new(Px(1.0), Px(1.0)),
                        ),
                        anchor_element: anchor_element.get(),
                        side: Side::Bottom,
                        align: Align::Start,
                        side_offset: Px(0.0),
                        options: AnchoredPanelOptions::default(),
                        layout_out: Some(layout_out),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: crate::element::Length::Px(Px(100.0)),
                                        height: crate::element::Length::Px(Px(20.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )]
                    },
                );

                vec![anchor, anchored]
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // The anchor is laid out before the anchored subtree in the same pass, so the anchor element
    // rect should be available without relying on cross-frame element queries.
    let expected_anchor = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
    let expected = crate::overlay_placement::anchored_panel_layout_sized_ex(
        bounds,
        expected_anchor,
        Size::new(Px(100.0), Px(20.0)),
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions::default(),
    );
    assert_eq!(app.models().get_copied(&layout_out), Some(expected));
}
