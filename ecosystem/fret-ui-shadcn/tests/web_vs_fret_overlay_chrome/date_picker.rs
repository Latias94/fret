use super::*;

#[test]
fn web_vs_fret_date_picker_with_presets_select_open_vp375x160_listbox_panel_size_matches_web_light()
{
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "date-picker-with-presets.select-open-vp375x160",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        settle_frames,
        |cx, open| {
            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{
                ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space,
            };
            use fret_ui_shadcn::select::SelectPosition;

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::Popover::new(open.clone())
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
                        let select = fret_ui_shadcn::Select::new(value.clone(), open.clone())
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
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_date_picker_with_presets_select_open_vp375x160_listbox_panel_size_matches_web_dark()
{
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "date-picker-with-presets.select-open-vp375x160",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        settle_frames,
        |cx, open| {
            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{
                ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space,
            };
            use fret_ui_shadcn::select::SelectPosition;

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::Popover::new(open.clone())
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
                        let select = fret_ui_shadcn::Select::new(value.clone(), open.clone())
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
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_date_picker_with_presets_select_open_vp375x240_listbox_panel_size_matches_web_light()
{
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "date-picker-with-presets.select-open-vp375x240",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        settle_frames,
        |cx, open| {
            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{
                ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space,
            };
            use fret_ui_shadcn::select::SelectPosition;

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::Popover::new(open.clone())
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
                        let select = fret_ui_shadcn::Select::new(value.clone(), open.clone())
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
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_date_picker_with_presets_select_open_vp375x240_listbox_panel_size_matches_web_dark()
{
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "date-picker-with-presets.select-open-vp375x240",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        settle_frames,
        |cx, open| {
            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{
                ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space,
            };
            use fret_ui_shadcn::select::SelectPosition;

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::Popover::new(open.clone())
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
                        let select = fret_ui_shadcn::Select::new(value.clone(), open.clone())
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
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_date_picker_with_presets_select_open_vp375x160_listbox_paints_above_popover() {
    let web = read_web_golden_open("date-picker-with-presets.select-open-vp375x160");
    let theme = web_theme_named(&web, "light");
    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let popover_open: Model<bool> = app.models_mut().insert(false);
    let select_open: Model<bool> = app.models_mut().insert(false);
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);

    let build = {
        let popover_open = popover_open.clone();
        let select_open = select_open.clone();
        let value = value.clone();

        move |cx: &mut ElementContext<'_, App>| {
            let popover_open = popover_open.clone();
            let select_open = select_open.clone();
            let value = value.clone();

            use fret_ui_kit::declarative::stack;
            use fret_ui_kit::{
                ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space,
            };
            use fret_ui_shadcn::select::SelectPosition;

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
                        let select =
                            fret_ui_shadcn::Select::new(value.clone(), select_open.clone())
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
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        }
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| vec![build(cx)],
    );

    let _ = app.models_mut().update(&popover_open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames.max(1) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            false,
            |cx| vec![build(cx)],
        );
    }

    let _ = app.models_mut().update(&select_open, |v| *v = true);
    for tick in 0..settle_frames.max(1) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames.max(1) + tick),
            tick + 1 == settle_frames.max(1),
            |cx| vec![build(cx)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let popover = largest_semantics_node(&snap, SemanticsRole::Dialog)
        .expect("missing popover dialog semantics node");
    let listbox = largest_semantics_node(&snap, SemanticsRole::ListBox)
        .expect("missing listbox semantics node");

    let (popover_idx, _) = find_best_chrome_quad_indexed(&scene, popover.bounds)
        .expect("painted chrome quad for popover");
    let (listbox_idx, _) = find_best_chrome_quad_indexed(&scene, listbox.bounds)
        .expect("painted chrome quad for listbox");

    assert!(
        listbox_idx > popover_idx,
        "expected listbox chrome to be painted after popover chrome (popover_idx={popover_idx} listbox_idx={listbox_idx})"
    );
}
