use std::{collections::BTreeMap, sync::Arc};

use fret_core::{
    AppWindowId, Edges, KeyCode, NodeId, Point, PointerId, Px, Rect, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{Theme, UiTree};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    events::{key_down, key_up, pointer_move},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_controls_suite_goldens_v1() {
    use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, Length, TextProps};
    use fret_ui_material3::{
        AssistChip, AssistChipVariant, Button, Card, CardVariant, Checkbox, FilterChip,
        FilterChipVariant, InputChip, Select, SelectItem, SuggestionChip, SuggestionChipVariant,
        Switch,
    };

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
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
