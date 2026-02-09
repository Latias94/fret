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
#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches() {
    assert_tooltip_demo_overlay_placement_matches("tooltip-demo");
}
#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches_tiny_viewport() {
    assert_tooltip_demo_overlay_placement_matches("tooltip-demo.vp1440x240");
}
#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches_mobile_tiny_viewport() {
    assert_tooltip_demo_overlay_placement_matches("tooltip-demo.vp375x240");
}
#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches() {
    assert_hover_card_demo_overlay_placement_matches("hover-card-demo");
}
#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches_tiny_viewport() {
    assert_hover_card_demo_overlay_placement_matches("hover-card-demo.vp1440x240");
}
#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches_mobile_tiny_viewport() {
    assert_hover_card_demo_overlay_placement_matches("hover-card-demo.vp375x240");
}
#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_sidebar_13_dialog_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonSize, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "sidebar-13",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .size(ButtonSize::Sm)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(Vec::new())
                        .refine_style(fret_ui_kit::ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .max_w(fret_ui_kit::MetricRef::Px(Px(800.0)))
                                .max_h(fret_ui_kit::MetricRef::Px(Px(500.0))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_command_dialog_overlay_center_matches() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}
#[test]
fn web_vs_fret_command_dialog_input_height_matches() {
    assert_command_dialog_input_height_matches("command-dialog");
}
#[test]
fn web_vs_fret_command_dialog_input_height_matches_tiny_viewport() {
    assert_command_dialog_input_height_matches("command-dialog.vp1440x240");
}
#[test]
fn web_vs_fret_command_dialog_input_height_matches_mobile_tiny_viewport() {
    assert_command_dialog_input_height_matches("command-dialog.vp375x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_height_matches() {
    assert_command_dialog_listbox_height_matches("command-dialog");
}
#[test]
fn web_vs_fret_command_dialog_listbox_height_matches_tiny_viewport() {
    assert_command_dialog_listbox_height_matches("command-dialog.vp1440x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_height_matches_mobile_tiny_viewport() {
    assert_command_dialog_listbox_height_matches("command-dialog.vp375x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_height_matches() {
    assert_command_dialog_listbox_option_height_matches("command-dialog");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_height_matches_tiny_viewport() {
    assert_command_dialog_listbox_option_height_matches("command-dialog.vp1440x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_height_matches_mobile_tiny_viewport() {
    assert_command_dialog_listbox_option_height_matches("command-dialog.vp375x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_insets_match() {
    assert_command_dialog_listbox_option_insets_match("command-dialog");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_insets_match_tiny_viewport() {
    assert_command_dialog_listbox_option_insets_match("command-dialog.vp1440x240");
}
#[test]
fn web_vs_fret_command_dialog_listbox_option_insets_match_mobile_tiny_viewport() {
    assert_command_dialog_listbox_option_insets_match("command-dialog.vp375x240");
}
#[test]
fn web_vs_fret_command_dialog_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}
#[test]
fn web_vs_fret_command_dialog_overlay_center_matches_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}
#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo.vp1440x240",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches_mobile_tiny_viewport() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo.vp375x240",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.top-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.top-vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right-vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom-vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left-vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_drawer_demo_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    assert_viewport_anchored_overlay_placement_matches(
        "drawer-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Drawer")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_drawer_demo_overlay_insets_match_mobile_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    assert_viewport_anchored_overlay_placement_matches(
        "drawer-demo.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Drawer")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_drawer_dialog_desktop_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "drawer-dialog.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Edit Profile")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .max_w(fret_ui_kit::MetricRef::Px(Px(425.0))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_drawer_dialog_mobile_overlay_insets_match() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle,
    };

    assert_viewport_anchored_overlay_placement_matches(
        "drawer-dialog.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Edit Profile")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DrawerContent::new(vec![
                        DrawerHeader::new(vec![
                            DrawerTitle::new("Edit profile").into_element(cx),
                            DrawerDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                },
            )
        },
    );
}
