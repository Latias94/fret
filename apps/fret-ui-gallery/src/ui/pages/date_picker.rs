use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_kit::declarative::style as decl_style;

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

    let theme = Theme::global(&*cx.app).snapshot();

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

    let simple = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-simple");

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
        .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-range");
    let range = range_picker;

    let dropdown_text = cx
        .app
        .models()
        .read(&dob_selected, |v| v.map(|d| d.to_string()))
        .ok()
        .flatten()
        .unwrap_or_else(|| "Pick a date".to_string());

    let dropdowns_is_desktop = fret_ui_kit::declarative::viewport_width_at_least(
        cx,
        fret_ui::Invalidation::Layout,
        Px(768.0),
        fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
    );

    let dropdowns = {
        let theme = theme.clone();
        let open = dob_open.clone();
        let open_for_trigger = open.clone();
        let open_for_content = open.clone();
        let dropdown_text = dropdown_text.clone();
        let dropdown_placeholder = dropdown_text == "Pick a date";

        let trigger = move |cx: &mut ElementContext<'_, App>| {
            let mut button = shadcn::Button::new(dropdown_text.clone())
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .leading_icon(fret_icons::IconId::new_static("lucide.calendar"))
                .content_justify_start()
                .text_weight(fret_core::FontWeight::NORMAL)
                .refine_layout(LayoutRefinement::default().w_px(Px(240.0)));

            if dropdown_placeholder {
                button = button.style(shadcn::button::ButtonStyle::default().foreground(
                    fret_ui_kit::WidgetStateProperty::new(Some(fret_ui_kit::ColorRef::Token {
                        key: "muted-foreground",
                        fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                    })),
                ));
            }

            button
                .test_id("ui-gallery-date-picker-dropdowns-trigger")
                .into_element(cx)
        };

        let content = move |cx: &mut ElementContext<'_, App>| {
            let calendar = shadcn::Calendar::new(dob_month.clone(), dob_selected.clone())
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-calendar");

            let done = shadcn::Button::new("Done")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .toggle_model(open_for_content.clone())
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-done");

            let footer_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N2),
                LayoutRefinement::default().w_full().min_w_0(),
            );
            let footer = cx.container(footer_props, move |_cx| vec![done]);

            let separator = shadcn::Separator::new()
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-separator");

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| vec![calendar, separator, footer],
            );

            if dropdowns_is_desktop {
                shadcn::PopoverContent::new([body])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-dropdowns-popover-content")
            } else {
                shadcn::DrawerContent::new([body])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-dropdowns-drawer-content")
            }
        };

        if dropdowns_is_desktop {
            shadcn::Popover::new(open.clone())
                .side(shadcn::PopoverSide::Bottom)
                .align(shadcn::PopoverAlign::Start)
                .into_element(cx, trigger, content)
                .test_id("ui-gallery-date-picker-dropdowns")
        } else {
            shadcn::Drawer::new(open.clone())
                .into_element(cx, trigger, content)
                .test_id("ui-gallery-date-picker-dropdowns")
        }
    };

    let demo = doc_layout::wrap_row_snapshot(
        cx,
        &theme,
        Space::N4,
        fret_ui::element::CrossAlign::Start,
        |_cx| vec![simple, dropdowns, range],
    )
    .test_id("ui-gallery-date-picker-demo");

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::DatePicker::new(rtl_open.clone(), rtl_month.clone(), rtl_selected.clone())
            .placeholder("Pick a date")
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-date-picker-rtl");

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Demo aligns to shadcn `DatePickerDemo` (Simple, With Dropdowns, With Range).",
            "The dropdowns demo renders Popover on desktop and Drawer on narrow viewports (explicit branches for deterministic gallery validation).",
            "Calendar dropdown caption improves large-jump navigation compared with arrow-only controls.",
            "For diag runs, some dates are intentionally disabled (via env flag) to validate skip behavior.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn DatePickerDemo flow (Simple + With Dropdowns + With Range). Extras: RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("shadcn demo: simple, dropdown caption, and range (2-month) pickers.")
                .code(
                    "rust",
                    r#"// Simple
shadcn::DatePicker::new(open, month, selected)
    .placeholder("Pick a date")
    .into_element(cx);

// With dropdown caption + Done footer (Popover on desktop, Drawer on mobile)
let is_desktop = fret_ui_kit::declarative::viewport_width_at_least(
    cx,
    fret_ui::Invalidation::Layout,
    Px(768.0),
    fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
);
if is_desktop {
    shadcn::Popover::new(open).into_element(cx, |cx| trigger, |cx| {
        shadcn::PopoverContent::new([calendar]).into_element(cx)
    })
} else {
    shadcn::Drawer::new(open).into_element(cx, |cx| trigger, |cx| {
        shadcn::DrawerContent::new([calendar]).into_element(cx)
    })
};

// With range (2 months)
shadcn::DateRangePicker::new(open, month, range_selected).into_element(cx);"#,
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Extras: RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::DatePicker::new(open, month, selected).into_element(cx)
});"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Notes", notes_stack)
                .description("Guidelines and parity notes for date picker recipes.")
                .max_w(Px(780.0)),
        ],
    );

    vec![body]
}
