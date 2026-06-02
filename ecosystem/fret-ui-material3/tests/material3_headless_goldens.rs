use std::{collections::BTreeMap, sync::Arc};

use fret_core::{
    AppWindowId, Edges, KeyCode, NodeId, Point, PointerId, Px, Rect, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{key_down, key_up, pointer_down, pointer_move, pointer_up};
use support::goldens::{
    Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, material3_scene_snapshot_v1,
    run_overlay_frame_scaled, run_overlay_frame_with_scene_scaled,
    settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
    snapshot_material3_scene_at_frame_v1, write_or_assert_material3_suite_for_test_v1,
};
use support::headless_autocomplete_cases::load_material3_autocomplete_golden_suite_v1;
use support::headless_carousel_item_cases::load_material3_carousel_item_golden_suite_v1;
use support::headless_interactions::{
    dispatch_idle_pointer, dispatch_key_tap, focus_test_id, hover_test_id,
};
use support::headless_list_cases::load_material3_list_golden_suite_v1;
use support::headless_menu_dialog_style_cases::{
    Material3MenuDialogStyleGoldenCaseKindV1, load_material3_menu_dialog_style_golden_suite_v1,
};
use support::headless_navigation_cases::load_material3_navigation_golden_suite_v1;
use support::headless_search_cases::load_material3_search_golden_suite_v1;
use support::headless_slider_cases::{
    Material3SliderKeyboardInteractionV1, Material3SliderPointerInteractionV1,
    load_material3_slider_golden_suite_v1,
};
use support::headless_snackbar_cases::load_material3_snackbar_golden_suite_v1;
use support::headless_text_field_cases::load_material3_text_field_golden_suite_v1;
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::{apply_material_theme, apply_material_theme_rtl};

// Broad Material3 headless golden suites live here so focused component tests do not
// inherit unrelated golden refresh churn. Keep Radio-specific behavior in radio_alignment.rs.

fn scale_segment(scale_factor: f32) -> &'static str {
    if (scale_factor - 1.0).abs() < 1e-6 {
        "scale1_0"
    } else if (scale_factor - 1.25).abs() < 1e-6 {
        "scale1_25"
    } else if (scale_factor - 2.0).abs() < 1e-6 {
        "scale2_0"
    } else {
        panic!("unsupported scale factor: {scale_factor}");
    }
}

#[test]
fn material3_headless_controls_suite_goldens_v1() {
    use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, Length, TextProps};
    use fret_ui_material3::{
        AssistChip, AssistChipVariant, Button, Card, CardVariant, Checkbox, FilterChip,
        FilterChipVariant, InputChip, Select, SelectItem, SuggestionChip, SuggestionChipVariant,
        Switch,
    };

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(560.0)),
            );

            let checkbox_checked = app.models_mut().insert(true);
            let checkbox_unchecked = app.models_mut().insert(false);
            let switch_on = app.models_mut().insert(true);
            let switch_off = app.models_mut().insert(false);
            let filter_chip_selected = app.models_mut().insert(true);
            let filter_chip_unselected = app.models_mut().insert(false);
            let input_chip_selected = app.models_mut().insert(true);
            let input_chip_unselected = app.models_mut().insert(false);
            let select_empty: Model<Option<Arc<str>>> = app.models_mut().insert(None);
            let select_populated: Model<Option<Arc<str>>> =
                app.models_mut().insert(Some(Arc::<str>::from("beta")));

            let select_items: Arc<[SelectItem]> = vec![
                SelectItem::new("alpha", "Alpha"),
                SelectItem::new("beta", "Beta"),
                SelectItem::new("charlie", "Charlie (disabled)").disabled(true),
            ]
            .into();

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                    props.align = CrossAlign::Start;
                    let content = cx.flex(props, |cx| {
                        let theme = Theme::global(&*cx.app).clone();
                        let body_style = theme
                            .text_style_by_key("md.sys.typescale.body-medium")
                            .unwrap_or_default();
                        let body_color = theme.color_token("md.sys.color.on-surface");

                        let card_content =
                            |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                             label: &'static str| {
                                let mut container = ContainerProps::default();
                                container.layout.size.width = Length::Px(Px(360.0));
                                container.layout.size.height = Length::Px(Px(72.0));
                                container.padding = Edges::all(Px(12.0)).into();

                                let mut text = TextProps::new(Arc::<str>::from(label));
                                text.style = Some(body_style.clone());
                                text.color = Some(body_color);

                                cx.container(container, move |cx| vec![cx.text_props(text)])
                            };

                        vec![
                            Button::new("Filled").test_id("btn-filled").into_element(cx),
                            Button::new("Filled (disabled)")
                                .disabled(true)
                                .test_id("btn-filled-disabled")
                                .into_element(cx),
                            Checkbox::new(checkbox_checked.clone())
                                .a11y_label("checkbox checked")
                                .test_id("cb-checked")
                                .into_element(cx),
                            Checkbox::new(checkbox_unchecked.clone())
                                .a11y_label("checkbox unchecked")
                                .test_id("cb-unchecked")
                                .into_element(cx),
                            Switch::new(switch_on.clone())
                                .a11y_label("switch on")
                                .test_id("sw-on")
                                .into_element(cx),
                            Switch::new(switch_off.clone())
                                .a11y_label("switch off")
                                .test_id("sw-off")
                                .into_element(cx),
                            AssistChip::new("Assist chip")
                                .test_id("chip-flat")
                                .into_element(cx),
                            AssistChip::new("Assist chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .variant(AssistChipVariant::Elevated)
                                .test_id("chip-elevated")
                                .into_element(cx),
                            SuggestionChip::new("Suggestion chip")
                                .test_id("chip-suggestion-flat")
                                .into_element(cx),
                            SuggestionChip::new("Suggestion chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .variant(SuggestionChipVariant::Elevated)
                                .test_id("chip-suggestion-elevated")
                                .into_element(cx),
                            FilterChip::new(filter_chip_selected.clone(), "Filter chip")
                                .test_id("chip-filter-selected")
                                .into_element(cx),
                            FilterChip::new(filter_chip_unselected.clone(), "Filter chip (icon)")
                                .trailing_icon(fret_icons::ids::ui::SLASH)
                                .variant(FilterChipVariant::Elevated)
                                .test_id("chip-filter-unselected-elevated")
                                .into_element(cx),
                            InputChip::new(input_chip_selected.clone(), "Input chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .test_id("chip-input-selected")
                                .into_element(cx),
                            InputChip::new(input_chip_unselected.clone(), "Input chip")
                                .trailing_icon(fret_icons::ids::ui::SLASH)
                                .test_id("chip-input-unselected")
                                .into_element(cx),
                            Card::new()
                                .variant(CardVariant::Filled)
                                .test_id("card-filled")
                                .into_element(cx, |cx| vec![card_content(cx, "Filled card")]),
                            Card::new()
                                .variant(CardVariant::Outlined)
                                .test_id("card-outlined")
                                .into_element(cx, |cx| vec![card_content(cx, "Outlined card")]),
                            Select::new(select_empty.clone())
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Select")
                                .supporting_text("Supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .test_id("sel-empty")
                                .into_element(cx),
                            Select::new(select_populated.clone())
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .label("Select")
                                .supporting_text("Supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .test_id("sel-populated")
                                .into_element(cx),
                            Select::new(select_empty.clone())
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Select")
                                .supporting_text("Error supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .error(true)
                                .test_id("sel-error")
                                .into_element(cx),
                        ]
                    });

                    vec![with_padding(cx, Px(24.0), content)]
                })
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

            let btn_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("btn-filled")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected btn-filled in semantics snapshot ({label}, {scale})")
                });

            let select_empty_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("sel-empty")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected sel-empty in semantics snapshot ({label}, {scale})")
                });
            let select_error_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("sel-error")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected sel-error in semantics snapshot ({label}, {scale})")
                });
            let btn_bounds = ui
                .debug_node_visual_bounds(btn_node)
                .unwrap_or_else(|| panic!("expected btn-filled bounds ({label}, {scale})"));
            let btn_center = Point::new(
                Px(btn_bounds.origin.x.0 + btn_bounds.size.width.0 * 0.5),
                Px(btn_bounds.origin.y.0 + btn_bounds.size.height.0 * 0.5),
            );
            let select_error_bounds = ui
                .debug_node_visual_bounds(select_error_node)
                .unwrap_or_else(|| panic!("expected sel-error bounds ({label}, {scale})"));
            let select_error_center = Point::new(
                Px(select_error_bounds.origin.x.0 + select_error_bounds.size.width.0 * 0.5),
                Px(select_error_bounds.origin.y.0 + select_error_bounds.size.height.0 * 0.5),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 controls idle scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            let render_select_supporting_text_insets =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "select_insets_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                vec![
                                    Select::new(select_empty.clone())
                                        .label("Select")
                                        .supporting_text("Supporting text")
                                        .placeholder("Pick one")
                                        .items(select_items.clone())
                                        .test_id("sel-inset-no-icon")
                                        .into_element(cx),
                                    Select::new(select_populated.clone())
                                        .leading_icon(fret_icons::ids::ui::SEARCH)
                                        .label("Select")
                                        .supporting_text("Supporting text")
                                        .placeholder("Pick one")
                                        .items(select_items.clone())
                                        .test_id("sel-inset-icon")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let select_supporting_inset_message = format!(
                "expected the Material3 select supporting text inset scenes to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle_select_supporting_text_insets".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_supporting_inset_message,
                    &render_select_supporting_text_insets,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), btn_center),
            );

            let hover_message = format!(
                "expected the Material3 controls hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover_btn_filled".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(btn_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 controls focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_btn_filled".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &focus_visible_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(select_empty_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let select_focus_visible_message = format!(
                "expected the Material3 select focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_select_empty".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_focus_visible_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), select_error_center),
            );

            let select_hover_message = format!(
                "expected the Material3 select hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover_select_error".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(select_error_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let select_error_focus_visible_message = format!(
                "expected the Material3 select error focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_select_error".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_error_focus_visible_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-controls.{scale}.{label}"),
                "material3_headless_controls_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_fab_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{Fab, FabSize, FabVariant};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(240.0)),
            );

            let render =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "fab_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                let row =
                                    |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                     variant: FabVariant,
                                     id_prefix: &'static str| {
                                        let mut props = FlexProps::default();
                                        props.direction = fret_core::Axis::Horizontal;
                                        props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                        cx.flex(props, move |cx| {
                                            vec![
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .a11y_label("fab")
                                                    .test_id(format!("{id_prefix}-fab"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Small)
                                                    .a11y_label("fab small")
                                                    .test_id(format!("{id_prefix}-fab-small"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Large)
                                                    .a11y_label("fab large")
                                                    .test_id(format!("{id_prefix}-fab-large"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .label("Create")
                                                    .test_id(format!("{id_prefix}-extended-fab"))
                                                    .into_element(cx),
                                            ]
                                        })
                                    };

                                vec![
                                    row(cx, FabVariant::Surface, "fab-surface"),
                                    row(cx, FabVariant::Primary, "fab-primary"),
                                ]
                            });

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

            let fab_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("fab-surface-fab")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected fab-surface-fab in semantics snapshot ({label}, {scale})")
                });

            let bounds_message =
                format!("expected fab-surface bounds in headless suite ({label}, {scale})");
            let fab_bounds = ui
                .debug_node_visual_bounds(fab_node)
                .unwrap_or_else(|| panic!("{bounds_message}"));
            let fab_center = Point::new(
                Px(fab_bounds.origin.x.0 + fab_bounds.size.width.0 * 0.5),
                Px(fab_bounds.origin.y.0 + fab_bounds.size.height.0 * 0.5),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 fab idle scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), fab_center),
            );
            let hover_message = format!(
                "expected the Material3 fab hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(fab_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 fab focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &focus_visible_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-fab.{scale}.{label}"),
                "material3_headless_fab_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_segmented_button_suite_goldens_v1() {
    use std::collections::BTreeSet;

    use fret_ui::element::FlexProps;
    use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(260.0)),
            );

            let single_value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("alpha"));
            let multi_value: Model<BTreeSet<Arc<str>>> = app.models_mut().insert(
                [Arc::<str>::from("alpha"), Arc::<str>::from("beta")]
                    .into_iter()
                    .collect(),
            );

            let render =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "segmented_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                vec![
                                    SegmentedButtonSet::single(single_value.clone())
                                        .items(vec![
                                            SegmentedButtonItem::new("alpha", "Alpha")
                                                .test_id("segmented-single-alpha"),
                                            SegmentedButtonItem::new("beta", "Beta")
                                                .test_id("segmented-single-beta"),
                                            SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                                                .disabled(true)
                                                .test_id("segmented-single-gamma"),
                                        ])
                                        .a11y_label("Single segmented buttons")
                                        .test_id("segmented-single")
                                        .into_element(cx),
                                    SegmentedButtonSet::multi(multi_value.clone())
                                        .items(vec![
                                            SegmentedButtonItem::new("alpha", "Alpha")
                                                .icon(fret_icons::ids::ui::SEARCH)
                                                .test_id("segmented-multi-alpha"),
                                            SegmentedButtonItem::new("beta", "Beta")
                                                .icon(fret_icons::ids::ui::SETTINGS)
                                                .test_id("segmented-multi-beta"),
                                            SegmentedButtonItem::new("gamma", "Gamma")
                                                .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                                                .test_id("segmented-multi-gamma"),
                                        ])
                                        .a11y_label("Multi segmented buttons")
                                        .test_id("segmented-multi")
                                        .into_element(cx),
                                ]
                            });

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

            let hover_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("segmented-single-beta"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected segmented-single-beta in semantics snapshot ({label}, {scale})"
                    )
                });

            let hover_bounds = ui.debug_node_visual_bounds(hover_node).unwrap_or_else(|| {
                panic!("expected segmented-single-beta bounds in headless suite ({label}, {scale})")
            });
            let hover_center = Point::new(
                Px(hover_bounds.origin.x.0 + hover_bounds.size.width.0 * 0.5),
                Px(hover_bounds.origin.y.0 + hover_bounds.size.height.0 * 0.5),
            );

            let focus_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("segmented-single-alpha"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected segmented-single-alpha in semantics snapshot ({label}, {scale})"
                    )
                });

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 segmented button idle scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), hover_center),
            );
            let hover_message = format!(
                "expected the Material3 segmented button hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(focus_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 segmented button focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &focus_visible_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-segmented-button.{scale}.{label}"),
                "material3_headless_segmented_button_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_badge_suite_goldens_v1() {
    use fret_core::Corners;
    use fret_ui::element::{ContainerProps, FlexProps, Length};
    use fret_ui_material3::{Badge, BadgePlacement};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(200.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "badge_root",
                    |cx| {
                        let theme = Theme::global(&*cx.app).clone();
                        let anchor_color = theme.color_token("md.sys.color.surface-container-low");

                        let anchor = |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                      size: Px| {
                            let mut props = ContainerProps::default();
                            props.layout.size.width = Length::Px(size);
                            props.layout.size.height = Length::Px(size);
                            props.background = Some(anchor_color);
                            props.corner_radii = Corners::all(Px(8.0));
                            cx.container(props, |_cx| Vec::<AnyElement>::new())
                        };

                        let mut props = FlexProps::default();
                        props.direction = fret_core::Axis::Horizontal;
                        props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                        props.align = fret_ui::element::CrossAlign::Center;
                        props.wrap = false;

                        let content = cx.flex(props, |cx| {
                            let small = Px(24.0);
                            vec![
                                Badge::dot()
                                    .navigation_anchor_size(small)
                                    .test_id("badge-dot-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::text("9")
                                    .navigation_anchor_size(small)
                                    .test_id("badge-text-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::dot()
                                    .placement(BadgePlacement::TopRight)
                                    .anchor_size(Px(40.0))
                                    .test_id("badge-dot-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                                Badge::text("99+")
                                    .placement(BadgePlacement::TopRight)
                                    .anchor_size(Px(40.0))
                                    .test_id("badge-text-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                            ]
                        });

                        vec![with_padding(cx, Px(24.0), content)]
                    },
                )
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 badge scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-badge.{scale}.{label}"),
                "material3_headless_badge_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_top_app_bar_suite_goldens_v1() {
    use fret_icons::ids;
    use fret_ui::element::ContainerProps;
    use fret_ui_material3::{TopAppBar, TopAppBarAction, TopAppBarVariant};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(220.0)),
            );

            let make_actions = |extra: usize| -> Vec<TopAppBarAction> {
                let mut actions = vec![
                    TopAppBarAction::new(ids::ui::SEARCH)
                        .a11y_label("Search")
                        .test_id("top-app-bar-search"),
                    TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                        .a11y_label("More actions")
                        .test_id("top-app-bar-more"),
                ];
                if extra >= 1 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::SETTINGS)
                            .a11y_label("Settings")
                            .test_id("top-app-bar-settings"),
                    );
                }
                if extra >= 2 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::PLAY)
                            .a11y_label("Play")
                            .test_id("top-app-bar-play"),
                    );
                }
                actions
            };

            let mut snapshot_case =
                |case_label: &'static str,
                 variant: TopAppBarVariant,
                 scrolled: bool,
                 actions: Vec<TopAppBarAction>| {
                    let render = |ui: &mut UiTree<TestHost>,
                                  app: &mut TestHost,
                                  services: &mut dyn UiServices| {
                        let actions = actions.clone();
                        fret_ui::declarative::render_root(
                            ui,
                            app,
                            services,
                            window,
                            bounds,
                            "top_app_bar_root",
                            move |cx| {
                                let theme = Theme::global(&*cx.app).clone();

                                let mut bg = ContainerProps::default();
                                bg.layout.size.width = fret_ui::element::Length::Fill;
                                bg.layout.size.height = fret_ui::element::Length::Fill;
                                bg.background = Some(theme.color_token("md.sys.color.background"));

                                let bar = TopAppBar::new(case_label)
                                    .variant(variant)
                                    .scrolled(scrolled)
                                    .navigation_icon(
                                        TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                                            .a11y_label("Navigate")
                                            .test_id("top-app-bar-nav"),
                                    )
                                    .actions(actions)
                                    .test_id("top-app-bar");

                                vec![cx.container(bg, move |cx| vec![bar.into_element(cx)])]
                            },
                        )
                    };

                    let stable_message = format!(
                        "expected the Material3 top app bar scene to be stable after animations settle ({label}, {scale}, {case_label})"
                    );
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &stable_message,
                        &render,
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert(
                "small.idle".to_string(),
                snapshot_case("Small", TopAppBarVariant::Small, false, make_actions(0)),
            );
            cases.insert(
                "small.scrolled".to_string(),
                snapshot_case(
                    "Small (scrolled)",
                    TopAppBarVariant::Small,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.idle".to_string(),
                snapshot_case(
                    "Small Centered",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.scrolled".to_string(),
                snapshot_case(
                    "Small Centered (scrolled)",
                    TopAppBarVariant::SmallCentered,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.wide_actions".to_string(),
                snapshot_case(
                    "Small Centered (wide actions)",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(2),
                ),
            );
            cases.insert(
                "medium.idle".to_string(),
                snapshot_case("Medium", TopAppBarVariant::Medium, false, make_actions(0)),
            );
            cases.insert(
                "medium.scrolled".to_string(),
                snapshot_case(
                    "Medium (scrolled)",
                    TopAppBarVariant::Medium,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "large.idle".to_string(),
                snapshot_case("Large", TopAppBarVariant::Large, false, make_actions(0)),
            );
            cases.insert(
                "large.scrolled".to_string(),
                snapshot_case(
                    "Large (scrolled)",
                    TopAppBarVariant::Large,
                    true,
                    make_actions(0),
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-top-app-bar.{scale}.{label}"),
                "material3_headless_top_app_bar_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_navigation_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, ModalNavigationDrawer, NavigationDrawerVariant,
    };

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
    let navigation_suite = load_material3_navigation_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // NavigationBar: horizontal destinations with badges.
            {
                let case = navigation_suite.case("bar.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let bar = case.navigation_bar(value.clone()).into_element(cx);

                            vec![with_padding(cx, Px(24.0), bar)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation bar scene to be stable ({label}, {scale})"
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

            // NavigationRail: vertical destinations with disabled item.
            {
                let case = navigation_suite.case("rail.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let rail = case.navigation_rail(value.clone()).into_element(cx);

                            vec![with_padding(cx, Px(24.0), rail)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation rail scene to be stable ({label}, {scale})"
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

            // NavigationDrawer: pill selection + badges.
            {
                let case = navigation_suite.case("drawer.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let drawer = case
                                .navigation_drawer(value.clone(), NavigationDrawerVariant::Standard)
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), drawer)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation drawer scene to be stable ({label}, {scale})"
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

            // ModalNavigationDrawer: overlay open (scrim + focus trap surface).
            {
                let case = navigation_suite.case("modal_drawer.open");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let open = app.models_mut().insert(true);
                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let value = value.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let panel_value = value.clone();
                            let panel =
                                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                                    case.navigation_drawer(
                                        panel_value.clone(),
                                        NavigationDrawerVariant::Modal,
                                    )
                                    .into_element(cx)
                                };

                            let underlay =
                                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                                    Button::new(case.underlay_label())
                                        .variant(ButtonVariant::Outlined)
                                        .test_id(case.underlay_test_id())
                                        .into_element(cx)
                                };

                            let modal = ModalNavigationDrawer::new(open.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id(case.modal_test_id())
                                .into_element(cx, panel, underlay);

                            vec![with_padding(cx, Px(24.0), modal)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal navigation drawer overlay scene to be stable ({label}, {scale})"
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

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-navigation.{scale}.{label}"),
                "material3_headless_navigation_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_snackbar_suite_goldens_v1() {
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{SnackbarController, SnackbarHost};

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
    let snackbar_suite = load_material3_snackbar_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let snapshot_case =
                |case: &support::headless_snackbar_cases::Material3SnackbarGoldenCaseV1| {
                    let mut app = TestHost::default();
                    app.set_global(PlatformCapabilities::default());
                    apply_material_theme(&mut app, mode, variant);

                    let window = AppWindowId::default();
                    let mut services = FakeUiServices;
                    let mut ui: UiTree<TestHost> = UiTree::new();
                    ui.set_window(window);

                    let bounds = Rect::new(
                        Point::new(Px(0.0), Px(0.0)),
                        Size::new(Px(860.0), Px(520.0)),
                    );

                    let store = app.models_mut().insert(ToastStore::default());
                    let controller = SnackbarController::new(store.clone());
                    {
                        let mut action_host =
                            fret_ui::action::UiActionHostAdapter { app: &mut app };
                        let _id = controller.show(&mut action_host, window, case.to_snackbar());
                    }

                    let render =
                        move |ui: &mut UiTree<TestHost>,
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
                                    let host_layer = SnackbarHost::new(store.clone())
                                        .max_snackbars(1)
                                        .into_element(cx);

                                    vec![with_padding(cx, Px(24.0), host_layer)]
                                },
                            )
                        };

                    let message = format!(
                        "expected the Material3 snackbar overlay scene to be stable ({label}, {scale}, {})",
                        case.id()
                    );
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &message,
                        &render,
                    )
                };

            for case in snackbar_suite.cases() {
                cases.insert(case.id().to_string(), snapshot_case(case));
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-snackbar.{scale}.{label}"),
                "material3_headless_snackbar_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_divider_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, SpacerProps};
    use fret_ui_material3::Divider;

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(300.0), Px(220.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                    let content = cx.flex(props, |cx| {
                        let mut row = FlexProps::default();
                        row.direction = fret_core::Axis::Horizontal;
                        row.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                        row.layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                        row.layout.size.height = fret_ui::element::Length::Px(Px(32.0));

                        vec![
                            Divider::horizontal()
                                .test_id("divider-horizontal")
                                .into_element(cx),
                            cx.flex(row, |cx| {
                                vec![
                                    cx.spacer(SpacerProps::default()),
                                    Divider::vertical()
                                        .test_id("divider-vertical")
                                        .into_element(cx),
                                    cx.spacer(SpacerProps::default()),
                                ]
                            }),
                        ]
                    });

                    vec![with_padding(cx, Px(24.0), content)]
                })
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 divider scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    12,
                    24,
                    &idle_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-divider.{scale}.{label}"),
                "material3_headless_divider_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_list_suite_goldens_v1() {
    use fret_ui::element::FlexProps;

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
    let list_suite = load_material3_list_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in list_suite.cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(520.0), Px(420.0)),
                );

                let selected = app.models_mut().insert(list_suite.list().selected_value());

                let render = |ui: &mut UiTree<TestHost>,
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
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                            let list = list_suite.list().list(selected.clone()).into_element(cx);

                            let content = cx.flex(props, |_cx| vec![list]);
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                if case.is_idle() {
                    dispatch_idle_pointer(&mut ui, &mut app, &mut services);
                }

                if let Some(test_id) = case.hover_test_id() {
                    hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);
                    dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
                }

                let message = format!(
                    "expected list scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                let snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.settle_from_frame(),
                    case.total_frames(),
                    &message,
                    &render,
                );
                cases.insert(case.id().to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-list.{scale}.{label}"),
                "material3_headless_list_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_progress_indicator_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{CircularProgressIndicator, LinearProgressIndicator};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(260.0)),
            );

            let progress_0 = app.models_mut().insert(0.0f32);
            let progress_30 = app.models_mut().insert(0.3f32);
            let progress_100 = app.models_mut().insert(1.0f32);

            let render_determinate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::new(progress_0.clone())
                                        .test_id("linear-0")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_30.clone())
                                        .test_id("linear-30")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_100.clone())
                                        .test_id("linear-100")
                                        .into_element(cx),
                                    {
                                        let mut row = FlexProps::default();
                                        row.direction = fret_core::Axis::Horizontal;
                                        row.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                        cx.flex(row, |cx| {
                                            vec![
                                                CircularProgressIndicator::new(progress_0.clone())
                                                    .test_id("circular-0")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(progress_30.clone())
                                                    .test_id("circular-30")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(
                                                    progress_100.clone(),
                                                )
                                                .test_id("circular-100")
                                                .into_element(cx),
                                            ]
                                        })
                                    },
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .test_id("linear-indeterminate")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .test_id("circular-indeterminate")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate_four_color =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("linear-indeterminate-four-color")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("circular-indeterminate-four-color")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 progress indicator scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    12,
                    24,
                    &idle_message,
                    &render_determinate,
                ),
            );

            cases.insert(
                "indeterminate.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate,
                ),
            );

            cases.insert(
                "indeterminate.four_color.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate_four_color,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-progress-indicator.{scale}.{label}"),
                "material3_headless_progress_indicator_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_slider_suite_goldens_v1() {
    use fret_ui::element::FlexProps;

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
    let slider_suite = load_material3_slider_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(520.0), Px(320.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in slider_suite.cases() {
                let case_name = case.id();
                let window = AppWindowId::default();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                if case.is_rtl() {
                    apply_material_theme_rtl(&mut app, mode, variant);
                } else {
                    apply_material_theme(&mut app, mode, variant);
                }

                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let single_value_models = slider_suite
                    .single_value_models()
                    .iter()
                    .map(|definition| {
                        (
                            definition.id().to_string(),
                            app.models_mut().insert(definition.value()),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let range_value_models = slider_suite
                    .range_value_models()
                    .iter()
                    .map(|definition| {
                        (
                            definition.id().to_string(),
                            app.models_mut().insert(definition.values()),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let render_config = slider_suite.render_config_for(case);

                let render = |ui: &mut UiTree<TestHost>,
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
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                            let content = cx.flex(props, |cx| {
                                let mut elements = Vec::new();

                                for slider in slider_suite.sliders() {
                                    let model = single_value_models
                                        .get(slider.model_id())
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "expected single slider model {} ({label}, {scale}, {case_name})",
                                                slider.model_id()
                                            )
                                        })
                                        .clone();
                                    elements.push(slider.slider(model, render_config).into_element(cx));
                                }

                                for slider in slider_suite.range_sliders() {
                                    let model = range_value_models
                                        .get(slider.model_id())
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "expected range slider model {} ({label}, {scale}, {case_name})",
                                                slider.model_id()
                                            )
                                        })
                                        .clone();
                                    elements.push(
                                        slider.range_slider(model, render_config).into_element(cx),
                                    );
                                }

                                elements
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                dispatch_idle_pointer(&mut ui, &mut app, &mut services);

                if let Some(test_id) = case.hover_test_id() {
                    let hover_target = hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                    let node_bounds = hover_target.bounds;
                    let hover_at = hover_target.center;

                    match case.pointer_interaction() {
                        Some(Material3SliderPointerInteractionV1::Pressed) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), hover_at),
                            );
                        }
                        Some(Material3SliderPointerInteractionV1::Dragging) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), hover_at),
                            );
                            let drag_to = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.8),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), drag_to),
                            );
                        }
                        Some(Material3SliderPointerInteractionV1::RangeDragging) => {
                            let start_at = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.85),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            let drag_to = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.95),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), start_at),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), start_at),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), drag_to),
                            );
                        }
                        None => {}
                    }
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);

                    match case.keyboard_interaction() {
                        Some(Material3SliderKeyboardInteractionV1::SingleArrowCycle) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                        }
                        Some(Material3SliderKeyboardInteractionV1::SinglePageHomeEnd) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected single slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let value_model =
                                single_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected single slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_page_up = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_page_up - 0.4).abs() <= 1e-6,
                                "expected slider PageUp to increment by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_page_down = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_page_down - 0.3).abs() <= 1e-6,
                                "expected slider PageDown to decrement by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_home = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                after_home.abs() <= 1e-6,
                                "expected slider Home to snap to min (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                            let after_end = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_end - 1.0).abs() <= 1e-6,
                                "expected slider End to snap to max (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::SingleRtlArrowCycle) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected single slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let value_model =
                                single_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected single slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            let after_right = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_right - 0.29).abs() <= 1e-6,
                                "expected slider ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                            let after_left = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_left - 0.30).abs() <= 1e-6,
                                "expected slider ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangeThumbSwitch) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );

                            let end_test_id = case.secondary_focus_test_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider secondary focus test id ({label}, {scale}, {case_name})"
                                )
                            });
                            focus_test_id(&mut ui, end_test_id, &interaction_context);

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangePageHomeEnd) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let range_model =
                                range_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected range slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_page_up = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_page_up[0] - 0.4).abs() <= 1e-6
                                    && (after_page_up[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start PageUp to increment start by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_page_down = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_page_down[0] - 0.3).abs() <= 1e-6
                                    && (after_page_down[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start PageDown to decrement start by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_home = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_home[0].abs() <= 1e-6 && (after_home[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start Home to snap to min (case={case_name}, {label}, {scale})"
                            );

                            let end_test_id = case.secondary_focus_test_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider secondary focus test id ({label}, {scale}, {case_name})"
                                )
                            });
                            focus_test_id(&mut ui, end_test_id, &interaction_context);

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_end_page_down = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_page_down[0].abs() <= 1e-6
                                    && (after_end_page_down[1] - 0.6).abs() <= 1e-6,
                                "expected range slider end PageDown to decrement end by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_end_page_up = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_page_up[0].abs() <= 1e-6
                                    && (after_end_page_up[1] - 0.7).abs() <= 1e-6,
                                "expected range slider end PageUp to increment end by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_end_home = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_home[0].abs() <= 1e-6 && after_end_home[1].abs() <= 1e-6,
                                "expected range slider end Home to snap to start value (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                            let after_end_end = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_end[0].abs() <= 1e-6
                                    && (after_end_end[1] - 1.0).abs() <= 1e-6,
                                "expected range slider end End to snap to max (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangeRtlArrowCycle) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let range_model =
                                range_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected range slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            let after_right = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_right[0] - 0.29).abs() <= 1e-6
                                    && (after_right[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                            let after_left = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_left[0] - 0.30).abs() <= 1e-6
                                    && (after_left[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                            );
                        }
                        None => {}
                    }
                }

                let message = format!(
                    "expected the Material3 slider scene to be stable after animations settle ({label}, {scale}, {case_name})"
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
                &format!("material3-slider.{scale}.{label}"),
                "material3_headless_slider_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
#[ignore = "stale broad headless golden; use menu/dialog/tooltip/select state gates for default coverage and run explicitly when refreshing material3-overlays goldens"]
fn material3_headless_overlays_suite_goldens_v1() {
    use fret_ui::element::{CrossAlign, FlexProps, Length, MainAlign};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::menu::{MenuEntry, MenuItem};
    use fret_ui_material3::{
        Button, DropdownMenu, PlainTooltip, RichTooltip, Select, SelectItem, TooltipProvider,
    };

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let open = app.models_mut().insert(true);
            let open_model = open.clone();

            let render = move |ui: &mut UiTree<TestHost>,
                               app: &mut TestHost,
                               services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with_elements(cx, |cx| {
                            let tooltip_trigger = Button::new("Tooltip")
                                .test_id("tooltip-trigger")
                                .into_element(cx);
                            let tooltip = PlainTooltip::new(tooltip_trigger, "Tip")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .into_element(cx);

                            let menu = DropdownMenu::new(open_model.clone())
                                .a11y_label("menu")
                                .test_id("dropdown")
                                .into_element(
                                    cx,
                                    |cx| {
                                        Button::new("Menu")
                                            .test_id("dropdown-trigger")
                                            .into_element(cx)
                                    },
                                    |_cx| {
                                        vec![
                                            MenuEntry::Item(
                                                MenuItem::new("A").test_id("dropdown-item-a"),
                                            ),
                                            MenuEntry::Item(
                                                MenuItem::new("B").test_id("dropdown-item-b"),
                                            ),
                                            MenuEntry::Item(
                                                MenuItem::new("C").test_id("dropdown-item-c"),
                                            ),
                                        ]
                                    },
                                );

                            let mut props = FlexProps::default();
                            props.layout.size.width = Length::Fill;
                            props.direction = fret_core::Axis::Horizontal;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(48.0));
                            props.justify = MainAlign::SpaceBetween;
                            props.align = CrossAlign::Center;

                            let content = cx.flex(props, move |_cx| vec![tooltip, menu]);
                            vec![with_padding(cx, Px(24.0), content)]
                        })
                })
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

            let tooltip_trigger_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("tooltip-trigger")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected tooltip-trigger in semantics snapshot ({label}, {scale})")
                });
            let tooltip_trigger_bounds = ui
                .debug_node_visual_bounds(tooltip_trigger_node)
                .expect("expected tooltip-trigger bounds");
            let hover_at = Point::new(
                Px(tooltip_trigger_bounds.origin.x.0 + tooltip_trigger_bounds.size.width.0 * 0.5),
                Px(tooltip_trigger_bounds.origin.y.0 + tooltip_trigger_bounds.size.height.0 * 0.5),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), hover_at),
            );

            let mut opened = false;
            for _ in 0..12 {
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
                "expected both tooltip and menu overlays to be open ({label}, {scale})"
            );

            let mut settled: Option<Material3HeadlessGoldenV1> = None;
            for frame in 0..80 {
                let scene = run_overlay_frame_with_scene_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                if frame < 44 {
                    continue;
                }

                let snapshot = material3_scene_snapshot_v1(&scene);
                if let Some(prev) = settled.as_ref() {
                    assert_eq!(
                        snapshot, *prev,
                        "expected the Material3 overlays scene to be stable after animations settle ({label}, {scale})"
                    );
                } else {
                    settled = Some(snapshot);
                }
            }

            let Some(both_open_snapshot) = settled else {
                panic!("expected a settled overlays snapshot ({label}, {scale})");
            };

            let rich_both_open_snapshot = {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

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
                                    let tooltip_trigger = Button::new("Rich tooltip")
                                        .test_id("tooltip-trigger")
                                        .into_element(cx);
                                    let tooltip =
                                        RichTooltip::new(tooltip_trigger, "Supporting text")
                                            .title("Title")
                                            .open_delay_frames(Some(0))
                                            .close_delay_frames(Some(0))
                                            .into_element(cx);

                                    let menu = DropdownMenu::new(open_model.clone())
                                        .a11y_label("menu")
                                        .test_id("dropdown")
                                        .into_element(
                                            cx,
                                            |cx| {
                                                Button::new("Menu")
                                                    .test_id("dropdown-trigger")
                                                    .into_element(cx)
                                            },
                                            |_cx| {
                                                vec![
                                                    MenuEntry::Item(
                                                        MenuItem::new("A")
                                                            .test_id("dropdown-item-a"),
                                                    ),
                                                    MenuEntry::Item(
                                                        MenuItem::new("B")
                                                            .test_id("dropdown-item-b"),
                                                    ),
                                                    MenuEntry::Item(
                                                        MenuItem::new("C")
                                                            .test_id("dropdown-item-c"),
                                                    ),
                                                ]
                                            },
                                        );

                                    let mut props = FlexProps::default();
                                    props.layout.size.width = Length::Fill;
                                    props.direction = fret_core::Axis::Horizontal;
                                    props.gap = fret_ui::element::SpacingLength::Px(Px(48.0));
                                    props.justify = MainAlign::SpaceBetween;
                                    props.align = CrossAlign::Center;

                                    let content = cx.flex(props, move |_cx| vec![tooltip, menu]);
                                    vec![with_padding(cx, Px(24.0), content)]
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

                let tooltip_trigger_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some("tooltip-trigger")).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!("expected tooltip-trigger in semantics snapshot ({label}, {scale})")
                    });
                let tooltip_trigger_bounds = ui
                    .debug_node_visual_bounds(tooltip_trigger_node)
                    .expect("expected tooltip-trigger bounds");
                let hover_at = Point::new(
                    Px(tooltip_trigger_bounds.origin.x.0
                        + tooltip_trigger_bounds.size.width.0 * 0.5),
                    Px(tooltip_trigger_bounds.origin.y.0
                        + tooltip_trigger_bounds.size.height.0 * 0.5),
                );

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), hover_at),
                );

                let mut opened = false;
                for _ in 0..12 {
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
                    "expected both rich tooltip and menu overlays to be open ({label}, {scale})"
                );

                let mut settled: Option<Material3HeadlessGoldenV1> = None;
                for frame in 0..80 {
                    let scene = run_overlay_frame_with_scene_scaled(
                        &mut ui,
                        &mut app,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        false,
                        |ui, app, services| render(ui, app, services),
                    );

                    if frame < 44 {
                        continue;
                    }

                    let snapshot = material3_scene_snapshot_v1(&scene);
                    if let Some(prev) = settled.as_ref() {
                        assert_eq!(
                            snapshot, *prev,
                            "expected the Material3 rich tooltip overlays scene to be stable after animations settle ({label}, {scale})"
                        );
                    } else {
                        settled = Some(snapshot);
                    }
                }

                settled.unwrap_or_else(|| {
                    panic!("expected a settled rich tooltip overlays snapshot ({label}, {scale})")
                })
            };

            let (
                select_open_snapshot,
                select_open_trigger_snapshot,
                select_open_hover_selected_snapshot,
            ) = {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let selected: Model<Option<Arc<str>>> =
                    app.models_mut().insert(Some(Arc::<str>::from("beta")));
                let error_selected: Model<Option<Arc<str>>> = app.models_mut().insert(None);

                let items: Arc<[SelectItem]> = vec![
                    SelectItem::new("alpha", "Alpha")
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("select-item-alpha"),
                    SelectItem::new("beta", "Beta")
                        .leading_icon(fret_icons::ids::ui::SETTINGS)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("select-item-beta"),
                    SelectItem::new("charlie", "Charlie (disabled)")
                        .disabled(true)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .test_id("select-item-charlie-disabled"),
                ]
                .into();

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

                            let select = Select::new(selected)
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Label")
                                .supporting_text("Supporting text")
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .items(items.clone())
                                .test_id("material3-select-trigger")
                                .into_element(cx);

                            let select_error = Select::new(error_selected)
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .label("Label")
                                .supporting_text("Error supporting text")
                                .a11y_label("select error")
                                .placeholder("Pick one")
                                .items(items.clone())
                                .error(true)
                                .test_id("material3-select-trigger-error")
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

                let Some(snapshot) = ui.semantics_snapshot() else {
                    panic!(
                        "expected semantics snapshot for select overlay case ({label}, {scale})"
                    );
                };

                let select_trigger_node = snapshot
                    .nodes
                    .iter()
                    .find_map(|node| {
                        (node.test_id.as_deref() == Some("material3-select-trigger"))
                            .then_some(node.id)
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "expected material3-select-trigger in semantics snapshot ({label}, {scale})"
                        )
                    });

                let select_trigger_bounds = ui
                    .debug_node_visual_bounds(select_trigger_node)
                    .expect("expected select trigger bounds");
                let click_at = Point::new(
                    Px(select_trigger_bounds.origin.x.0 + select_trigger_bounds.size.width.0 * 0.5),
                    Px(select_trigger_bounds.origin.y.0
                        + select_trigger_bounds.size.height.0 * 0.5),
                );

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_down(PointerId(1), click_at),
                );
                ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

                let mut opened = false;
                for _ in 0..12 {
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
                    "expected the select overlay to be open after clicking the trigger ({label}, {scale})"
                );

                let select_open_message = format!(
                    "expected the Material3 select overlay scene to be stable after animations settle ({label}, {scale})"
                );
                let select_open_snapshot = settle_material3_overlay_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    44,
                    80,
                    &select_open_message,
                    &render,
                );

                let select_open_trigger_message = format!(
                    "expected the Material3 select trigger to be stable in open state ({label}, {scale})"
                );
                let select_open_trigger_snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
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

                let selected_item_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!("expected select-item-beta in semantics snapshot ({label}, {scale})")
                    });
                let selected_item_bounds = ui
                    .debug_node_visual_bounds(selected_item_node)
                    .unwrap_or_else(|| {
                        panic!("expected select-item-beta bounds ({label}, {scale})")
                    });
                let hover_at = Point::new(
                    Px(selected_item_bounds.origin.x.0 + selected_item_bounds.size.width.0 * 0.5),
                    Px(selected_item_bounds.origin.y.0 + selected_item_bounds.size.height.0 * 0.5),
                );

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), hover_at),
                );

                let select_hover_message = format!(
                    "expected the Material3 select overlay hover-selected scene to be stable after animations settle ({label}, {scale})"
                );
                let select_open_hover_selected_snapshot =
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &select_hover_message,
                        &render,
                    );

                (
                    select_open_snapshot,
                    select_open_trigger_snapshot,
                    select_open_hover_selected_snapshot,
                )
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert("both_open".to_string(), both_open_snapshot);
            cases.insert("rich_both_open".to_string(), rich_both_open_snapshot);
            cases.insert("select_open".to_string(), select_open_snapshot);
            cases.insert(
                "select_open_trigger".to_string(),
                select_open_trigger_snapshot,
            );
            cases.insert(
                "select_open_hover_selected".to_string(),
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

#[test]
fn material3_headless_autocomplete_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteVariant};

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
    let autocomplete_suite = load_material3_autocomplete_golden_suite_v1();
    let autocomplete_items = autocomplete_suite.items();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let closed_case = autocomplete_suite.closed_case();

            // Closed scene: show both variants so token drift is visible.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items = autocomplete_items.clone();

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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 autocomplete closed scene to be stable after animations settle ({label}, {scale})"
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

            for case in autocomplete_suite.open_cases() {
                let case_name = case.id();
                let focus_test_id = case.focus_test_id();

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items = autocomplete_items.clone();

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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
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

                let input_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some(focus_test_id)).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "expected {focus_test_id} input node in semantics snapshot ({label}, {scale}, {case_name})"
                        )
                    });

                ui.set_focus(Some(input_node));
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

                ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
                ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

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
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    }),
                    "expected autocomplete popover overlay to be open after ArrowDown ({label}, {scale}, {case_name})"
                );

                let message = format!(
                    "expected the Material3 autocomplete overlay scene to be stable after animations settle ({label}, {scale}, {case_name})"
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
                &format!("material3-autocomplete.{scale}.{label}"),
                "material3_headless_autocomplete_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_menu_dialog_style_suite_goldens_v1() {
    use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, Length, MainAlign};
    use fret_ui_material3::menu::Menu;
    use fret_ui_material3::{Button, Dialog};

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
    let style_suite = load_material3_menu_dialog_style_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Menu: default vs override (in the same scene).
            {
                let menu_case = style_suite
                    .case(Material3MenuDialogStyleGoldenCaseKindV1::MenuDefaultVsOverride);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let style = menu_case.menu_style();
                let entries = style_suite.menu_entries();

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
                            let default_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("default menu")
                                .test_id("menu-default")
                                .into_element(cx);

                            let override_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("override menu")
                                .test_id("menu-override")
                                .style(style.clone())
                                .into_element(cx);

                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Horizontal;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(32.0));
                            props.align = CrossAlign::Start;
                            props.justify = MainAlign::Center;

                            let content = cx.flex(props, |cx| {
                                let mut left = ContainerProps::default();
                                left.layout.size.width = Length::Px(Px(360.0));
                                let left = cx.container(left, |_cx| vec![default_menu]);

                                let mut right = ContainerProps::default();
                                right.layout.size.width = Length::Px(Px(360.0));
                                let right = cx.container(right, |_cx| vec![override_menu]);

                                vec![left, right]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 menu style scene to be stable ({label}, {scale}, {})",
                    menu_case.id()
                );
                cases.insert(
                    menu_case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        menu_case.settle_from_frame(),
                        menu_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: default open state (modal overlay).
            {
                let dialog_case =
                    style_suite.case(Material3MenuDialogStyleGoldenCaseKindV1::DialogDefault);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);
                let headline = dialog_case.headline().to_string();
                let supporting_text = dialog_case.supporting_text().to_string();
                let actions = dialog_case.dialog_actions();

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
                            let dialog = Dialog::new(open.clone())
                                .headline(headline.clone())
                                .supporting_text(supporting_text.clone())
                                .actions(actions.clone())
                                .test_id("dialog-default")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog default scene to be stable after animations settle ({label}, {scale}, {})",
                    dialog_case.id()
                );
                cases.insert(
                    dialog_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        dialog_case.settle_from_frame(),
                        dialog_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: override surface + text colors.
            {
                let dialog_case =
                    style_suite.case(Material3MenuDialogStyleGoldenCaseKindV1::DialogOverride);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);
                let headline = dialog_case.headline().to_string();
                let supporting_text = dialog_case.supporting_text().to_string();
                let actions = dialog_case.dialog_actions();
                let style = dialog_case.dialog_style();

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
                            let dialog = Dialog::new(open.clone())
                                .headline(headline.clone())
                                .supporting_text(supporting_text.clone())
                                .actions(actions.clone())
                                .style(style.clone())
                                .test_id("dialog-override")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog override scene to be stable after animations settle ({label}, {scale}, {})",
                    dialog_case.id()
                );
                cases.insert(
                    dialog_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        dialog_case.settle_from_frame(),
                        dialog_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-menu-dialog-style.{scale}.{label}"),
                "material3_headless_menu_dialog_style_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_bottom_sheet_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet,
    };

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked sheet (standard): non-overlay surface.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

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
                            let sheet = DockedBottomSheet::new()
                                .variant(DockedBottomSheetVariant::Standard)
                                .test_id("bottom-sheet-docked")
                                .into_element(cx, |cx| {
                                    vec![
                                        cx.text("Docked bottom sheet"),
                                        Button::new("Primary")
                                            .variant(ButtonVariant::Filled)
                                            .test_id("bottom-sheet-docked-primary")
                                            .into_element(cx),
                                        Button::new("Secondary")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-docked-secondary")
                                            .into_element(cx),
                                    ]
                                });

                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked bottom sheet scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked_standard".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal sheet (open): overlay surface + scrim.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

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
                            let sheet = ModalBottomSheet::new(open_model.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("bottom-sheet-modal")
                                .into_element(
                                    cx,
                                    |cx| {
                                        Button::new("Underlay probe")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-underlay-probe")
                                            .into_element(cx)
                                    },
                                    |cx| {
                                        vec![
                                            cx.text("Modal bottom sheet"),
                                            Button::new("Close")
                                                .variant(ButtonVariant::Filled)
                                                .test_id("bottom-sheet-modal-close")
                                                .into_element(cx),
                                        ]
                                    },
                                );
                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal bottom sheet overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-bottom-sheet.{scale}.{label}"),
                "material3_headless_bottom_sheet_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_date_picker_suite_goldens_v1() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

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

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked picker: non-overlay surface.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

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
                            let picker = DockedDatePicker::new(month.clone(), selected.clone())
                                .variant(DatePickerVariant::Docked)
                                .today(Some(today))
                                .test_id("date-picker-docked")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked date picker scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: overlay + scrim + focus trap.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let open = app.models_mut().insert(true);
                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

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
                            let dialog = DatePickerDialog::new(
                                open.clone(),
                                month.clone(),
                                selected.clone(),
                            )
                            .today(Some(today))
                            .open_duration_ms(Some(1))
                            .close_duration_ms(Some(1))
                            .test_id("date-picker-modal")
                            .into_element(cx, |cx| {
                                Button::new("Underlay probe")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("date-picker-underlay-probe")
                                    .into_element(cx)
                            });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 date picker modal overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-date-picker.{scale}.{label}"),
                "material3_headless_date_picker_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_time_picker_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedTimePicker, TimePickerDialog, TimePickerDisplayMode,
    };
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked picker: non-overlay surface.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let time = app.models_mut().insert(selected_time);

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
                            let picker = DockedTimePicker::new(time.clone())
                                .test_id("time-picker-docked")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked time picker scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Docked picker: input mode.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let time = app.models_mut().insert(selected_time);

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
                            let picker = DockedTimePicker::new(time.clone())
                                .display_mode(TimePickerDisplayMode::Input)
                                .test_id("time-picker-docked-input")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked time picker input scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked_input".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: overlay + scrim + focus trap.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let open = app.models_mut().insert(true);
                let time = app.models_mut().insert(selected_time);

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
                            let dialog = TimePickerDialog::new(open.clone(), time.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("time-picker-modal")
                                .into_element(cx, |cx| {
                                    Button::new("Underlay probe")
                                        .variant(ButtonVariant::Outlined)
                                        .test_id("time-picker-underlay-probe")
                                        .into_element(cx)
                                });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 time picker modal overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: input mode.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let open = app.models_mut().insert(true);
                let time = app.models_mut().insert(selected_time);

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
                            let dialog = TimePickerDialog::new(open.clone(), time.clone())
                                .initial_display_mode(TimePickerDisplayMode::Input)
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("time-picker-modal-input")
                                .into_element(cx, |cx| {
                                    Button::new("Underlay probe")
                                        .variant(ButtonVariant::Outlined)
                                        .test_id("time-picker-underlay-probe")
                                        .into_element(cx)
                                });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 time picker modal overlay input scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open_input".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-time-picker.{scale}.{label}"),
                "material3_headless_time_picker_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_text_field_suite_goldens_v1() {
    use fret_ui::element::FlexProps;

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
    let text_field_suite = load_material3_text_field_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in text_field_suite.cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(640.0), Px(520.0)),
                );

                let field_models = text_field_suite
                    .fields()
                    .iter()
                    .map(|field| {
                        let model = app.models_mut().insert(field.value().to_string());
                        (field, model)
                    })
                    .collect::<Vec<_>>();

                let render = |ui: &mut UiTree<TestHost>,
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
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                            let content = cx.flex(props, |cx| {
                                field_models
                                    .iter()
                                    .map(|(field, model)| {
                                        field.text_field(model.clone()).into_element(cx)
                                    })
                                    .collect::<Vec<_>>()
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                if case.is_idle() {
                    dispatch_idle_pointer(&mut ui, &mut app, &mut services);
                }

                if let Some(test_id) = case.hover_test_id() {
                    hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);
                    dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
                }

                let message = format!(
                    "expected text field scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                let snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.settle_from_frame(),
                    case.total_frames(),
                    &message,
                    &render,
                );
                cases.insert(case.id().to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-text-field.{scale}.{label}"),
                "material3_headless_text_field_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_search_bar_suite_goldens_v1() {
    use fret_icons::ids::ui;
    use fret_ui_material3::SearchBar;

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
    let search_suite = load_material3_search_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in search_suite.search_bar_cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(640.0), Px(240.0)),
                );

                let model = app.models_mut().insert(String::new());
                let model_for_render = model.clone();

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
                            let search_bar = SearchBar::new(model_for_render.clone())
                                .placeholder("Search")
                                .leading_icon(ui::SEARCH)
                                .trailing_icon(ui::CLOSE)
                                .test_id("sb")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), search_bar)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let node_id: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some("sb")).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!("expected sb in semantics snapshot ({label}, {scale}, {case_name})")
                    });
                let node_bounds = ui.debug_node_visual_bounds(node_id).unwrap_or_else(|| {
                    panic!("expected sb bounds ({label}, {scale}, {case_name})")
                });
                let center = Point::new(
                    Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.5),
                    Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                );

                if case_name == "idle" {
                    ui.dispatch_event(
                        &mut app,
                        &mut services,
                        &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                    );
                }

                if case.hover() {
                    ui.dispatch_event(&mut app, &mut services, &pointer_move(PointerId(1), center));
                }

                if case.pressed() {
                    ui.dispatch_event(&mut app, &mut services, &pointer_down(PointerId(1), center));
                }

                if case.focus_visible() {
                    ui.set_focus(Some(node_id));
                    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                }

                let message = format!(
                    "expected the Material3 search bar scene to be stable ({label}, {scale}, {case_name})"
                );
                let snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.settle_from_frame(),
                    case.total_frames(),
                    &message,
                    &render,
                );

                cases.insert(case.id().to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-search-bar.{scale}.{label}"),
                "material3_headless_search_bar_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_search_view_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::SearchView;

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
    let search_suite = load_material3_search_golden_suite_v1();
    let search_view_results = search_suite.search_view_results();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in search_suite.search_view_cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(720.0), Px(520.0)),
                );

                let open_model = app.models_mut().insert(case.open());
                let query = app.models_mut().insert(String::new());
                let presentation = case.presentation();
                let results = search_view_results.clone();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let results = results.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let content = cx.named("search_view_content", |cx| {
                                let mut props = FlexProps::default();
                                props.direction = fret_core::Axis::Vertical;
                                props.gap = fret_ui::element::SpacingLength::Px(Px(8.0));
                                cx.flex(props, |cx| {
                                    results
                                        .iter()
                                        .map(|label| cx.text(label.clone()))
                                        .collect::<Vec<_>>()
                                })
                            });

                            let search_view = SearchView::new(open_model.clone(), query.clone())
                                .placeholder("Search")
                                .a11y_label("Search")
                                .test_id("sv")
                                .presentation(presentation)
                                .into_element(cx, |_cx| vec![content]);

                            let content = cx.named("search_view_root", |cx| {
                                let mut root = FlexProps::default();
                                root.direction = fret_core::Axis::Vertical;
                                root.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                cx.flex(root, |cx| {
                                    vec![
                                        search_view,
                                        cx.text("Underlay probe"),
                                        cx.text("Underlay probe 2"),
                                    ]
                                })
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 search view overlay scene to be stable ({label}, {scale}, {case_name})"
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
                        28,
                        72,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-search-view.{scale}.{label}"),
                "material3_headless_search_view_suite_goldens_v1",
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_carousel_item_suite_goldens_v1() {
    use fret_ui::element::{ContainerProps, FlexProps, Length, TextProps};

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
    let carousel_suite = load_material3_carousel_item_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in carousel_suite.cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(520.0), Px(340.0)),
                );

                let on_activate: fret_ui::action::OnActivate = Arc::new(|_host, _cx, _reason| {});

                let render = |ui: &mut UiTree<TestHost>,
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
                            let theme = Theme::global(&*cx.app).clone();
                            let body_style = theme
                                .text_style_by_key("md.sys.typescale.body-medium")
                                .unwrap_or_default();
                            let body_color = theme.color_token("md.sys.color.on-surface");

                            let item_content =
                                |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                 label: &str| {
                                    let mut container = ContainerProps::default();
                                    container.layout.size.width = Length::Fill;
                                    container.layout.size.height = Length::Fill;
                                    container.padding = Edges::all(Px(16.0)).into();

                                    let mut text = TextProps::new(Arc::<str>::from(label));
                                    text.style = Some(body_style.clone());
                                    text.color = Some(body_color);

                                    cx.container(container, move |cx| vec![cx.text_props(text)])
                                };

                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            props.wrap = false;

                            let content = cx.flex(props, |cx| {
                                let mut elements = Vec::new();
                                for item in carousel_suite.items() {
                                    elements.push(
                                        item.carousel_item(on_activate.clone())
                                            .into_element(cx, |cx| {
                                                vec![item_content(cx, item.label())]
                                            }),
                                    );
                                }
                                elements
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                if case.is_idle() {
                    dispatch_idle_pointer(&mut ui, &mut app, &mut services);
                }

                if let Some(test_id) = case.hover_test_id() {
                    hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);
                    dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
                }

                let message = format!(
                    "expected carousel item scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                let snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.settle_from_frame(),
                    case.total_frames(),
                    &message,
                    &render,
                );
                cases.insert(case.id().to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-carousel-item.{scale}.{label}"),
                "material3_headless_carousel_item_suite_goldens_v1",
                &suite,
            );
        }
    }
}
