use super::*;

#[test]
fn fret_dropdown_menu_tracks_trigger_when_underlay_scrolls() {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(400.0)),
    );

    let open = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    let trigger_test_id = "scroll-underlay-trigger";

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::dropdown_menu::{DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

        let open = open.clone();
        let scroll_handle = scroll_handle.clone();

        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = fret_ui::element::Overflow::Clip;

        vec![cx.scroll(
            fret_ui::element::ScrollProps {
                layout: scroll_layout,
                axis: fret_ui::element::ScrollAxis::Y,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(1200.0));
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let dropdown = DropdownMenu::new(open.clone());

                        vec![dropdown.into_element(
                            cx,
                            move |cx| {
                                cx.semantics(
                                    fret_ui::element::SemanticsProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.position =
                                                fret_ui::element::PositionStyle::Absolute;
                                            layout.inset.left = Some(Px(16.0));
                                            layout.inset.top = Some(Px(160.0));
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(32.0));
                                            layout
                                        },
                                        role: SemanticsRole::Button,
                                        label: Some(Arc::from("ScrollUnderlayTrigger")),
                                        test_id: Some(Arc::from(trigger_test_id)),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.pressable_with_id(
                                            fret_ui::element::PressableProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |cx, _st, _id| vec![cx.text("Open")],
                                        )]
                                    },
                                )
                            },
                            move |_cx| {
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item A")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item B")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Item C")),
                                ]
                            },
                        )]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed so the trigger element id mapping is stable.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| render(cx, &open),
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and settle motion to avoid interpreting the open animation as scroll drift.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| render(cx, &open),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| render(cx, &open),
        );
    }

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before =
        find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger semantics");
    let menu_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (before scroll)");

    let dx_before = menu_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = menu_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the menu panel).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected scroll handle offset to update after wheel; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: apply the scroll. Anchored overlays place using last-frame bounds, so this frame
    // may still use the pre-scroll trigger rect. The runtime should request a follow-up redraw
    // so the next frame can re-anchor using the updated bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        false,
        |cx| render(cx, &open),
    );

    // Paint once so scroll-induced child render transforms are reflected in last-frame visual
    // bounds caches (used by anchored overlay placement).
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let effects = app.flush_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Redraw(w) if *w == window)),
        "expected a follow-up redraw after scroll to re-anchor overlays; effects={effects:?}",
    );
    for effect in effects {
        match effect {
            Effect::Redraw(_) => {}
            other => app.push_effect(other),
        }
    }

    // Frame N+1: expected to re-anchor the menu to the scrolled trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4 + settle_frames),
        true,
        |cx| render(cx, &open),
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after =
        find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger semantics");
    let menu_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = menu_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = menu_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "dropdown menu anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "dropdown menu anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}

#[path = "dropdown_menu_misc/fixtures.rs"]
mod fixtures;
