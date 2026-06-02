use std::{collections::BTreeMap, sync::Arc};

use fret_core::{AppWindowId, KeyCode, Point, Px, Rect, Size, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{
    UiTree,
    element::{ContainerProps, FlexProps, Length},
};
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::{AutocompleteItem, AutocompleteVariant, ExposedDropdown};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, run_overlay_frame_scaled,
        settle_material3_overlay_scene_snapshot_v1, write_or_assert_material3_suite_for_test_v1,
    },
    headless_exposed_dropdown_cases::load_material3_exposed_dropdown_golden_suite_v1,
    headless_interactions::{dispatch_key_tap, focus_test_id, node_id_by_test_id},
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_exposed_dropdown_suite_goldens_v1() {
    let exposed_suite = load_material3_exposed_dropdown_golden_suite_v1();
    let exposed_items = exposed_suite.items();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let closed_case = exposed_suite.closed_case();

            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
                let filled_selected = app.models_mut().insert(None);
                let outlined_query = app.models_mut().insert(String::from("Beta"));
                let filled_query = app.models_mut().insert(String::new());
                let items = exposed_items.clone();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let content = exposed_dropdown_pair(
                                cx,
                                items.clone(),
                                outlined_selected.clone(),
                                outlined_query.clone(),
                                filled_selected.clone(),
                                filled_query.clone(),
                            );
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 exposed dropdown closed scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    closed_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        16,
                        32,
                        &message,
                        &render,
                    ),
                );
            }

            for case in exposed_suite.open_cases() {
                let case_name = case.id();
                let context = format!("{label}, {scale}, {case_name}");

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
                let filled_selected = app.models_mut().insert(None);
                let outlined_query = app.models_mut().insert(String::from("Beta"));
                let filled_query = app.models_mut().insert(String::new());
                let items = exposed_items.clone();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let content = exposed_dropdown_pair(
                                cx,
                                items.clone(),
                                outlined_selected.clone(),
                                outlined_query.clone(),
                                filled_selected.clone(),
                                filled_query.clone(),
                            );
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let _ = node_id_by_test_id(&ui, case.trailing_icon_test_id(), &context);
                focus_test_id(&mut ui, case.target_test_id(), &context);
                dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                assert!(
                    stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open
                    }),
                    "expected exposed dropdown popover overlay to be registered open after ArrowDown ({context})"
                );

                let message = format!(
                    "expected the Material3 exposed dropdown overlay scene to be stable after animations settle ({context})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-exposed-dropdown.{scale}.{label}"),
                "material3_headless_exposed_dropdown_suite_goldens_v1",
                &suite,
            );
        }
    }

    fn exposed_dropdown_pair(
        cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
        items: Arc<[AutocompleteItem]>,
        outlined_selected: Model<Option<Arc<str>>>,
        outlined_query: Model<String>,
        filled_selected: Model<Option<Arc<str>>>,
        filled_query: Model<String>,
    ) -> fret_ui::element::AnyElement {
        let mut column = FlexProps::default();
        column.direction = fret_core::Axis::Vertical;
        column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));

        let outlined = ExposedDropdown::new(outlined_selected)
            .query(outlined_query)
            .variant(AutocompleteVariant::Outlined)
            .leading_icon(fret_icons::ids::ui::SEARCH)
            .label("Outlined")
            .placeholder("Pick one")
            .supporting_text("Committed value")
            .items(items.clone())
            .a11y_label("outlined exposed dropdown")
            .test_id("material3-exposed-outlined")
            .into_element(cx);
        let outlined = cx.container(
            {
                let mut props = ContainerProps::default();
                props.layout.size.width = Length::Px(Px(360.0));
                props
            },
            move |_cx| vec![outlined],
        );

        let filled = ExposedDropdown::new(filled_selected)
            .query(filled_query)
            .variant(AutocompleteVariant::Filled)
            .label("Filled")
            .placeholder("Pick one")
            .supporting_text("Editable query")
            .items(items)
            .a11y_label("filled exposed dropdown")
            .test_id("material3-exposed-filled")
            .into_element(cx);
        let filled = cx.container(
            {
                let mut props = ContainerProps::default();
                props.layout.size.width = Length::Px(Px(360.0));
                props
            },
            move |_cx| vec![filled],
        );

        cx.flex(column, |_cx| vec![outlined, filled])
    }
}
