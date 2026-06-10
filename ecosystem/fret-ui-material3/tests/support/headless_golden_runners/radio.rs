use std::{collections::BTreeMap, sync::Arc};

use fret_core::{AppWindowId, KeyCode, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{UiTree, element::AnyElement, elements::ElementContext};
use fret_ui_material3::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    events::pointer_move,
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_interactions::{dispatch_key_tap, focus_test_id, hover_test_id},
    headless_radio_cases::{Material3RadioGoldenCaseKindV1, load_material3_radio_golden_suite_v1},
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_radio_suite_goldens_v1() {
    let radio_suite = load_material3_radio_golden_suite_v1();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(520.0), Px(360.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in radio_suite.cases() {
                let case_name = case.id();
                let context = format!("{label}, {scale}, {case_name}");

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let selected = app.models_mut().insert(true);
                let unselected = app.models_mut().insert(false);
                let disabled_selected = app.models_mut().insert(true);
                let disabled_unselected = app.models_mut().insert(false);
                let horizontal_value = app.models_mut().insert(Some(Arc::<str>::from("alpha")));
                let vertical_value = app.models_mut().insert(Some(Arc::<str>::from("epsilon")));

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "radio_root",
                        |cx| {
                            let content = radio_matrix(
                                cx,
                                selected.clone(),
                                unselected.clone(),
                                disabled_selected.clone(),
                                disabled_unselected.clone(),
                                horizontal_value.clone(),
                                vertical_value.clone(),
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
                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                );

                match case.kind() {
                    Material3RadioGoldenCaseKindV1::Idle => {}
                    Material3RadioGoldenCaseKindV1::Hover => {
                        let _ = hover_test_id(
                            &mut ui,
                            &mut app,
                            &mut services,
                            case.target_test_id(),
                            &context,
                        );
                    }
                    Material3RadioGoldenCaseKindV1::FocusVisible => {
                        focus_test_id(&mut ui, case.target_test_id(), &context);
                        dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::Tab);
                    }
                }

                let message = format!(
                    "expected the Material3 radio {case_name} scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-radio.{scale}.{label}"),
                "material3_headless_radio_suite_goldens_v1",
                &suite,
            );
        }
    }
}

fn radio_matrix(
    cx: &mut ElementContext<'_, TestHost>,
    selected: Model<bool>,
    unselected: Model<bool>,
    disabled_selected: Model<bool>,
    disabled_unselected: Model<bool>,
    horizontal_value: Model<Option<Arc<str>>>,
    vertical_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    let mut column = fret_ui::element::FlexProps::default();
    column.direction = fret_core::Axis::Vertical;
    column.gap = fret_ui::element::SpacingLength::Px(Px(18.0));

    cx.flex(column, |cx| {
        vec![
            standalone_radio_row(
                cx,
                selected,
                unselected,
                disabled_selected,
                disabled_unselected,
            ),
            radio_group_row(cx, horizontal_value, vertical_value),
        ]
    })
}

fn standalone_radio_row(
    cx: &mut ElementContext<'_, TestHost>,
    selected: Model<bool>,
    unselected: Model<bool>,
    disabled_selected: Model<bool>,
    disabled_unselected: Model<bool>,
) -> AnyElement {
    let mut row = fret_ui::element::FlexProps::default();
    row.direction = fret_core::Axis::Horizontal;
    row.gap = fret_ui::element::SpacingLength::Px(Px(12.0));

    cx.flex(row, |cx| {
        vec![
            Radio::new(selected)
                .a11y_label("selected radio")
                .test_id("radio-selected")
                .into_element(cx),
            Radio::new(unselected)
                .a11y_label("unselected radio")
                .test_id("radio-unselected")
                .into_element(cx),
            Radio::new(disabled_selected)
                .disabled(true)
                .a11y_label("disabled selected radio")
                .test_id("radio-disabled-selected")
                .into_element(cx),
            Radio::new(disabled_unselected)
                .disabled(true)
                .a11y_label("disabled unselected radio")
                .test_id("radio-disabled-unselected")
                .into_element(cx),
        ]
    })
}

fn radio_group_row(
    cx: &mut ElementContext<'_, TestHost>,
    horizontal_value: Model<Option<Arc<str>>>,
    vertical_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    let mut row = fret_ui::element::FlexProps::default();
    row.direction = fret_core::Axis::Horizontal;
    row.gap = fret_ui::element::SpacingLength::Px(Px(40.0));
    row.align = fret_ui::element::CrossAlign::Start;

    cx.flex(row, |cx| {
        vec![
            RadioGroup::new(horizontal_value)
                .orientation(RadioGroupOrientation::Horizontal)
                .gap(Px(8.0))
                .a11y_label("horizontal radio group")
                .test_id("radio-group-horizontal")
                .items(vec![
                    RadioGroupItem::new("alpha")
                        .a11y_label("Alpha")
                        .test_id("radio-group-horizontal-alpha"),
                    RadioGroupItem::new("beta")
                        .disabled(true)
                        .a11y_label("Beta")
                        .test_id("radio-group-horizontal-beta"),
                    RadioGroupItem::new("gamma")
                        .a11y_label("Gamma")
                        .test_id("radio-group-horizontal-gamma"),
                ])
                .into_element(cx),
            RadioGroup::new(vertical_value)
                .orientation(RadioGroupOrientation::Vertical)
                .gap(Px(8.0))
                .a11y_label("vertical radio group")
                .test_id("radio-group-vertical")
                .items(vec![
                    RadioGroupItem::new("delta")
                        .a11y_label("Delta")
                        .test_id("radio-group-vertical-delta"),
                    RadioGroupItem::new("epsilon")
                        .a11y_label("Epsilon")
                        .test_id("radio-group-vertical-epsilon"),
                    RadioGroupItem::new("zeta")
                        .disabled(true)
                        .a11y_label("Zeta")
                        .test_id("radio-group-vertical-zeta"),
                ])
                .into_element(cx),
        ]
    })
}
