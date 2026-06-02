use std::collections::BTreeMap;

use fret_core::{AppWindowId, Axis, KeyCode, Px, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{
    UiTree,
    element::{AnyElement, CrossAlign, FlexProps, Length, MainAlign},
    elements::ElementContext,
};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    AssistChip, AssistChipVariant, ChipSet, ChipSetItem, FilterChip, FilterChipVariant, InputChip,
    SuggestionChip, SuggestionChipVariant,
};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_chip_set_cases::{
        Material3ChipSetGoldenCaseKindV1, Material3ChipSetGoldenLayoutDirectionV1,
        load_material3_chip_set_golden_suite_v1,
    },
    headless_interactions::{
        dispatch_idle_pointer, dispatch_key_tap, focus_test_id, hover_test_id,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::{apply_material_theme, apply_material_theme_rtl},
};

pub(crate) fn run_material3_headless_chip_set_suite_goldens_v1() {
    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];
    let chip_set_suite = load_material3_chip_set_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = chip_set_suite.bounds();
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in chip_set_suite.cases() {
                let case_name = case.id();
                let context = format!("{label}, {scale}, {case_name}");

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                match case.layout_direction() {
                    Material3ChipSetGoldenLayoutDirectionV1::Ltr => {
                        apply_material_theme(&mut app, mode, variant);
                    }
                    Material3ChipSetGoldenLayoutDirectionV1::Rtl => {
                        apply_material_theme_rtl(&mut app, mode, variant);
                    }
                }

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let filter_selected = app.models_mut().insert(true);
                let filter_unselected = app.models_mut().insert(false);
                let input_selected = app.models_mut().insert(true);
                let input_unselected = app.models_mut().insert(false);

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "chip_set_root",
                        |cx| {
                            let content = chip_set_matrix(
                                cx,
                                filter_selected.clone(),
                                filter_unselected.clone(),
                                input_selected.clone(),
                                input_unselected.clone(),
                            );
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                ui.set_focus(None);
                dispatch_idle_pointer(&mut ui, &mut app, &mut services);

                match case.kind() {
                    Material3ChipSetGoldenCaseKindV1::Idle => {}
                    Material3ChipSetGoldenCaseKindV1::Hover => {
                        let _ = hover_test_id(
                            &mut ui,
                            &mut app,
                            &mut services,
                            case.target_test_id(),
                            &context,
                        );
                    }
                    Material3ChipSetGoldenCaseKindV1::FocusVisible => {
                        focus_test_id(&mut ui, case.target_test_id(), &context);
                        dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::Tab);
                    }
                }

                let message = format!(
                    "expected the Material3 chip set {case_name} scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-chip-set.{scale}.{label}"),
                "material3_headless_chip_set_suite_goldens_v1",
                &suite,
            );
        }
    }
}

fn chip_set_matrix(
    cx: &mut ElementContext<'_, TestHost>,
    filter_selected: Model<bool>,
    filter_unselected: Model<bool>,
    input_selected: Model<bool>,
    input_unselected: Model<bool>,
) -> AnyElement {
    let mut column = FlexProps::default();
    column.layout.size.width = Length::Fill;
    column.layout.size.min_width = Some(Length::Px(Px(0.0)));
    column.direction = Axis::Vertical;
    column.justify = MainAlign::Start;
    column.align = CrossAlign::Start;
    column.gap = Px(20.0).into();

    cx.flex(column, move |cx| {
        vec![
            main_chip_set(cx, filter_selected, input_unselected),
            wrap_chip_set_lane(cx, filter_unselected, input_selected),
        ]
    })
}

fn main_chip_set(
    cx: &mut ElementContext<'_, TestHost>,
    filter_selected: Model<bool>,
    input_unselected: Model<bool>,
) -> AnyElement {
    ChipSet::new(vec![
        ChipSetItem::from(
            AssistChip::new("Assist")
                .leading_icon(fret_icons::ids::ui::SEARCH)
                .variant(AssistChipVariant::Elevated)
                .test_id("chip-set-assist"),
        ),
        ChipSetItem::from(
            SuggestionChip::new("Suggest")
                .leading_icon(fret_icons::ids::ui::SETTINGS)
                .variant(SuggestionChipVariant::Elevated)
                .test_id("chip-set-suggestion"),
        ),
        ChipSetItem::from(
            FilterChip::new(filter_selected, "Filter")
                .leading_icon(fret_icons::ids::ui::CHECK)
                .trailing_icon(fret_icons::ids::ui::CLOSE)
                .test_id("chip-set-filter"),
        ),
        ChipSetItem::from(
            InputChip::new(input_unselected, "Input")
                .leading_icon(fret_icons::ids::ui::SEARCH)
                .trailing_icon(fret_icons::ids::ui::CLOSE)
                .test_id("chip-set-input"),
        ),
        ChipSetItem::from(
            SuggestionChip::new("Disabled")
                .disabled(true)
                .test_id("chip-set-disabled"),
        ),
    ])
    .a11y_label("Material chip set")
    .test_id("chip-set-main")
    .into_element(cx)
}

fn wrap_chip_set_lane(
    cx: &mut ElementContext<'_, TestHost>,
    filter_unselected: Model<bool>,
    input_selected: Model<bool>,
) -> AnyElement {
    let mut lane = FlexProps::default();
    lane.layout.size.width = Length::Px(Px(124.0));
    lane.layout.size.min_width = Some(Length::Px(Px(0.0)));
    lane.direction = Axis::Vertical;
    lane.justify = MainAlign::Start;
    lane.align = CrossAlign::Stretch;

    cx.flex(lane, move |cx| {
        vec![
            ChipSet::new(vec![
                ChipSetItem::from(SuggestionChip::new("Alpha").test_id("chip-set-wrap-alpha")),
                ChipSetItem::from(SuggestionChip::new("Beta").test_id("chip-set-wrap-beta")),
                ChipSetItem::from(
                    FilterChip::new(filter_unselected, "Gamma")
                        .variant(FilterChipVariant::Elevated)
                        .test_id("chip-set-wrap-gamma"),
                ),
                ChipSetItem::from(
                    InputChip::new(input_selected, "Delta")
                        .trailing_icon(fret_icons::ids::ui::CLOSE)
                        .test_id("chip-set-wrap-delta"),
                ),
            ])
            .wrap_layout(true)
            .gap(Px(8.0))
            .a11y_label("Wrapping chip set")
            .test_id("chip-set-wrap")
            .into_element(cx),
        ]
    })
}
