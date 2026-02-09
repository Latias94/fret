use super::*;

#[test]
fn fret_combobox_popover_tracks_trigger_when_underlay_scrolls() {
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

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::combobox::{Combobox, ComboboxItem};

        let value = value.clone();
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
                        let items = (0..40).map(|idx| {
                            let value = Arc::from(format!("value-{idx}"));
                            let label = Arc::from(format!("Label {idx}"));
                            ComboboxItem::new(value, label)
                        });

                        vec![cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(280.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![
                                    Combobox::new(value, open)
                                        .a11y_label("Combobox")
                                        .placeholder("Select an option")
                                        .items(items)
                                        .into_element(cx),
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
    let trigger_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .expect("combobox trigger semantics (before scroll)");
    let listbox_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("combobox listbox semantics (before scroll)");

    let dx_before = listbox_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = listbox_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Scroll the underlay (wheel over the scroll viewport, not over the combobox panel).
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

    // Frame N+1: expected to re-anchor the listbox to the scrolled trigger.
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
    let trigger_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .expect("combobox trigger semantics (after scroll)");
    let listbox_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("combobox listbox semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = listbox_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = listbox_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "combobox anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "combobox anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );
}
#[test]
fn fret_combobox_responsive_drawer_blocks_underlay_scroll_on_mobile() {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = StyleAwareServices::default();

    // Mobile width triggers the Drawer-backed "responsive combobox" mode.
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(400.0)),
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::combobox::{Combobox, ComboboxItem};

        let value = value.clone();
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
                        let items = (0..40).map(|idx| {
                            let value = Arc::from(format!("value-{idx}"));
                            let label = Arc::from(format!("Label {idx}"));
                            ComboboxItem::new(value, label)
                        });

                        vec![cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(16.0));
                                    layout.inset.top = Some(Px(160.0));
                                    layout.size.width = Length::Px(Px(280.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![
                                    Combobox::new(value, open)
                                        .responsive(true)
                                        .a11y_label("Combobox")
                                        .placeholder("Select an option")
                                        .items(items)
                                        .into_element(cx),
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
        .expect("semantics snapshot (before wheel)")
        .clone();
    let _ = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("combobox listbox semantics (before wheel)");

    // Drawer-backed combobox is expected to behave modally; wheeling outside the list should not
    // scroll the underlay (prevents "menu drift" when the trigger moves under scroll).
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
        scroll_handle.offset().y.0.abs() < 0.01,
        "expected responsive combobox drawer to block underlay scroll; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: apply event consequences (if any) and paint once so any transforms update bounds
    // caches.
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
    for effect in effects {
        app.push_effect(effect);
    }

    // Frame N+1: the drawer stays open, and the underlay remains unscrolled.
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
        .expect("semantics snapshot (after wheel)")
        .clone();
    let _ = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("combobox listbox semantics (after wheel)");

    assert!(
        scroll_handle.offset().y.0.abs() < 0.01,
        "expected responsive combobox drawer to keep underlay unscrolled; y={}",
        scroll_handle.offset().y.0
    );
}
#[test]
fn web_vs_fret_combobox_demo_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo");
}
#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp1440x320");
}
#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp375x320");
}
#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp1440x240");
}
#[test]
fn web_vs_fret_combobox_demo_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo");
}
#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp1440x320");
}
#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp375x320");
}
#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp1440x240");
}
#[test]
fn web_vs_fret_combobox_demo_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo");
}
#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp1440x320");
}
#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp375x320");
}
#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp1440x240");
}
#[test]
fn web_vs_fret_combobox_popover_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "combobox-popover",
        Some("dialog"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonVariant, Popover, PopoverAlign, PopoverContent, PopoverSide,
            };

            Popover::new(open.clone())
                .side(PopoverSide::Right)
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        PopoverContent::new(Vec::new())
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_px(MetricRef::Px(Px(288.0)))
                                    .h_px(MetricRef::Px(Px(205.33334))),
                            )
                            .into_element(cx)
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_combobox_popover_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "combobox-popover.vp375x240",
        Some("dialog"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonVariant, Popover, PopoverAlign, PopoverContent, PopoverSide,
            };

            Popover::new(open.clone())
                .side(PopoverSide::Right)
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        PopoverContent::new(Vec::new())
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_px(MetricRef::Px(Px(288.0)))
                                    .h_px(MetricRef::Px(Px(205.33334))),
                            )
                            .into_element(cx)
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_combobox_responsive_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "combobox-responsive",
        Some("dialog"),
        |cx, open| {
            use fret_ui_shadcn::{Combobox, ComboboxItem};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let items = vec![
                ComboboxItem::new("nextjs", "Next.js"),
                ComboboxItem::new("sveltekit", "SvelteKit"),
                ComboboxItem::new("nuxt", "Nuxt.js"),
                ComboboxItem::new("remix", "Remix"),
                ComboboxItem::new("astro", "Astro"),
            ];

            Combobox::new(value, open.clone())
                .a11y_label("Select a framework")
                .width(Px(200.0))
                .responsive(true)
                .items(items)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_combobox_responsive_overlay_placement_matches_mobile_tiny_viewport() {
    assert_viewport_anchored_overlay_placement_matches(
        "combobox-responsive.vp375x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            use fret_ui_shadcn::{Combobox, ComboboxItem};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let items = vec![
                ComboboxItem::new("nextjs", "Next.js"),
                ComboboxItem::new("sveltekit", "SvelteKit"),
                ComboboxItem::new("nuxt", "Nuxt.js"),
                ComboboxItem::new("remix", "Remix"),
                ComboboxItem::new("astro", "Astro"),
            ];

            Combobox::new(value, open.clone())
                .a11y_label("Select a framework")
                .width(Px(200.0))
                .responsive(true)
                .items(items)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_combobox_popover_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-popover");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-responsive");
}
#[test]
fn web_vs_fret_combobox_popover_listbox_height_matches_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_height_matches("combobox-popover.vp375x240");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_height_matches_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_height_matches("combobox-responsive.vp375x240");
}
#[test]
fn web_vs_fret_combobox_popover_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-popover");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-responsive");
}
#[test]
fn web_vs_fret_combobox_popover_listbox_option_height_matches_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_option_height_matches("combobox-popover.vp375x240");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_option_height_matches_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_option_height_matches("combobox-responsive.vp375x240");
}
#[test]
fn web_vs_fret_combobox_popover_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-popover");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-responsive");
}
#[test]
fn web_vs_fret_combobox_popover_listbox_option_insets_match_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_option_insets_match("combobox-popover.vp375x240");
}
#[test]
fn web_vs_fret_combobox_responsive_listbox_option_insets_match_mobile_tiny_viewport() {
    assert_combobox_demo_listbox_option_insets_match("combobox-responsive.vp375x240");
}
