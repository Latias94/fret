use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DatePickerRecipe {
    WithPresetsOpenOverlayPlacement,
    WithPresetsPresetTomorrowOpenOverlayPlacement,
    WithPresetsSelectListboxOverlayPlacement,
    WithRangeOpenOverlayPlacement,
}

#[derive(Debug, Clone, Deserialize)]
struct DatePickerCase {
    id: String,
    web_name: String,
    recipe: DatePickerRecipe,
}

fn build_date_picker_with_presets_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use time::Month;

    let month: Model<CalendarMonth> = cx
        .app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

    fret_ui_shadcn::DatePickerWithPresets::new(open.clone(), month, selected)
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx)
}

fn build_date_picker_with_presets_preset_tomorrow_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use time::{Date, Month};

    let month: Model<CalendarMonth> = cx
        .app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
        Date::from_calendar_date(2026, Month::January, 16).expect("selected date"),
    ));

    fret_ui_shadcn::DatePickerWithPresets::new(open.clone(), month, selected)
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx)
}

fn build_date_picker_with_presets_select_listbox_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_kit::{ChromeRefinement, LengthRefinement, MetricRef};
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
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
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
}

fn build_date_range_picker_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use time::{Date, Month};

    let month: Model<CalendarMonth> = cx
        .app
        .models_mut()
        .insert(CalendarMonth::new(2022, Month::January));
    let selected: Model<DateRangeSelection> = cx.app.models_mut().insert(DateRangeSelection {
        from: Some(Date::from_calendar_date(2022, Month::January, 20).expect("from date")),
        to: Some(Date::from_calendar_date(2022, Month::February, 9).expect("to date")),
    });

    fret_ui_shadcn::DateRangePicker::new(open.clone(), month, selected)
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(300.0))))
        .into_element(cx)
}

#[test]
fn web_vs_fret_date_picker_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_date_picker_cases_v1.json"
    ));
    let suite: FixtureSuite<DatePickerCase> =
        serde_json::from_str(raw).expect("date picker fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("date-picker case={}", case.id);
        match case.recipe {
            DatePickerRecipe::WithPresetsOpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_date_picker_with_presets_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Pick a date"),
                    SemanticsRole::Dialog,
                );
            }
            DatePickerRecipe::WithPresetsPresetTomorrowOpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| {
                        build_date_picker_with_presets_preset_tomorrow_open_overlay(cx, open)
                    },
                    SemanticsRole::Button,
                    Some("January 16th, 2026"),
                    SemanticsRole::Dialog,
                );
            }
            DatePickerRecipe::WithPresetsSelectListboxOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("listbox"),
                    |cx, open| build_date_picker_with_presets_select_listbox_overlay(cx, open),
                    SemanticsRole::ComboBox,
                    None,
                    SemanticsRole::ListBox,
                );
            }
            DatePickerRecipe::WithRangeOpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_date_range_picker_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Jan 20, 2022 - Feb 09, 2022"),
                    SemanticsRole::Dialog,
                );
            }
        }
    }
}
