use super::*;

#[test]
fn fret_select_tracks_trigger_when_underlay_scrolls() {
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

    let trigger_test_id = "scroll-underlay-select-trigger";

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
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
                            fret_ui_shadcn::SelectItem::new(value, label)
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
                                    fret_ui_shadcn::Select::new(value, open)
                                        .a11y_label("Select")
                                        .placeholder("Select an option")
                                        .trigger_test_id(trigger_test_id)
                                        .refine_layout(
                                            fret_ui_kit::LayoutRefinement::default()
                                                .w_px(Px(280.0)),
                                        )
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
    let trigger_before =
        find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger semantics");
    let listbox_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("listbox semantics (before scroll)");

    let dx_before = listbox_before.bounds.origin.x.0 - trigger_before.bounds.origin.x.0;
    let dy_before = listbox_before.bounds.origin.y.0 - trigger_before.bounds.origin.y.0;

    // Select is expected to install a modal barrier; wheeling outside the listbox should *not*
    // scroll the underlay. (This is still a high-signal check: if the barrier regresses, the
    // underlay scroll can re-anchor overlays and appear as "menu drift".)
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
        "expected modal select to block underlay scroll; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: apply event consequences (if any) and paint once so any transforms update bounds
    // caches. This mirrors the anchored overlay scroll tests even though we expect no underlay
    // scroll here.
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

    // Frame N+1: listbox + trigger should remain stable after wheeling outside the listbox.
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
    let listbox_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("listbox semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() < 0.5,
        "expected trigger to remain stable under modal barrier wheel (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    let dx_after = listbox_after.bounds.origin.x.0 - trigger_after.bounds.origin.x.0;
    let dy_after = listbox_after.bounds.origin.y.0 - trigger_after.bounds.origin.y.0;

    assert_close(
        "select listbox anchor dx stable under scroll",
        dx_after,
        dx_before,
        1.0,
    );
    assert_close(
        "select listbox anchor dy stable under scroll",
        dy_after,
        dy_before,
        1.0,
    );

    // Now wheel *inside* the listbox: it should scroll its own viewport without moving the
    // anchored panel relative to the trigger.
    let top_label_before = snap_after
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox_after.bounds, n.bounds))
        .min_by(|a, b| a.bounds.origin.y.0.total_cmp(&b.bounds.origin.y.0))
        .and_then(|n| n.label.as_deref())
        .unwrap_or_else(|| panic!("missing visible listbox option label (before wheel)"))
        .to_string();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(
                Px(listbox_after.bounds.origin.x.0 + 10.0),
                Px(listbox_after.bounds.origin.y.0 + 10.0),
            ),
            delta: Point::new(Px(0.0), Px(-120.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(5 + settle_frames),
        true,
        |cx| render(cx, &open),
    );
    let snap_scrolled = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after listbox wheel)")
        .clone();
    let trigger_scrolled =
        find_semantics_by_test_id(&snap_scrolled, trigger_test_id).expect("trigger semantics");
    let listbox_scrolled = snap_scrolled
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("listbox semantics (after listbox wheel)");

    let top_label_after = snap_scrolled
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox_scrolled.bounds, n.bounds))
        .min_by(|a, b| a.bounds.origin.y.0.total_cmp(&b.bounds.origin.y.0))
        .and_then(|n| n.label.as_deref())
        .unwrap_or_else(|| panic!("missing visible listbox option label (after wheel)"))
        .to_string();

    assert!(
        top_label_before != top_label_after,
        "expected listbox wheel to scroll options (top_before={top_label_before:?} top_after={top_label_after:?})"
    );

    let dx_scrolled = listbox_scrolled.bounds.origin.x.0 - trigger_scrolled.bounds.origin.x.0;
    let dy_scrolled = listbox_scrolled.bounds.origin.y.0 - trigger_scrolled.bounds.origin.y.0;
    assert_close(
        "select listbox anchor dx stable under listbox scroll",
        dx_scrolled,
        dx_after,
        1.0,
    );
    assert_close(
        "select listbox anchor dy stable under listbox scroll",
        dy_scrolled,
        dy_after,
        1.0,
    );
}
#[test]
fn web_vs_fret_select_scrollable_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x450",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
#[test]
fn web_vs_fret_select_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-demo",
        Some("listbox"),
        |cx, open| {
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("Fruits").into(),
                    SelectItem::new("apple", "Apple").into(),
                    SelectItem::new("banana", "Banana").into(),
                    SelectItem::new("blueberry", "Blueberry").into(),
                    SelectItem::new("grapes", "Grapes").into(),
                    SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a fruit")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
#[test]
fn web_vs_fret_select_demo_vp375x160_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-demo.vp375x160",
        Some("listbox"),
        |cx, open| {
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("Fruits").into(),
                    SelectItem::new("apple", "Apple").into(),
                    SelectItem::new("banana", "Banana").into(),
                    SelectItem::new("blueberry", "Blueberry").into(),
                    SelectItem::new("grapes", "Grapes").into(),
                    SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a fruit")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
#[test]
fn web_vs_fret_select_demo_open_option_metrics_match() {
    assert_select_demo_open_option_metrics_match("select-demo");
}
#[test]
fn web_vs_fret_select_demo_vp375x160_open_option_metrics_match() {
    assert_select_demo_open_option_metrics_match("select-demo.vp375x160");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x240",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}
#[test]
fn web_vs_fret_select_scrollable_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp375x240");
}
#[test]
fn web_vs_fret_select_scrollable_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable.vp375x240");
}
#[test]
fn web_vs_fret_select_scrollable_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable.vp375x240");
}
#[test]
fn web_vs_fret_select_scrollable_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable.vp375x240");
}
#[test]
fn web_vs_fret_select_scrollable_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable.vp375x240");
}
#[test]
fn web_vs_fret_select_scrollable_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable");
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x450");
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x240");
}
#[test]
fn web_vs_fret_select_scrollable_mobile_tiny_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp375x240");
}
