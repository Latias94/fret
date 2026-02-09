use super::*;

#[test]
fn fret_tooltip_tracks_trigger_when_underlay_scrolls() {
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

    let scroll_handle = ScrollHandle::default();
    let trigger_test_id = "scroll-underlay-tooltip-trigger";

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Tooltip, TooltipContent, TooltipProvider};

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
                        TooltipProvider::new()
                            .delay_duration_frames(0)
                            .skip_delay_duration_frames(0)
                            .with_elements(cx, |cx| {
                                let trigger = cx.semantics(
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
                                        label: Some(Arc::from("ScrollUnderlayTooltipTrigger")),
                                        test_id: Some(Arc::from(trigger_test_id)),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.pressable(
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
                                            |cx, _st| vec![cx.text("Focus me")],
                                        )]
                                    },
                                );

                                let content = TooltipContent::new(vec![TooltipContent::text(
                                    cx,
                                    "Tooltip content",
                                )])
                                .into_element(cx);
                                let tooltip = Tooltip::new(trigger, content)
                                    .open_delay_frames(0)
                                    .close_delay_frames(0);

                                vec![tooltip.into_element(cx)]
                            })
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed and locate trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render,
    );

    let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger0 = find_semantics_by_test_id(&snap0, trigger_test_id).expect("trigger semantics");
    // Focus trigger to open tooltip (Radix: open on keyboard focus).
    ui.set_focus(Some(trigger0.id));

    // Frame 2+: open and settle motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            render,
        );
    }
    let _ = app.flush_effects();

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before = find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger");
    let tooltip_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Tooltip)
        .expect("tooltip semantics (before scroll)");

    let dx_before = tooltip_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = tooltip_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the tooltip).
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

    // Frame N: apply the scroll and paint once so scroll transforms update visual bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        false,
        render,
    );
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

    // Frame N+1: expected to re-anchor tooltip to the scrolled trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        true,
        render,
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after = find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger");
    let tooltip_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Tooltip)
        .expect("tooltip semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = tooltip_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = tooltip_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "tooltip anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "tooltip anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn fret_popover_tracks_trigger_when_underlay_scrolls() {
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

    let trigger_test_id = "scroll-underlay-popover-trigger";

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::popover::{Popover, PopoverContent};

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
                        vec![Popover::new(open.clone()).into_element(
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
                                        label: Some(Arc::from("ScrollUnderlayPopoverTrigger")),
                                        test_id: Some(Arc::from(trigger_test_id)),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.pressable(
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
                                            move |cx, _st| vec![cx.text("Open")],
                                        )]
                                    },
                                )
                            },
                            move |cx| {
                                PopoverContent::new(vec![cx.text("Popover")])
                                    .a11y_label("ScrollUnderlayPopoverContent")
                                    .into_element(cx)
                            },
                        )]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed and locate trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| render(cx, &open),
    );

    let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let _trigger0 = find_semantics_by_test_id(&snap0, trigger_test_id).expect("trigger semantics");

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and settle motion.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        false,
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
    let _ = app.flush_effects();

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before = find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger");
    let dialog_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Dialog)
        .expect("popover dialog semantics (before scroll)");

    let dx_before = dialog_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = dialog_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the popover panel).
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

    // Frame N: apply the scroll and paint once so scroll transforms update visual bounds caches.
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

    // Frame N+1: expected to re-anchor popover to the scrolled trigger.
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
    let trigger_after = find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger");
    let dialog_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Dialog)
        .expect("popover dialog semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = dialog_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = dialog_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "popover anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "popover anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn fret_hover_card_tracks_trigger_when_underlay_scrolls() {
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

    let scroll_handle = ScrollHandle::default();
    let trigger_test_id = "scroll-underlay-hover-card-trigger";
    let content_test_id = "scroll-underlay-hover-card-content";

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{HoverCard, HoverCardContent, HoverCardSide};

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
                        let trigger = cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                role: SemanticsRole::Button,
                                label: Some(Arc::from("ScrollUnderlayHoverCardTrigger")),
                                test_id: Some(Arc::from(trigger_test_id)),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.pressable(
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
                                    |cx, _st| vec![cx.text("Focus me")],
                                )]
                            },
                        );

                        let content = cx.semantics(
                            fret_ui::element::SemanticsProps {
                                role: SemanticsRole::Panel,
                                test_id: Some(Arc::from(content_test_id)),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![
                                    HoverCardContent::new(vec![cx.text("HoverCard")])
                                        .into_element(cx),
                                ]
                            },
                        );

                        vec![
                            HoverCard::new(trigger, content)
                                .open_delay_frames(0)
                                .close_delay_frames(0)
                                .side(HoverCardSide::Bottom)
                                .into_element(cx),
                        ]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed and locate trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render,
    );

    let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger0 = find_semantics_by_test_id(&snap0, trigger_test_id).expect("trigger semantics");

    // Enter keyboard mode and focus trigger to open hover card.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(trigger0.id));

    // Frame 2+: open and settle motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            render,
        );
    }
    let _ = app.flush_effects();

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before = find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger");
    let card_before = find_semantics_by_test_id(&snap_before, content_test_id)
        .expect("hover card content semantics (before scroll)");

    let dx_before = card_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = card_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the hover card panel).
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

    // Frame N: apply the scroll and paint once so scroll transforms update visual bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        false,
        render,
    );
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

    // Frame N+1: expected to re-anchor hover card to the scrolled trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        true,
        render,
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after = find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger");
    let card_after = find_semantics_by_test_id(&snap_after, content_test_id)
        .expect("hover card content semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = card_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = card_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "hover card anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "hover card anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}

#[path = "misc_overlays/fixtures.rs"]
mod fixtures;
