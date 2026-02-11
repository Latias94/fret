use super::super::*;

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

pub(super) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DatePickerModels {
        range_open: Option<Model<bool>>,
        range_month: Option<Model<CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        dob_open: Option<Model<bool>>,
        dob_month: Option<Model<CalendarMonth>>,
        dob_selected: Option<Model<Option<Date>>>,
        rtl_open: Option<Model<bool>>,
        rtl_month: Option<Model<CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
    }

    fn parse_iso_date_ymd(raw: &str) -> Option<Date> {
        let raw = raw.trim();
        let (year, rest) = raw.split_once('-')?;
        let (month, day) = rest.split_once('-')?;

        let year: i32 = year.parse().ok()?;
        let month: u8 = month.parse().ok()?;
        let day: u8 = day.parse().ok()?;

        let month = time::Month::try_from(month).ok()?;
        Date::from_calendar_date(year, month, day).ok()
    }

    let diag_calendar_roving =
        std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|v| !v.is_empty());

    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let (
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
    ) = cx.with_state(DatePickerModels::default, |st| {
        (
            st.range_open.clone(),
            st.range_month.clone(),
            st.range_selected.clone(),
            st.dob_open.clone(),
            st.dob_month.clone(),
            st.dob_selected.clone(),
            st.rtl_open.clone(),
            st.rtl_month.clone(),
            st.rtl_selected.clone(),
        )
    });

    let (
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
    ) = match (
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
    ) {
        (
            Some(range_open),
            Some(range_month),
            Some(range_selected),
            Some(dob_open),
            Some(dob_month),
            Some(dob_selected),
            Some(rtl_open),
            Some(rtl_month),
            Some(rtl_selected),
        ) => (
            range_open,
            range_month,
            range_selected,
            dob_open,
            dob_month,
            dob_selected,
            rtl_open,
            rtl_month,
            rtl_selected,
        ),
        _ => {
            let range_open = cx.app.models_mut().insert(false);
            let diag_month = CalendarMonth::from_date(
                Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
            );
            let diag_from =
                Date::from_calendar_date(2024, time::Month::February, 13).expect("valid date");
            let range_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let range_selected = cx.app.models_mut().insert(if diag_calendar_roving {
                DateRangeSelection {
                    from: Some(diag_from),
                    to: None,
                }
            } else {
                DateRangeSelection::default()
            });
            let dob_open = cx.app.models_mut().insert(false);
            let dob_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let dob_selected = cx.app.models_mut().insert(None::<Date>);
            let rtl_open = cx.app.models_mut().insert(false);
            let rtl_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let rtl_selected = cx.app.models_mut().insert(Some(today));

            cx.with_state(DatePickerModels::default, |st| {
                st.range_open = Some(range_open.clone());
                st.range_month = Some(range_month.clone());
                st.range_selected = Some(range_selected.clone());
                st.dob_open = Some(dob_open.clone());
                st.dob_month = Some(dob_month.clone());
                st.dob_selected = Some(dob_selected.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.rtl_month = Some(rtl_month.clone());
                st.rtl_selected = Some(rtl_selected.clone());
            });

            (
                range_open,
                range_month,
                range_selected,
                dob_open,
                dob_month,
                dob_selected,
                rtl_open,
                rtl_month,
                rtl_selected,
            )
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(780.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let gap_card = |cx: &mut ElementContext<'_, App>,
                    title: &'static str,
                    details: &'static str,
                    test_id: &'static str| {
        let alert_content = shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info")),
            shadcn::AlertTitle::new("Guide-aligned placeholder").into_element(cx),
            shadcn::AlertDescription::new(details).into_element(cx),
        ])
        .variant(shadcn::AlertVariant::Default)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(700.0)))
        .into_element(cx)
        .test_id(test_id);
        section_card(cx, title, alert_content)
    };

    let basic_selected = cx
        .app
        .models()
        .read(&selected, |v| v.map(|d| Arc::<str>::from(d.to_string())))
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let basic_picker = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .into_element(cx)
        .test_id("ui-gallery-date-picker-basic");

    let basic_content = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| vec![basic_picker, cx.text(format!("selected: {basic_selected}"))],
    );
    let basic = section_card(cx, "Basic", basic_content);

    let range_value = cx
        .app
        .models()
        .get_cloned(&range_selected)
        .unwrap_or_default();
    let range_from = range_value
        .from
        .map(|d| d.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let range_to = range_value
        .to
        .map(|d| d.to_string())
        .unwrap_or_else(|| "<none>".to_string());

    let mut range_picker = shadcn::DateRangePicker::new(
        range_open.clone(),
        range_month.clone(),
        range_selected.clone(),
    )
    .placeholder("Pick a date range");

    if diag_calendar_roving {
        let d14 = Date::from_calendar_date(2024, time::Month::February, 14).expect("valid date");
        let d15 = Date::from_calendar_date(2024, time::Month::February, 15).expect("valid date");
        range_picker = range_picker.disabled_by(move |d| d == d14 || d == d15);
    }

    let range_picker = range_picker
        .into_element(cx)
        .test_id("ui-gallery-date-picker-range");

    let range_content = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                range_picker,
                cx.text(format!("from: {range_from}")),
                cx.text(format!("to: {range_to}")),
            ]
        },
    );
    let range = section_card(cx, "Range Picker", range_content);

    let dob_text = cx
        .app
        .models()
        .read(&dob_selected, |v| v.map(|d| d.to_string()))
        .ok()
        .flatten()
        .unwrap_or_else(|| "Pick date of birth".to_string());

    let dob = {
        let dob_picker = shadcn::Popover::new(dob_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new(dob_text)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(dob_open.clone())
                        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([shadcn::Calendar::new(
                        dob_month.clone(),
                        dob_selected.clone(),
                    )
                    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                    .close_on_select(dob_open.clone())
                    .into_element(cx)])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .into_element(cx)
                },
            )
            .test_id("ui-gallery-date-picker-dob");

        section_card(cx, "Date of Birth", dob_picker)
    };

    let input = gap_card(
        cx,
        "Input",
        "Input-driven parsing is not yet exposed by current Fret DatePicker API. This section remains explicit to keep docs parity auditable.",
        "ui-gallery-date-picker-input-gap",
    );

    let time_picker = gap_card(
        cx,
        "Time Picker",
        "Time selection widgets are currently implemented in Calendar recipes, but not yet unified into DatePicker API.",
        "ui-gallery-date-picker-time-gap",
    );

    let natural_language = gap_card(
        cx,
        "Natural Language Picker",
        "Natural-language parsing (e.g. chrono-node style) is not available in this runtime surface yet.",
        "ui-gallery-date-picker-natural-gap",
    );

    let rtl = {
        let rtl_picker = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::DatePicker::new(rtl_open.clone(), rtl_month.clone(), rtl_selected.clone())
                    .placeholder("Pick a date")
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-date-picker-rtl");

        section_card(cx, "RTL", rtl_picker)
    };

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Date Picker docs order: Basic, Range Picker, Date of Birth, Input, Time Picker, Natural Language Picker, RTL.",
    );

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                basic,
                range,
                dob,
                input,
                time_picker,
                natural_language,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-date-picker-component");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic + Range",
                    r#"let single = shadcn::DatePicker::new(open, month, selected)
    .placeholder("Pick a date")
    .into_element(cx);

let range = shadcn::DateRangePicker::new(open, month, range_selected)
    .placeholder("Pick a date range")
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "DOB Dropdown Caption",
                    r#"shadcn::Popover::new(open).into_element(cx, |cx| trigger, |cx| {
    shadcn::PopoverContent::new([
        shadcn::Calendar::new(month, selected)
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .close_on_select(open)
            .into_element(cx),
    ]).into_element(cx)
});"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::DatePicker::new(open, month, selected).into_element(cx)
});"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Date picker parity should follow docs sequence even when some recipe surfaces are not yet available in the API.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep unsupported examples visible as explicit gap cards to avoid hidden regressions in future alignment passes.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For date-of-birth flows, dropdown month/year caption improves large-jump navigation compared with arrow-only controls.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Add deterministic test IDs on every scenario so diag scripts can capture state transitions and layout snapshots.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-date-picker",
        component_panel,
        code_panel,
        notes_panel,
    )
}
