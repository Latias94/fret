use std::{collections::BTreeMap, sync::Arc};

use fret_core::{AppWindowId, PointerId, Px, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{
    UiTree,
    element::{CrossAlign, FlexProps, Length, MainAlign},
};
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::{
    Button, DropdownMenu, PlainTooltip, RichTooltip, Select, SelectItem, TooltipProvider,
};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    events::{pointer_down, pointer_up},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, run_overlay_frame_scaled,
        settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_interactions::hover_test_id,
    headless_overlay_cases::{
        Material3OverlaySelectTriggerV1, Material3OverlayTooltipKindV1,
        load_material3_overlay_golden_suite_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

fn select_from_fixture(
    selected: Model<Option<Arc<str>>>,
    trigger: &Material3OverlaySelectTriggerV1,
    items: Arc<[SelectItem]>,
) -> Select {
    let mut select = Select::new(selected)
        .label(trigger.label())
        .supporting_text(trigger.supporting_text())
        .a11y_label(trigger.a11y_label())
        .placeholder(trigger.placeholder())
        .items(items)
        .test_id(trigger.test_id());
    if let Some(icon) = trigger.leading_icon() {
        select = select.leading_icon(icon);
    }
    if trigger.error() {
        select = select.error(true);
    }
    select
}

pub(crate) fn run_material3_headless_overlays_suite_goldens_v1() {
    let overlay_suite = load_material3_overlay_golden_suite_v1();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in overlay_suite.tooltip_menu_cases() {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let open = app.models_mut().insert(true);
                let open_model = open.clone();

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
                            TooltipProvider::new()
                                .delay_duration_frames(0)
                                .skip_delay_duration_frames(0)
                                .with_elements(cx, |cx| {
                                    let tooltip_fixture = case.tooltip();
                                    let tooltip_trigger =
                                        Button::new(tooltip_fixture.trigger_label())
                                            .test_id(tooltip_fixture.trigger_test_id())
                                            .into_element(cx);
                                    let tooltip = match tooltip_fixture.kind() {
                                        Material3OverlayTooltipKindV1::Plain => PlainTooltip::new(
                                            tooltip_trigger,
                                            tooltip_fixture.supporting_text(),
                                        )
                                        .open_delay_frames(Some(0))
                                        .close_delay_frames(Some(0))
                                        .into_element(cx),
                                        Material3OverlayTooltipKindV1::Rich => RichTooltip::new(
                                            tooltip_trigger,
                                            tooltip_fixture.supporting_text(),
                                        )
                                        .title(tooltip_fixture.title())
                                        .open_delay_frames(Some(0))
                                        .close_delay_frames(Some(0))
                                        .into_element(cx),
                                    };

                                    let menu = DropdownMenu::new(open_model.clone())
                                        .a11y_label(case.menu().a11y_label())
                                        .test_id(case.menu().test_id())
                                        .into_element(
                                            cx,
                                            |cx| {
                                                Button::new(case.menu().trigger_label())
                                                    .test_id(case.menu().trigger_test_id())
                                                    .into_element(cx)
                                            },
                                            |_cx| case.menu_entries(),
                                        );

                                    let mut props = FlexProps::default();
                                    props.layout.size.width = Length::Fill;
                                    props.direction = fret_core::Axis::Horizontal;
                                    props.gap = fret_ui::element::SpacingLength::Px(Px(48.0));
                                    props.justify = MainAlign::SpaceBetween;
                                    props.align = CrossAlign::Center;

                                    let content = cx.flex(props, move |_cx| vec![tooltip, menu]);
                                    vec![with_padding(cx, case.padding(), content)]
                                })
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

                let context = format!("{} ({label}, {scale})", case.id());
                let _ = hover_test_id(
                    &mut ui,
                    &mut app,
                    &mut services,
                    case.tooltip().trigger_test_id(),
                    &context,
                );

                let mut opened = false;
                for _ in 0..case.open_wait_frames() {
                    run_overlay_frame_scaled(
                        &mut ui,
                        &mut app,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        false,
                        |ui, app, services| render(ui, app, services),
                    );

                    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                    let tooltip_open = stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Tooltip && entry.open && entry.visible
                    });
                    let menu_open = stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    });
                    if tooltip_open && menu_open {
                        opened = true;
                        break;
                    }
                }
                assert!(
                    opened,
                    "expected tooltip and menu overlays to be open ({context})"
                );

                let message = format!(
                    "expected the Material3 overlays scene to be stable after animations settle ({context})"
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
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let (
                select_open_snapshot,
                select_open_trigger_snapshot,
                select_open_hover_selected_snapshot,
            ) = {
                let case = overlay_suite.select_case();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();
                let selected: Model<Option<Arc<str>>> =
                    app.models_mut().insert(case.selected_value());
                let error_selected: Model<Option<Arc<str>>> =
                    app.models_mut().insert(case.error_selected_value());
                let items = case.select_items();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let selected = selected.clone();
                    let error_selected = error_selected.clone();
                    let items = items.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            props.align = CrossAlign::Start;

                            let select =
                                select_from_fixture(selected, case.select_trigger(), items.clone())
                                    .into_element(cx);

                            let select_error = select_from_fixture(
                                error_selected,
                                case.select_error_trigger(),
                                items.clone(),
                            )
                            .into_element(cx);

                            vec![cx.flex(props, move |_cx| vec![select, select_error])]
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

                let context = format!("{} ({label}, {scale})", case.id());
                let click_at = hover_test_id(
                    &mut ui,
                    &mut app,
                    &mut services,
                    case.select_trigger().test_id(),
                    &context,
                )
                .center;

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_down(PointerId(1), click_at),
                );
                ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

                let mut opened = false;
                for _ in 0..case.open_wait_frames() {
                    run_overlay_frame_scaled(
                        &mut ui,
                        &mut app,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        false,
                        |ui, app, services| render(ui, app, services),
                    );

                    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                    let select_open = stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    });
                    if select_open {
                        opened = true;
                        break;
                    }
                }
                assert!(
                    opened,
                    "expected the select overlay to be open after clicking the trigger ({context})"
                );

                let select_open_message = format!(
                    "expected the Material3 select overlay scene to be stable after animations settle ({context})"
                );
                let select_open_snapshot = settle_material3_overlay_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    case.open_settle_from_frame(),
                    case.open_total_frames(),
                    &select_open_message,
                    &render,
                );

                let select_open_trigger_message = format!(
                    "expected the Material3 select trigger to be stable in open state ({context})"
                );
                let select_open_trigger_snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.trigger_settle_from_frame(),
                    case.trigger_total_frames(),
                    &select_open_trigger_message,
                    &render,
                );

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

                let _ = hover_test_id(
                    &mut ui,
                    &mut app,
                    &mut services,
                    case.select_hover_selected_item_test_id(),
                    &context,
                );

                let select_hover_message = format!(
                    "expected the Material3 select overlay hover-selected scene to be stable after animations settle ({context})"
                );
                let select_open_hover_selected_snapshot =
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        case.hover_settle_from_frame(),
                        case.hover_total_frames(),
                        &select_hover_message,
                        &render,
                    );

                (
                    select_open_snapshot,
                    select_open_trigger_snapshot,
                    select_open_hover_selected_snapshot,
                )
            };

            cases.insert(
                overlay_suite
                    .select_case()
                    .select_open_snapshot_id()
                    .to_string(),
                select_open_snapshot,
            );
            cases.insert(
                overlay_suite
                    .select_case()
                    .select_trigger_snapshot_id()
                    .to_string(),
                select_open_trigger_snapshot,
            );
            cases.insert(
                overlay_suite
                    .select_case()
                    .select_hover_snapshot_id()
                    .to_string(),
                select_open_hover_selected_snapshot,
            );
            let suite = Material3HeadlessSuiteV1 { cases };

            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-overlays.{scale}.{label}"),
                "material3_headless_overlays_suite_goldens_v1",
                &suite,
            );
        }
    }
}
