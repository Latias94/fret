use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px, Rect, Size,
};
use fret_ui::UiTree;
use fret_ui::declarative::{render_dismissible_root_with_hooks, render_root};
use fret_ui::element::{
    InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle, SizeStyle,
};

use super::{NullServices, TestUiHostImpl};

#[test]
fn portal_pointer_region_blocks_underlay_only_inside_body_region() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::new();

    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay_downs_hook = underlay_downs.clone();

    let underlay_root = render_root(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "test.underlay.pointer_region",
        |ecx| {
            let mut props = PointerRegionProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            let region = ecx.pointer_region(props, |ecx| {
                ecx.pointer_region_on_pointer_down(Arc::new(move |_host, _cx, _down| {
                    underlay_downs_hook.fetch_add(1, Ordering::Relaxed);
                    true
                }));
                Vec::new()
            });
            vec![region]
        },
    );

    ui.set_root(underlay_root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let portal_downs = Arc::new(AtomicUsize::new(0));
    let portal_downs_hook = portal_downs.clone();
    let portal_root = render_dismissible_root_with_hooks(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "test.portal.body_pointer_region",
        |ecx| {
            let mut props = PointerRegionProps::default();
            props.layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(Px(20.0)),
                    top: Some(Px(36.0)),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(Px(260.0)),
                    height: Length::Px(Px(80.0)),
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.pointer_region(props, |ecx| {
                ecx.pointer_region_on_pointer_down(Arc::new(move |_host, _cx, _down| {
                    portal_downs_hook.fetch_add(1, Ordering::Relaxed);
                    true
                }));
                Vec::new()
            })]
        },
    );

    let _portal_layer = ui.push_overlay_root_ex(portal_root, false, true);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let header_pos = Point::new(Px(30.0), Px(20.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: header_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: header_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    let body_pos = Point::new(Px(30.0), Px(60.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: body_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: body_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    assert_eq!(
        underlay_downs.load(Ordering::Relaxed),
        1,
        "expected underlay to receive pointer down only outside portal interactive region"
    );
    assert_eq!(
        portal_downs.load(Ordering::Relaxed),
        1,
        "expected portal interactive region to receive pointer down only inside its bounds"
    );
}
