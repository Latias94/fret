use std::{collections::BTreeMap, sync::Arc};

use fret_core::{AppWindowId, Axis, KeyCode, Px, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{
    UiTree,
    element::{AnyElement, CrossAlign, FlexProps, Length, MainAlign},
    elements::ElementContext,
};
use fret_ui_material3::{TabItem, TabPanel, Tabs, TabsVariant};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_interactions::{
        dispatch_idle_pointer, dispatch_key_tap, focus_test_id, hover_test_id,
    },
    headless_tabs_cases::{
        Material3TabsGoldenCaseKindV1, Material3TabsGoldenLayoutDirectionV1,
        load_material3_tabs_golden_suite_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::{apply_material_theme, apply_material_theme_rtl},
};

pub(crate) fn run_material3_headless_tabs_suite_goldens_v1() {
    let tabs_suite = load_material3_tabs_golden_suite_v1();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let bounds = tabs_suite.bounds();
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in tabs_suite.cases() {
                let case_name = case.id();
                let context = format!("{label}, {scale}, {case_name}");

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                match case.layout_direction() {
                    Material3TabsGoldenLayoutDirectionV1::Ltr => {
                        apply_material_theme(&mut app, mode, variant);
                    }
                    Material3TabsGoldenLayoutDirectionV1::Rtl => {
                        apply_material_theme_rtl(&mut app, mode, variant);
                    }
                }

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let primary_fixed = app.models_mut().insert(Arc::<str>::from("overview"));
                let secondary_fixed = app.models_mut().insert(Arc::<str>::from("settings"));
                let primary_scrollable = app.models_mut().insert(Arc::<str>::from("workspace"));
                let secondary_scrollable = app.models_mut().insert(Arc::<str>::from("analytics"));

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "tabs_root",
                        |cx| {
                            let content = tabs_matrix(
                                cx,
                                primary_fixed.clone(),
                                secondary_fixed.clone(),
                                primary_scrollable.clone(),
                                secondary_scrollable.clone(),
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
                    Material3TabsGoldenCaseKindV1::Idle => {}
                    Material3TabsGoldenCaseKindV1::Hover => {
                        let _ = hover_test_id(
                            &mut ui,
                            &mut app,
                            &mut services,
                            case.target_test_id(),
                            &context,
                        );
                    }
                    Material3TabsGoldenCaseKindV1::FocusVisible => {
                        focus_test_id(&mut ui, case.target_test_id(), &context);
                        dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::Tab);
                    }
                }

                let message = format!(
                    "expected the Material3 tabs {case_name} scene to be stable after animations settle ({label}, {scale})"
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
                &format!("material3-tabs.{scale}.{label}"),
                "material3_headless_tabs_suite_goldens_v1",
                &suite,
            );
        }
    }
}

fn tabs_matrix(
    cx: &mut ElementContext<'_, TestHost>,
    primary_fixed: Model<Arc<str>>,
    secondary_fixed: Model<Arc<str>>,
    primary_scrollable: Model<Arc<str>>,
    secondary_scrollable: Model<Arc<str>>,
) -> AnyElement {
    let mut column = FlexProps::default();
    column.layout.size.width = Length::Fill;
    column.layout.size.min_width = Some(Length::Px(Px(0.0)));
    column.direction = Axis::Vertical;
    column.justify = MainAlign::Start;
    column.align = CrossAlign::Stretch;
    column.gap = Px(18.0).into();

    cx.flex(column, move |cx| {
        vec![
            primary_fixed_tabs(cx, primary_fixed),
            secondary_fixed_tabs(secondary_fixed).into_element(cx),
            primary_scrollable_tabs(primary_scrollable).into_element(cx),
            secondary_scrollable_tabs(secondary_scrollable).into_element(cx),
        ]
    })
}

fn primary_fixed_tabs(
    cx: &mut ElementContext<'_, TestHost>,
    selected: Model<Arc<str>>,
) -> AnyElement {
    Tabs::new(selected)
        .a11y_label("Primary fixed tabs")
        .test_id("tabs-primary-fixed")
        .variant(TabsVariant::Primary)
        .content_fill_remaining(false)
        .items(vec![
            TabItem::new("overview", "Overview")
                .leading_icon(fret_icons::ids::ui::SEARCH)
                .test_id("tabs-primary-fixed-overview"),
            TabItem::new("details", "Details")
                .leading_icon(fret_icons::ids::ui::SETTINGS)
                .test_id("tabs-primary-fixed-details"),
            TabItem::new("disabled", "Disabled")
                .disabled(true)
                .test_id("tabs-primary-fixed-disabled"),
        ])
        .panels(vec![
            TabPanel::new("overview", [cx.text("Overview panel")])
                .test_id("tabs-primary-fixed-panel-overview"),
            TabPanel::new("details", [cx.text("Details panel")])
                .test_id("tabs-primary-fixed-panel-details"),
        ])
        .into_element(cx)
}

fn secondary_fixed_tabs(selected: Model<Arc<str>>) -> Tabs {
    Tabs::new(selected)
        .a11y_label("Secondary fixed tabs")
        .test_id("tabs-secondary-fixed")
        .secondary()
        .items(vec![
            TabItem::new("home", "Home").test_id("tabs-secondary-fixed-home"),
            TabItem::new("settings", "Settings").test_id("tabs-secondary-fixed-settings"),
            TabItem::new("profile", "Profile").test_id("tabs-secondary-fixed-profile"),
            TabItem::new("disabled", "Disabled")
                .disabled(true)
                .test_id("tabs-secondary-fixed-disabled"),
        ])
}

fn primary_scrollable_tabs(selected: Model<Arc<str>>) -> Tabs {
    Tabs::new(selected)
        .a11y_label("Primary scrollable tabs")
        .test_id("tabs-primary-scrollable")
        .scrollable(true)
        .items(vec![
            TabItem::new("workspace", "Workspace Settings")
                .stacked_icon(fret_icons::ids::ui::SETTINGS)
                .test_id("tabs-primary-scrollable-workspace"),
            TabItem::new("history", "History")
                .stacked_icon(fret_icons::ids::ui::SEARCH)
                .test_id("tabs-primary-scrollable-history"),
            TabItem::new("reports", "Reports")
                .stacked_icon(fret_icons::ids::ui::PLAY)
                .test_id("tabs-primary-scrollable-reports"),
            TabItem::new("disabled", "Disabled")
                .disabled(true)
                .stacked_icon(fret_icons::ids::ui::SLASH)
                .test_id("tabs-primary-scrollable-disabled"),
        ])
}

fn secondary_scrollable_tabs(selected: Model<Arc<str>>) -> Tabs {
    Tabs::new(selected)
        .a11y_label("Secondary scrollable tabs")
        .test_id("tabs-secondary-scrollable")
        .secondary()
        .scrollable(true)
        .items(vec![
            TabItem::new("analytics", "Analytics")
                .leading_icon(fret_icons::ids::ui::SEARCH)
                .test_id("tabs-secondary-scrollable-analytics"),
            TabItem::new("billing", "Billing")
                .leading_icon(fret_icons::ids::ui::SETTINGS)
                .test_id("tabs-secondary-scrollable-billing"),
            TabItem::new("exports", "Exports")
                .leading_icon(fret_icons::ids::ui::PLAY)
                .test_id("tabs-secondary-scrollable-exports"),
            TabItem::new("disabled", "Disabled")
                .disabled(true)
                .leading_icon(fret_icons::ids::ui::SLASH)
                .test_id("tabs-secondary-scrollable-disabled"),
        ])
}
