use std::{collections::BTreeMap, sync::Arc};

use fret_core::{
    AppWindowId, Edges, KeyCode, NodeId, Point, PointerId, Px, Rect, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::element::AnyElement;
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{key_down, key_up, pointer_down, pointer_move};
use support::goldens::{
    Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
    settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
    snapshot_material3_scene_at_frame_v1, write_or_assert_material3_suite_for_test_v1,
};
use support::headless_search_cases::load_material3_search_golden_suite_v1;
use support::headless_snackbar_cases::load_material3_snackbar_golden_suite_v1;
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

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
    support::headless_golden_runners::navigation::run_material3_headless_navigation_suite_goldens_v1(
    );
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
    support::headless_golden_runners::list::run_material3_headless_list_suite_goldens_v1();
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
    support::headless_golden_runners::slider::run_material3_headless_slider_suite_goldens_v1();
}

#[test]
fn material3_headless_overlays_suite_goldens_v1() {
    support::headless_golden_runners::overlays::run_material3_headless_overlays_suite_goldens_v1();
}

#[test]
fn material3_headless_autocomplete_suite_goldens_v1() {
    support::headless_golden_runners::autocomplete::run_material3_headless_autocomplete_suite_goldens_v1();
}

#[test]
fn material3_headless_menu_dialog_style_suite_goldens_v1() {
    support::headless_golden_runners::menu_dialog_style::run_material3_headless_menu_dialog_style_suite_goldens_v1();
}

#[test]
fn material3_headless_bottom_sheet_suite_goldens_v1() {
    support::headless_golden_runners::bottom_sheet::run_material3_headless_bottom_sheet_suite_goldens_v1();
}

#[test]
fn material3_headless_date_picker_suite_goldens_v1() {
    support::headless_golden_runners::date_picker::run_material3_headless_date_picker_suite_goldens_v1();
}

#[test]
fn material3_headless_time_picker_suite_goldens_v1() {
    support::headless_golden_runners::time_picker::run_material3_headless_time_picker_suite_goldens_v1();
}

#[test]
fn material3_headless_text_field_suite_goldens_v1() {
    support::headless_golden_runners::text_field::run_material3_headless_text_field_suite_goldens_v1();
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
    support::headless_golden_runners::carousel_item::run_material3_headless_carousel_item_suite_goldens_v1();
}
