use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebThemeName {
    Light,
    Dark,
}

impl WebThemeName {
    fn as_str(&self) -> &'static str {
        match self {
            WebThemeName::Light => "light",
            WebThemeName::Dark => "dark",
        }
    }

    fn scheme(&self) -> fret_ui_shadcn::shadcn_themes::ShadcnColorScheme {
        match self {
            WebThemeName::Light => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
            WebThemeName::Dark => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DatePickerOverlayChromeRecipe {
    ListboxPanelSize,
}

#[derive(Debug, Clone, Deserialize)]
struct DatePickerOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: DatePickerOverlayChromeRecipe,
    theme: WebThemeName,
}

fn build_date_picker_with_presets_select_open(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Space};
    use fret_ui_shadcn::select::SelectPosition;

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let open = open.clone();

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

#[test]
fn web_vs_fret_date_picker_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_date_picker_cases_v1.json"
    ));
    let suite: FixtureSuite<DatePickerOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome date picker fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome date-picker case={}", case.id);
        match case.recipe {
            DatePickerOverlayChromeRecipe::ListboxPanelSize => {
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_overlay_panel_size_matches_by_portal_slot_theme(
                    &case.web_name,
                    "select-content",
                    case.theme.as_str(),
                    case.theme.scheme(),
                    SemanticsRole::ListBox,
                    settle_frames,
                    build_date_picker_with_presets_select_open,
                );
            }
        }
    }
}
