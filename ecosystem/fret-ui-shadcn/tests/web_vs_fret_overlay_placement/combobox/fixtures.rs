use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ComboboxRecipe {
    ListboxHeight,
    ListboxOptionHeight,
    ListboxOptionInsets,
    PopoverOverlayPlacement,
    ResponsiveOverlayPlacement,
    ResponsiveViewportAnchoredOverlayPlacement,
}

#[derive(Debug, Clone, Deserialize)]
struct ComboboxCase {
    id: String,
    web_name: String,
    recipe: ComboboxRecipe,
}

fn build_combobox_popover_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
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
}

fn build_combobox_responsive_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
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
}

#[test]
fn web_vs_fret_combobox_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_combobox_cases_v1.json"
    ));
    let suite: FixtureSuite<ComboboxCase> =
        serde_json::from_str(raw).expect("combobox fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("combobox case={}", case.id);
        match case.recipe {
            ComboboxRecipe::ListboxHeight => {
                assert_combobox_demo_listbox_height_matches(&case.web_name);
            }
            ComboboxRecipe::ListboxOptionHeight => {
                assert_combobox_demo_listbox_option_height_matches(&case.web_name);
            }
            ComboboxRecipe::ListboxOptionInsets => {
                assert_combobox_demo_listbox_option_insets_match(&case.web_name);
            }
            ComboboxRecipe::PopoverOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_combobox_popover_overlay(cx, open),
                    SemanticsRole::Button,
                    None,
                    SemanticsRole::Dialog,
                );
            }
            ComboboxRecipe::ResponsiveOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_combobox_responsive_overlay(cx, open),
                    SemanticsRole::ComboBox,
                    None,
                    SemanticsRole::Dialog,
                );
            }
            ComboboxRecipe::ResponsiveViewportAnchoredOverlayPlacement => {
                assert_viewport_anchored_overlay_placement_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    |cx, open| build_combobox_responsive_overlay(cx, open),
                );
            }
        }
    }
}
