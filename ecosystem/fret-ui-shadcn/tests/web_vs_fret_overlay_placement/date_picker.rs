use super::*;

#[test]
fn web_vs_fret_date_picker_with_presets_select_listbox_scroll_matches_web_scrolled_tiny_viewport_160h()
 {
    let web = read_web_golden_open("date-picker-with-presets.select-open-vp375x160-scrolled-80");
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(&theme);
    let expected_first_visible = web_first_visible_select_option_label(
        web_listbox,
        &["Today", "Tomorrow", "In 3 days", "In a week"],
    )
    .expect("web first visible select option label");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::from_web_theme(theme);

    let bounds = bounds_for_web_theme(&theme);

    let popover_open: Model<bool> = app.models_mut().insert(false);
    let select_open: Model<bool> = app.models_mut().insert(false);
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);

    let render = {
        let popover_open = popover_open.clone();
        let select_open = select_open.clone();
        let value = value.clone();

        move |cx: &mut ElementContext<'_, App>| {
            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{ChromeRefinement, LengthRefinement, MetricRef, Space};
            use fret_ui_shadcn::select::SelectPosition;

            let popover_open = popover_open.clone();
            let select_open = select_open.clone();
            let value = value.clone();

            vec![
                fret_ui_shadcn::Popover::new(popover_open)
                    .align(fret_ui_shadcn::PopoverAlign::Start)
                    .side(fret_ui_shadcn::PopoverSide::Bottom)
                    .into_element(
                        cx,
                        |cx| {
                            fret_ui_shadcn::Button::new("Pick a date")
                                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                                .refine_layout(
                                    LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))),
                                )
                                .into_element(cx)
                        },
                        move |cx| {
                            let value = value.clone();
                            let select_open = select_open.clone();

                            let select = fret_ui_shadcn::Select::new(value, select_open)
                                .placeholder("Select")
                                .position(SelectPosition::Popper)
                                .items([
                                    fret_ui_shadcn::SelectItem::new("0", "Today"),
                                    fret_ui_shadcn::SelectItem::new("1", "Tomorrow"),
                                    fret_ui_shadcn::SelectItem::new("3", "In 3 days"),
                                    fret_ui_shadcn::SelectItem::new("7", "In a week"),
                                ])
                                .into_element(cx);

                            let body = stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N2).items_stretch(),
                                move |_cx| vec![select],
                            );

                            fret_ui_shadcn::PopoverContent::new([body])
                                .refine_style(ChromeRefinement::default().p(Space::N2))
                                .refine_layout(
                                    LayoutRefinement::default().w(LengthRefinement::Auto),
                                )
                                .into_element(cx)
                        },
                    ),
            ]
        }
    };

    // Frame 1: mount closed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        render.clone(),
    );

    // Open popover and settle.
    let _ = app.models_mut().update(&popover_open, |v| *v = true);
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
            render.clone(),
        );
    }

    // Open select and settle; request semantics snapshot at the end.
    let _ = app.models_mut().update(&select_open, |v| *v = true);
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames + tick),
            request_semantics,
            render.clone(),
        );
    }

    // Paint once so last-frame visual bounds caches are populated (Select uses them for initial
    // scroll alignment). This keeps the test closer to real render-frame behavior.
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    let _ = app.flush_effects();

    // One more frame so scroll alignment logic can observe the populated caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames * 2),
        true,
        render.clone(),
    );

    let snap_before = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox_before = snap_before
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            let aa = a.bounds.size.width.0 * a.bounds.size.height.0;
            let bb = b.bounds.size.width.0 * b.bounds.size.height.0;
            aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("missing listbox semantics node");

    if std::env::var("FRET_DEBUG_SELECT_SCROLL")
        .ok()
        .is_some_and(|v| v == "1")
    {
        eprintln!(
            "select scroll debug: listbox bounds={:?}",
            listbox_before.bounds
        );
        if let Some(active) = listbox_before.active_descendant {
            let active_node = snap_before
                .nodes
                .iter()
                .find(|n| n.id == active)
                .and_then(|n| n.label.as_deref());
            eprintln!("  listbox active_descendant={active:?} label={active_node:?}");
        } else {
            eprintln!("  listbox active_descendant=<none>");
        }
        for opt in snap_before
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
        {
            eprintln!(
                "  opt label={:?} bounds={:?}",
                opt.label.as_deref(),
                opt.bounds
            );
        }
    }

    // Wheel over the listbox: should scroll listbox content (not re-anchor/move the overlay).
    let listbox_center = Point::new(
        Px(listbox_before.bounds.origin.x.0 + listbox_before.bounds.size.width.0 * 0.5),
        Px(listbox_before.bounds.origin.y.0 + listbox_before.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: listbox_center,
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Frame N: apply scroll and snapshot.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames * 2),
        true,
        render.clone(),
    );

    let snap_after = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox_after = snap_after
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            let aa = a.bounds.size.width.0 * a.bounds.size.height.0;
            let bb = b.bounds.size.width.0 * b.bounds.size.height.0;
            aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("missing listbox semantics node (after scroll)");

    assert_close(
        "listbox overlay x stable under internal scroll",
        listbox_after.bounds.origin.x.0,
        listbox_before.bounds.origin.x.0,
        1.0,
    );
    assert_close(
        "listbox overlay y stable under internal scroll",
        listbox_after.bounds.origin.y.0,
        listbox_before.bounds.origin.y.0,
        1.0,
    );

    let after_first_visible = fret_first_visible_listbox_option_label(
        &snap_after,
        listbox_after.bounds,
        &["Today", "Tomorrow", "In 3 days", "In a week"],
    )
    .unwrap_or("<missing>");

    assert_eq!(
        after_first_visible, expected_first_visible,
        "first-visible option label mismatch after listbox scroll"
    );
}

#[path = "date_picker/fixtures.rs"]
mod fixtures;
