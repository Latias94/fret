#![cfg(feature = "diagnostics")]

//! Stable automation-surface tests for Material 3 recipes.

use std::{rc::Rc, sync::Arc};

use fret_core::{
    AppWindowId, Point, PointerId, Px, Rect, SemanticsLive, SemanticsRole, Size, UiServices,
};
use fret_runtime::{
    ModelHost, PlatformCapabilities,
    fret_i18n::{
        I18nLookup, I18nLookupError, I18nService, LocaleId, LocalizedMessage, MessageArgValue,
        MessageArgs, MessageKey,
    },
};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_move, pointer_up};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn live_test_id_exists(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> bool {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .any(|m| ui.debug_node_visual_bounds(m.node).is_some())
}

fn live_test_id_anchor_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live bounds for test_id {id}"))
}

fn live_test_id_layout_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_bounds(m.node)
                .or_else(|| ui.debug_node_visual_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live layout bounds for test_id {id}"))
}

fn rect_center_x(bounds: Rect) -> f32 {
    bounds.origin.x.0 + bounds.size.width.0 * 0.5
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn click_semantics_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    test_id: &str,
    pointer_id: PointerId,
) {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    let bounds = ui
        .debug_node_visual_bounds(node)
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"));
    let click_at = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(app, services, &pointer_down(pointer_id, click_at));
    ui.dispatch_event(app, services, &pointer_up(pointer_id, click_at));
}

fn semantics_node_disabled(ui: &UiTree<TestHost>, test_id: &str) -> bool {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
        .flags
        .disabled
}

fn semantics_node_label(ui: &UiTree<TestHost>, test_id: &str) -> String {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
        .label
        .clone()
        .unwrap_or_else(|| panic!("expected semantics label for test_id {test_id}"))
}

fn semantics_node_value(ui: &UiTree<TestHost>, test_id: &str) -> Option<String> {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
        .value
        .clone()
}

fn semantics_node_role(ui: &UiTree<TestHost>, test_id: &str) -> SemanticsRole {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
        .role
}

fn semantics_node_selected(ui: &UiTree<TestHost>, test_id: &str) -> bool {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
        .flags
        .selected
}

fn message_arg_u64(args: Option<&MessageArgs>, name: &str) -> u64 {
    args.and_then(|args| {
        args.iter().find_map(|(key, value)| {
            (key == name).then_some(match value {
                MessageArgValue::Unsigned(value) => *value,
                MessageArgValue::Integer(value) => (*value).try_into().unwrap_or_default(),
                MessageArgValue::Number(value) => *value as u64,
                MessageArgValue::String(_) | MessageArgValue::Bool(_) => 0,
            })
        })
    })
    .unwrap_or_default()
}

fn message_arg_display(args: Option<&MessageArgs>, name: &str) -> String {
    args.and_then(|args| {
        args.iter().find_map(|(key, value)| {
            (key == name).then(|| match value {
                MessageArgValue::String(value) => value.clone(),
                MessageArgValue::Number(value) => value.to_string(),
                MessageArgValue::Integer(value) => value.to_string(),
                MessageArgValue::Unsigned(value) => value.to_string(),
                MessageArgValue::Bool(value) => value.to_string(),
            })
        })
    })
    .unwrap_or_default()
}

struct TimePickerStringLookup;

impl I18nLookup for TimePickerStringLookup {
    fn format(
        &self,
        preferred_locales: &[LocaleId],
        key: &MessageKey,
        args: Option<&MessageArgs>,
    ) -> Result<LocalizedMessage, I18nLookupError> {
        let locale = preferred_locales
            .first()
            .cloned()
            .unwrap_or_else(LocaleId::en_us);
        let text = match key.as_str() {
            "material3-time-picker-title" => "Localized select time".to_string(),
            "material3-time-picker-dismiss" => "Localized dismiss".to_string(),
            "material3-time-picker-cancel" => "Localized cancel".to_string(),
            "material3-time-picker-confirm" => "Localized ok".to_string(),
            "material3-time-picker-toggle-input" => "Localized text input mode".to_string(),
            "material3-time-picker-toggle-dial" => "Localized clock mode".to_string(),
            "material3-time-picker-hour-selection" => "Localized hour selector".to_string(),
            "material3-time-picker-minute-selection" => "Localized minute selector".to_string(),
            "material3-time-picker-hour-text-field" => "Localized hour input".to_string(),
            "material3-time-picker-minute-text-field" => "Localized minute input".to_string(),
            "material3-time-picker-period-toggle" => "Localized period toggle".to_string(),
            "material3-time-picker-period-am" => "Localized AM".to_string(),
            "material3-time-picker-period-pm" => "Localized PM".to_string(),
            "material3-time-picker-hour" => "Localized hour supporting text".to_string(),
            "material3-time-picker-minute" => "Localized minute supporting text".to_string(),
            "material3-time-picker-hour-error-12h" => "Localized 12h hour error".to_string(),
            "material3-time-picker-hour-error-24h" => "Localized 24h hour error".to_string(),
            "material3-time-picker-minute-error" => "Localized minute error".to_string(),
            "material3-time-picker-hour-value-12h" => {
                format!("Localized 12h hour {}", message_arg_u64(args, "hour"))
            }
            "material3-time-picker-hour-value-24h" => {
                format!("Localized 24h hour {}", message_arg_u64(args, "hour"))
            }
            "material3-time-picker-minute-value" => {
                format!("Localized minute {}", message_arg_u64(args, "minute"))
            }
            _ => return Err(I18nLookupError::MissingKey { key: key.clone() }),
        };

        Ok(LocalizedMessage {
            text,
            locale,
            fallback_depth: 0,
        })
    }
}

fn install_time_picker_string_lookup(app: &mut TestHost) {
    app.set_global(I18nService::default().with_lookup(Rc::new(TimePickerStringLookup)));
}

struct DatePickerStringLookup;

impl I18nLookup for DatePickerStringLookup {
    fn format(
        &self,
        preferred_locales: &[LocaleId],
        key: &MessageKey,
        args: Option<&MessageArgs>,
    ) -> Result<LocalizedMessage, I18nLookupError> {
        let locale = preferred_locales
            .first()
            .cloned()
            .unwrap_or_else(LocaleId::en_us);
        let text = match key.as_str() {
            "material3-date-picker-title" => "Localized select date".to_string(),
            "material3-date-picker-dismiss" => "Localized dismiss date picker".to_string(),
            "material3-date-picker-cancel" => "Localized cancel date".to_string(),
            "material3-date-picker-confirm" => "Localized confirm date".to_string(),
            "material3-date-picker-previous-month" => "Localized previous month".to_string(),
            "material3-date-picker-next-month" => "Localized next month".to_string(),
            "material3-date-picker-previous-month-short" => "Prev*".to_string(),
            "material3-date-picker-next-month-short" => "Next*".to_string(),
            "material3-date-picker-month-year" => format!(
                "Localized {} {}",
                message_arg_display(args, "month"),
                message_arg_display(args, "year")
            ),
            "material3-date-picker-day-description" => format!(
                "Localized date {}-{:02}-{:02}",
                message_arg_display(args, "year"),
                message_arg_u64(args, "month_number"),
                message_arg_u64(args, "day")
            ),
            "material3-date-picker-today-description" => "Localized today".to_string(),
            "material3-date-picker-today-date-description" => format!(
                "{} / {}",
                message_arg_display(args, "today"),
                message_arg_display(args, "date")
            ),
            "material3-date-picker-month-january" => "M01".to_string(),
            "material3-date-picker-month-february" => "M02".to_string(),
            "material3-date-picker-month-march" => "M03".to_string(),
            "material3-date-picker-month-april" => "M04".to_string(),
            "material3-date-picker-month-may" => "M05".to_string(),
            "material3-date-picker-month-june" => "M06".to_string(),
            "material3-date-picker-month-july" => "M07".to_string(),
            "material3-date-picker-month-august" => "M08".to_string(),
            "material3-date-picker-month-september" => "M09".to_string(),
            "material3-date-picker-month-october" => "M10".to_string(),
            "material3-date-picker-month-november" => "M11".to_string(),
            "material3-date-picker-month-december" => "M12".to_string(),
            "material3-date-picker-weekday-short-monday" => "Mo*".to_string(),
            "material3-date-picker-weekday-short-tuesday" => "Tu*".to_string(),
            "material3-date-picker-weekday-short-wednesday" => "We*".to_string(),
            "material3-date-picker-weekday-short-thursday" => "Th*".to_string(),
            "material3-date-picker-weekday-short-friday" => "Fr*".to_string(),
            "material3-date-picker-weekday-short-saturday" => "Sa*".to_string(),
            "material3-date-picker-weekday-short-sunday" => "Su*".to_string(),
            "material3-date-picker-weekday-long-monday" => "Localized Monday".to_string(),
            "material3-date-picker-weekday-long-tuesday" => "Localized Tuesday".to_string(),
            "material3-date-picker-weekday-long-wednesday" => "Localized Wednesday".to_string(),
            "material3-date-picker-weekday-long-thursday" => "Localized Thursday".to_string(),
            "material3-date-picker-weekday-long-friday" => "Localized Friday".to_string(),
            "material3-date-picker-weekday-long-saturday" => "Localized Saturday".to_string(),
            "material3-date-picker-weekday-long-sunday" => "Localized Sunday".to_string(),
            _ => return Err(I18nLookupError::MissingKey { key: key.clone() }),
        };

        Ok(LocalizedMessage {
            text,
            locale,
            fallback_depth: 0,
        })
    }
}

fn install_date_picker_string_lookup(app: &mut TestHost) {
    app.set_global(I18nService::default().with_lookup(Rc::new(DatePickerStringLookup)));
}

fn semantics_node_live(ui: &UiTree<TestHost>, test_id: &str) -> Option<(SemanticsLive, bool)> {
    let node = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    node.flags.live.map(|live| (live, node.flags.live_atomic))
}

#[test]
fn material3_select_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{Select, SelectItem, SelectVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha")
            .leading_icon(ids::ui::CHECK)
            .trailing_icon(ids::ui::CLOSE)
            .test_id("m3-select-item-alpha"),
        SelectItem::new("beta", "Beta").test_id("m3-select-item-beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            let selected_model = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let select = Select::new(selected_model)
                    .variant(SelectVariant::Filled)
                    .a11y_label("Material select")
                    .label("Choice")
                    .placeholder("Pick one")
                    .items(items)
                    .test_id("m3-select")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), select)]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for id in [
        "m3-select",
        "m3-select.chrome",
        "m3-select.active-indicator",
        "m3-select.label",
        "m3-select.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Select part test_id {id}"
        );
    }

    let trigger =
        semantics_node_id_by_test_id(&ui, "m3-select").expect("expected m3-select semantics node");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger)
        .expect("expected select trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    for _ in 0..8 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        if live_test_id_exists(&ui, &app, window, "m3-select.listbox") {
            break;
        }
    }

    for id in [
        "m3-select.listbox",
        "m3-select-item-alpha",
        "m3-select-item-alpha.chrome",
        "m3-select-item-alpha.leading-icon",
        "m3-select-item-alpha.trailing-icon",
        "m3-select-item-beta",
        "m3-select-item-beta.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Select popup/item part test_id {id}"
        );
    }

    let listbox_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-select.listbox");
    let selected_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-select-item-beta.chrome");
    let epsilon = 0.5;
    assert!(
        (selected_bounds.origin.x.0 - (listbox_bounds.origin.x.0 + 4.0)).abs() <= epsilon,
        "expected selected Select item chrome to use Material selectable-item horizontal inset; listbox_x={}, selected_x={}",
        listbox_bounds.origin.x.0,
        selected_bounds.origin.x.0
    );
    assert!(
        (selected_bounds.size.width.0 - (listbox_bounds.size.width.0 - 8.0)).abs() <= epsilon,
        "expected selected Select item chrome width to account for both horizontal insets; listbox_width={}, selected_width={}",
        listbox_bounds.size.width.0,
        selected_bounds.size.width.0
    );
}

#[test]
fn material3_switch_exposes_stable_part_test_ids() {
    use fret_ui_material3::Switch;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let selected = app.models_mut().insert(false);
    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let switch = Switch::new(selected_model)
                    .icons(true)
                    .a11y_label("Material switch")
                    .test_id("m3-switch")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), switch)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-switch",
        "m3-switch.chrome",
        "m3-switch.track",
        "m3-switch.handle",
        "m3-switch.icon-on",
        "m3-switch.icon-off",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Switch part test_id {id}"
        );
    }
}

#[test]
fn material3_choice_controls_expose_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{
        Checkbox, IconButton, IconToggleButton, RadioGroup, RadioGroupItem, RadioGroupOrientation,
        RangeSlider, Slider,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(420.0)),
    );

    let checkbox_checked = app.models_mut().insert(true);
    let radio_value = app.models_mut().insert(Some(Arc::<str>::from("alpha")));
    let icon_toggle_checked = app.models_mut().insert(false);
    let slider_value = app.models_mut().insert(0.4_f32);
    let range_values = app.models_mut().insert([0.2_f32, 0.8_f32]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let checkbox_checked = checkbox_checked.clone();
            let radio_value = radio_value.clone();
            let icon_toggle_checked = icon_toggle_checked.clone();
            let slider_value = slider_value.clone();
            let range_values = range_values.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = fret_ui::element::FlexProps::default();
                props.direction = fret_core::Axis::Vertical;
                props.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                props.layout.size.width = fret_ui::element::Length::Px(Px(560.0));
                let controls = cx.flex(props, |cx| {
                    vec![
                        Checkbox::new(checkbox_checked)
                            .a11y_label("Material checkbox")
                            .test_id("m3-checkbox")
                            .into_element(cx),
                        RadioGroup::new(radio_value)
                            .orientation(RadioGroupOrientation::Horizontal)
                            .gap(Px(8.0))
                            .a11y_label("Material radio group")
                            .test_id("m3-radio-group")
                            .items(vec![
                                RadioGroupItem::new("alpha")
                                    .a11y_label("Alpha")
                                    .test_id("m3-radio-alpha"),
                                RadioGroupItem::new("beta")
                                    .a11y_label("Beta")
                                    .test_id("m3-radio-beta"),
                            ])
                            .into_element(cx),
                        IconButton::new(ids::ui::SEARCH)
                            .a11y_label("Material icon button")
                            .test_id("m3-icon-button")
                            .into_element(cx),
                        IconToggleButton::new(icon_toggle_checked, ids::ui::CHECK)
                            .a11y_label("Material icon toggle")
                            .test_id("m3-icon-toggle")
                            .into_element(cx),
                        Slider::new(slider_value)
                            .range(0.0, 1.0)
                            .a11y_label("Material slider")
                            .test_id("m3-slider")
                            .into_element(cx),
                        RangeSlider::new(range_values)
                            .range(0.0, 1.0)
                            .a11y_label("Material range slider")
                            .test_id("m3-range-slider")
                            .into_element(cx),
                    ]
                });
                vec![with_padding(cx, Px(32.0), controls)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-checkbox",
        "m3-checkbox.chrome",
        "m3-radio-group",
        "m3-radio-alpha",
        "m3-radio-alpha.chrome",
        "m3-radio-beta",
        "m3-radio-beta.chrome",
        "m3-icon-button",
        "m3-icon-button.chrome",
        "m3-icon-toggle",
        "m3-icon-toggle.chrome",
        "m3-slider",
        "m3-slider.track",
        "m3-slider.active-track",
        "m3-slider.handle",
        "m3-range-slider",
        "m3-range-slider.start",
        "m3-range-slider.start.handle",
        "m3-range-slider.end",
        "m3-range-slider.end.handle",
        "m3-range-slider.track",
        "m3-range-slider.active-track",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live choice-control part test_id {id}"
        );
    }
}

#[test]
fn material3_segmented_buttons_and_chips_expose_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{
        AssistChip, ChipSet, ChipSetItem, FilterChip, InputChip, SegmentedButtonItem,
        SegmentedButtonSet, SuggestionChip,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(360.0)),
    );

    let segmented_value = app.models_mut().insert(Arc::<str>::from("alpha"));
    let filter_selected = app.models_mut().insert(true);
    let input_selected = app.models_mut().insert(false);

    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let segmented_value = segmented_value.clone();
        let filter_selected = filter_selected.clone();
        let input_selected = input_selected.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let mut props = fret_ui::element::FlexProps::default();
            props.direction = fret_core::Axis::Vertical;
            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
            props.layout.size.width = fret_ui::element::Length::Px(Px(600.0));
            let controls = cx.flex(props, |cx| {
                let no_op =
                    Arc::new(|_host: &mut dyn fret_ui::action::UiActionHost, _cx, _reason| {});
                vec![
                    SegmentedButtonSet::single(segmented_value)
                        .a11y_label("Material segmented buttons")
                        .test_id("m3-segmented")
                        .items(vec![
                            SegmentedButtonItem::new("alpha", "Alpha")
                                .test_id("m3-segmented-alpha"),
                            SegmentedButtonItem::new("beta", "Beta").test_id("m3-segmented-beta"),
                        ])
                        .into_element(cx),
                    ChipSet::new(vec![
                        ChipSetItem::from(
                            AssistChip::new("Assist")
                                .leading_icon(ids::ui::SEARCH)
                                .test_id("m3-assist-chip"),
                        ),
                        ChipSetItem::from(
                            SuggestionChip::new("Suggest")
                                .leading_icon(ids::ui::SEARCH)
                                .test_id("m3-suggestion-chip"),
                        ),
                        ChipSetItem::from(
                            FilterChip::new(filter_selected, "Filter")
                                .trailing_icon(ids::ui::CLOSE)
                                .on_trailing_icon_activate(no_op.clone())
                                .test_id("m3-filter-chip"),
                        ),
                        ChipSetItem::from(
                            InputChip::new(input_selected, "Input")
                                .leading_icon(ids::ui::SEARCH)
                                .trailing_icon(ids::ui::CLOSE)
                                .on_trailing_icon_activate(no_op.clone())
                                .test_id("m3-input-chip"),
                        ),
                    ])
                    .a11y_label("Material chip set")
                    .test_id("m3-chip-set")
                    .into_element(cx),
                ]
            });
            vec![with_padding(cx, Px(32.0), controls)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-segmented",
        "m3-segmented-alpha",
        "m3-segmented-alpha.chrome",
        "m3-segmented-beta",
        "m3-segmented-beta.chrome",
        "m3-chip-set",
        "m3-assist-chip",
        "m3-assist-chip.chrome",
        "m3-suggestion-chip",
        "m3-suggestion-chip.chrome",
        "m3-filter-chip",
        "m3-filter-chip.chrome",
        "m3-filter-chip.trailing-icon",
        "m3-input-chip",
        "m3-input-chip.chrome",
        "m3-input-chip.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live segmented/chip part test_id {id}"
        );
    }
}

#[test]
fn material3_surface_data_display_expose_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui::element::{ContainerProps, FlexProps, Length, SpacerProps};
    use fret_ui_material3::{
        Badge, BadgePlacement, Button, ButtonVariant, Card, CardVariant, CarouselItem,
        CarouselItemVariant, CircularProgressIndicator, Divider, Fab, FabVariant,
        LinearProgressIndicator, List, ListItem, TopAppBar, TopAppBarAction, TopAppBarVariant,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(760.0), Px(760.0)),
    );

    let list_selected = app.models_mut().insert(Arc::<str>::from("beta"));
    let progress = app.models_mut().insert(0.4_f32);
    let no_op: fret_ui::action::OnActivate = Arc::new(|_host, _cx, _reason| {});

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let list_selected = list_selected.clone();
            let progress = progress.clone();
            let no_op = no_op.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut column = FlexProps::default();
                column.direction = fret_core::Axis::Vertical;
                column.gap = fret_ui::element::SpacingLength::Px(Px(14.0));
                column.layout.size.width = Length::Px(Px(620.0));

                let content = cx.flex(column, |cx| {
                    let anchor = |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                        let mut props = ContainerProps::default();
                        props.layout.size.width = Length::Px(Px(40.0));
                        props.layout.size.height = Length::Px(Px(40.0));
                        cx.container(props, |_cx| Vec::new())
                    };

                    let mut row = FlexProps::default();
                    row.direction = fret_core::Axis::Horizontal;
                    row.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                    row.align = fret_ui::element::CrossAlign::Center;

                    vec![
                        cx.flex(row, |cx| {
                            vec![
                                Badge::text("7")
                                    .placement(BadgePlacement::TopRight)
                                    .anchor_size(Px(40.0))
                                    .a11y_label("Material badge")
                                    .test_id("m3-badge")
                                    .into_element(cx, |cx| vec![anchor(cx)]),
                                Button::new("Button")
                                    .variant(ButtonVariant::Filled)
                                    .test_id("m3-button")
                                    .into_element(cx),
                                Fab::new(ids::ui::PLUS)
                                    .variant(FabVariant::Primary)
                                    .a11y_label("Material fab")
                                    .test_id("m3-fab")
                                    .into_element(cx),
                            ]
                        }),
                        Card::new()
                            .variant(CardVariant::Outlined)
                            .on_activate(no_op.clone())
                            .a11y_label("Material card")
                            .test_id("m3-card")
                            .into_element(cx, |cx| vec![cx.text("Card content")]),
                        CarouselItem::new()
                            .variant(CarouselItemVariant::WithOutline)
                            .width(Px(420.0))
                            .height(Px(72.0))
                            .on_activate(no_op.clone())
                            .a11y_label("Material carousel item")
                            .test_id("m3-carousel")
                            .into_element(cx, |cx| vec![cx.text("Carousel item")]),
                        Divider::horizontal().test_id("m3-divider").into_element(cx),
                        List::new(list_selected)
                            .test_id("m3-list")
                            .items(vec![
                                ListItem::new("alpha", "Alpha")
                                    .leading_icon(ids::ui::SEARCH)
                                    .test_id("m3-list-alpha"),
                                ListItem::new("beta", "Beta")
                                    .trailing_icon(ids::ui::CHEVRON_RIGHT)
                                    .test_id("m3-list-beta"),
                            ])
                            .into_element(cx),
                        LinearProgressIndicator::new(progress.clone())
                            .test_id("m3-linear-progress")
                            .into_element(cx),
                        cx.flex(row, |cx| {
                            vec![
                                CircularProgressIndicator::new(progress)
                                    .test_id("m3-circular-progress")
                                    .into_element(cx),
                                cx.spacer(SpacerProps::default()),
                            ]
                        }),
                        TopAppBar::new("Top App Bar")
                            .variant(TopAppBarVariant::Small)
                            .navigation_icon(
                                TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                                    .a11y_label("Navigate")
                                    .test_id("m3-top-app-bar-nav"),
                            )
                            .actions(vec![
                                TopAppBarAction::new(ids::ui::SEARCH)
                                    .a11y_label("Search")
                                    .test_id("m3-top-app-bar-search"),
                            ])
                            .test_id("m3-top-app-bar")
                            .into_element(cx),
                    ]
                });

                vec![with_padding(cx, Px(32.0), content)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-badge",
        "m3-badge.anchor",
        "m3-badge.badge",
        "m3-button",
        "m3-button.chrome",
        "m3-card",
        "m3-card.chrome",
        "m3-carousel",
        "m3-carousel.chrome",
        "m3-divider",
        "m3-fab",
        "m3-fab.chrome",
        "m3-list",
        "m3-list-alpha",
        "m3-list-alpha.chrome",
        "m3-list-alpha.headline",
        "m3-list-alpha.leading-icon",
        "m3-list-beta",
        "m3-list-beta.chrome",
        "m3-list-beta.headline",
        "m3-list-beta.trailing-icon",
        "m3-linear-progress",
        "m3-linear-progress.track",
        "m3-linear-progress.active-track",
        "m3-circular-progress",
        "m3-circular-progress.track",
        "m3-circular-progress.active-track",
        "m3-top-app-bar",
        "m3-top-app-bar-nav",
        "m3-top-app-bar-nav.chrome",
        "m3-top-app-bar-search",
        "m3-top-app-bar-search.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live surface/data-display part test_id {id}"
        );
    }

    assert_eq!(semantics_node_role(&ui, "m3-badge"), SemanticsRole::Group);
    assert_eq!(
        semantics_node_label(&ui, "m3-badge.badge"),
        "Material badge"
    );
    assert_eq!(
        semantics_node_value(&ui, "m3-badge.badge").as_deref(),
        Some("7")
    );
    assert_eq!(semantics_node_role(&ui, "m3-list"), SemanticsRole::List);
    assert_eq!(
        semantics_node_role(&ui, "m3-list-alpha"),
        SemanticsRole::ListItem
    );
    assert!(!semantics_node_selected(&ui, "m3-list-alpha"));
    assert!(semantics_node_selected(&ui, "m3-list-beta"));
    assert_eq!(
        semantics_node_role(&ui, "m3-linear-progress"),
        SemanticsRole::ProgressBar
    );
    assert_eq!(
        semantics_node_role(&ui, "m3-circular-progress"),
        SemanticsRole::ProgressBar
    );
}

#[test]
fn material3_tabs_exposes_stable_part_test_ids() {
    use fret_ui_material3::{TabItem, Tabs};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("overview"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let tabs = Tabs::new(selected.clone())
                    .a11y_label("Material tabs")
                    .test_id("m3-tabs")
                    .items(vec![
                        TabItem::new("overview", "Overview").test_id("m3-tab-overview"),
                        TabItem::new("settings", "Settings").test_id("m3-tab-settings"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), tabs)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-tabs",
        "m3-tabs.chrome",
        "m3-tabs.active-indicator",
        "m3-tab-overview",
        "m3-tab-overview.chrome",
        "m3-tab-settings",
        "m3-tab-settings.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Tabs part test_id {id}"
        );
    }
}

#[test]
fn material3_navigation_bar_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("search"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(selected.clone())
                    .a11y_label("Material navigation bar")
                    .test_id("m3-navigation-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .badge_dot()
                            .test_id("m3-nav-search"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .test_id("m3-nav-settings"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-navigation-bar",
        "m3-navigation-bar.chrome",
        "m3-navigation-bar.active-indicator",
        "m3-nav-search",
        "m3-nav-search.chrome",
        "m3-nav-search.icon",
        "m3-nav-search.label",
        "m3-nav-search.badge",
        "m3-nav-settings",
        "m3-nav-settings.chrome",
        "m3-nav-settings.icon",
        "m3-nav-settings.label",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live NavigationBar part test_id {id}"
        );
    }
}

#[test]
fn material3_navigation_rail_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(520.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("play"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(selected.clone())
                    .a11y_label("Material navigation rail")
                    .test_id("m3-navigation-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("m3-rail-search"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .badge_text("99+")
                            .test_id("m3-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-navigation-rail",
        "m3-navigation-rail.chrome",
        "m3-navigation-rail.active-indicator",
        "m3-rail-search",
        "m3-rail-search.chrome",
        "m3-rail-search.icon",
        "m3-rail-search.label",
        "m3-rail-play",
        "m3-rail-play.chrome",
        "m3-rail-play.icon",
        "m3-rail-play.label",
        "m3-rail-play.badge",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live NavigationRail part test_id {id}"
        );
    }
}

#[test]
fn material3_text_field_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{TextField, TextFieldVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let value = app.models_mut().insert(String::new());
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let field = TextField::new(value.clone())
                    .variant(TextFieldVariant::Filled)
                    .label("Email")
                    .placeholder("name@example.com")
                    .supporting_text("Required")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .test_id("m3-text-field")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), field)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-text-field",
        "m3-text-field.chrome",
        "m3-text-field.active-indicator",
        "m3-text-field.label",
        "m3-text-field.supporting-text",
        "m3-text-field.leading-icon",
        "m3-text-field.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live TextField part test_id {id}"
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("m3-text-field"))
        .expect("expected TextField input semantics node");
    let label = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("m3-text-field.label"))
        .expect("expected TextField label semantics node");
    let supporting = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("m3-text-field.supporting-text"))
        .expect("expected TextField supporting text semantics node");
    assert!(
        input.labelled_by.contains(&label.id),
        "expected TextField input to be labelled by the visual label"
    );
    assert!(
        input.described_by.contains(&supporting.id),
        "expected TextField input to be described by supporting text"
    );
}

#[test]
fn material3_autocomplete_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(360.0)),
    );

    let query = app.models_mut().insert(String::new());
    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha").test_id("m3-autocomplete-alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            let selected_model = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let autocomplete = Autocomplete::new(query.clone())
                    .selected_value(selected_model)
                    .a11y_label("Material autocomplete")
                    .label("Search")
                    .placeholder("Type")
                    .supporting_text("Pick one")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_dropdown_icon(true)
                    .items(items)
                    .test_id("m3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), autocomplete)]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for id in [
        "m3-autocomplete",
        "m3-autocomplete.chrome",
        "m3-autocomplete.label",
        "m3-autocomplete.supporting-text",
        "m3-autocomplete.leading-icon",
        "m3-autocomplete.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Autocomplete field part test_id {id}"
        );
    }

    let trigger = semantics_node_id_by_test_id(&ui, "m3-autocomplete")
        .expect("expected m3-autocomplete semantics node");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger)
        .expect("expected autocomplete bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    for _ in 0..8 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        if live_test_id_exists(&ui, &app, window, "m3-autocomplete.listbox") {
            break;
        }
    }

    for id in [
        "m3-autocomplete.listbox",
        "m3-autocomplete-alpha",
        "m3-autocomplete-alpha.chrome",
        "m3-autocomplete.option.beta",
        "m3-autocomplete.option.beta.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Autocomplete popup/item part test_id {id}"
        );
    }

    let chrome_bounds = live_test_id_anchor_bounds(&ui, &app, window, "m3-autocomplete.chrome");
    let listbox_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-autocomplete.listbox");
    let epsilon = 1.0;
    assert!(
        (listbox_bounds.origin.x.0 - chrome_bounds.origin.x.0).abs() <= epsilon,
        "expected Autocomplete popup to align with field chrome x (got {}, want {})",
        listbox_bounds.origin.x.0,
        chrome_bounds.origin.x.0
    );
    assert!(
        (listbox_bounds.size.width.0 - chrome_bounds.size.width.0).abs() <= epsilon,
        "expected Autocomplete popup width to match field chrome width (got {}, want {})",
        listbox_bounds.size.width.0,
        chrome_bounds.size.width.0
    );

    let selected_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-autocomplete.option.beta.chrome");
    assert!(
        (selected_bounds.origin.x.0 - (listbox_bounds.origin.x.0 + 4.0)).abs() <= epsilon,
        "expected Autocomplete selected item chrome to use Material selectable-item horizontal inset; listbox_x={}, selected_x={}",
        listbox_bounds.origin.x.0,
        selected_bounds.origin.x.0
    );
    assert!(
        (selected_bounds.size.width.0 - (listbox_bounds.size.width.0 - 8.0)).abs() <= epsilon,
        "expected Autocomplete selected item chrome width to account for both horizontal insets; listbox_width={}, selected_width={}",
        listbox_bounds.size.width.0,
        selected_bounds.size.width.0
    );
}

#[test]
fn material3_exposed_dropdown_popup_matches_field_chrome_bounds() {
    use fret_icons::ids;
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(360.0)),
    );

    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha").test_id("m3-exposed-dropdown-alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query.clone())
                    .a11y_label("Material exposed dropdown")
                    .label("Assignee")
                    .placeholder("Choose")
                    .supporting_text("Pick one")
                    .leading_icon(ids::ui::SEARCH)
                    .items(items)
                    .test_id("m3-exposed-dropdown")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), exposed)]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for id in [
        "m3-exposed-dropdown",
        "m3-exposed-dropdown.chrome",
        "m3-exposed-dropdown.label",
        "m3-exposed-dropdown.supporting-text",
        "m3-exposed-dropdown.leading-icon",
        "m3-exposed-dropdown.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live ExposedDropdown field part test_id {id}"
        );
    }

    click_semantics_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "m3-exposed-dropdown.trailing-icon",
        PointerId(1),
    );

    for _ in 0..8 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        if live_test_id_exists(&ui, &app, window, "m3-exposed-dropdown.listbox") {
            break;
        }
    }

    for id in [
        "m3-exposed-dropdown.listbox",
        "m3-exposed-dropdown-alpha",
        "m3-exposed-dropdown-alpha.chrome",
        "m3-exposed-dropdown.option.beta",
        "m3-exposed-dropdown.option.beta.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live ExposedDropdown popup/item part test_id {id}"
        );
    }

    let chrome_bounds = live_test_id_anchor_bounds(&ui, &app, window, "m3-exposed-dropdown.chrome");
    let listbox_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-exposed-dropdown.listbox");
    let epsilon = 1.0;
    assert!(
        (listbox_bounds.origin.x.0 - chrome_bounds.origin.x.0).abs() <= epsilon,
        "expected ExposedDropdown popup to align with field chrome x (got {}, want {})",
        listbox_bounds.origin.x.0,
        chrome_bounds.origin.x.0
    );
    assert!(
        (listbox_bounds.size.width.0 - chrome_bounds.size.width.0).abs() <= epsilon,
        "expected ExposedDropdown popup width to match field chrome width (got {}, want {})",
        listbox_bounds.size.width.0,
        chrome_bounds.size.width.0
    );

    let selected_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-exposed-dropdown.option.beta.chrome");
    assert!(
        (selected_bounds.origin.x.0 - (listbox_bounds.origin.x.0 + 4.0)).abs() <= epsilon,
        "expected ExposedDropdown option chrome to use Material selectable-item horizontal inset; listbox_x={}, option_x={}",
        listbox_bounds.origin.x.0,
        selected_bounds.origin.x.0
    );
    assert!(
        (selected_bounds.size.width.0 - (listbox_bounds.size.width.0 - 8.0)).abs() <= epsilon,
        "expected ExposedDropdown option chrome width to account for both horizontal insets; listbox_width={}, option_width={}",
        listbox_bounds.size.width.0,
        selected_bounds.size.width.0
    );
}

#[test]
fn material3_autocomplete_filled_exposes_active_indicator_part_test_id() {
    use fret_icons::ids;
    use fret_ui_material3::{Autocomplete, AutocompleteItem, AutocompleteVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(220.0)),
    );

    let query = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let autocomplete = Autocomplete::new(query.clone())
                    .variant(AutocompleteVariant::Filled)
                    .a11y_label("Material autocomplete")
                    .label("Search")
                    .placeholder("Type")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_dropdown_icon(true)
                    .items(items)
                    .test_id("m3-autocomplete-filled")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), autocomplete)]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for id in [
        "m3-autocomplete-filled",
        "m3-autocomplete-filled.chrome",
        "m3-autocomplete-filled.active-indicator",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live filled Autocomplete part test_id {id}"
        );
    }
}

#[test]
fn material3_navigation_drawer_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(360.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("search"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(selected.clone())
                    .a11y_label("Material navigation drawer")
                    .test_id("m3-navigation-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("m3-drawer-search"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .test_id("m3-drawer-settings"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-navigation-drawer",
        "m3-navigation-drawer.chrome",
        "m3-drawer-search",
        "m3-drawer-search.chrome",
        "m3-drawer-search.icon",
        "m3-drawer-search.label",
        "m3-drawer-settings",
        "m3-drawer-settings.chrome",
        "m3-drawer-settings.icon",
        "m3-drawer-settings.label",
        "m3-drawer-settings.badge",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live NavigationDrawer part test_id {id}"
        );
    }
}

#[test]
fn material3_modal_navigation_drawer_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{
        Button, ButtonVariant, ModalNavigationDrawer, NavigationDrawer, NavigationDrawerItem,
        NavigationDrawerVariant,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(420.0)),
    );

    let open = app.models_mut().insert(true);
    let selected = app.models_mut().insert(Arc::<str>::from("search"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open.clone();
            let selected = selected.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = ModalNavigationDrawer::new(open.clone())
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-modal-navigation-drawer")
                    .into_element(
                        cx,
                        |cx| {
                            NavigationDrawer::new(selected.clone())
                                .variant(NavigationDrawerVariant::Modal)
                                .a11y_label("Material modal navigation drawer")
                                .test_id("m3-modal-navigation-drawer-content")
                                .items(vec![
                                    NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                                        .test_id("m3-modal-drawer-search"),
                                    NavigationDrawerItem::new(
                                        "settings",
                                        "Settings",
                                        ids::ui::SETTINGS,
                                    )
                                    .badge_label("4")
                                    .test_id("m3-modal-drawer-settings"),
                                ])
                                .into_element(cx)
                        },
                        |cx| {
                            let underlay = Button::new("Underlay")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-modal-navigation-drawer-underlay")
                                .into_element(cx);
                            with_padding(cx, Px(32.0), underlay)
                        },
                    );
                vec![drawer]
            })
        };

    for _ in 0..8 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        if live_test_id_exists(&ui, &app, window, "m3-modal-navigation-drawer.panel") {
            break;
        }
    }

    for id in [
        "m3-modal-navigation-drawer",
        "m3-modal-navigation-drawer.scrim",
        "m3-modal-navigation-drawer.scrim.chrome",
        "m3-modal-navigation-drawer.panel",
        "m3-modal-navigation-drawer-content",
        "m3-modal-navigation-drawer-content.chrome",
        "m3-modal-drawer-search",
        "m3-modal-drawer-search.chrome",
        "m3-modal-drawer-search.icon",
        "m3-modal-drawer-search.label",
        "m3-modal-drawer-settings",
        "m3-modal-drawer-settings.chrome",
        "m3-modal-drawer-settings.icon",
        "m3-modal-drawer-settings.label",
        "m3-modal-drawer-settings.badge",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live ModalNavigationDrawer part test_id {id}"
        );
    }
}

#[test]
fn material3_search_bar_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::SearchBar;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(980.0), Px(180.0)),
    );

    let query = app.models_mut().insert(String::new());
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = SearchBar::new(query.clone())
                    .a11y_label("Material search")
                    .placeholder("Search")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .test_id("m3-search-bar")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-search-bar",
        "m3-search-bar.chrome",
        "m3-search-bar.leading-icon",
        "m3-search-bar.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live SearchBar part test_id {id}"
        );
    }

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-search-bar.chrome");
    assert_px_close(chrome.size.height.0, 56.0, "SearchBar chrome height");
    assert_px_close(chrome.size.width.0, 720.0, "SearchBar chrome max width");
}

#[test]
fn material3_search_view_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{SearchView, SearchViewPresentation};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let open = app.models_mut().insert(true);
    let query = app.models_mut().insert(String::new());
    let full_screen_open = app.models_mut().insert(true);
    let full_screen_query = app.models_mut().insert(String::new());
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let view = SearchView::new(open.clone(), query.clone())
                .a11y_label("Material search view")
                .placeholder("Search")
                .leading_icon(ids::ui::SEARCH)
                .trailing_icon(ids::ui::CLOSE)
                .test_id("m3-search-view")
                .into_element(cx, |cx| {
                    vec![
                        cx.text_props(fret_ui::element::TextProps::new(Arc::<str>::from("Result"))),
                    ]
                });
            let full_screen_view =
                SearchView::new(full_screen_open.clone(), full_screen_query.clone())
                    .a11y_label("Material full-screen search view")
                    .placeholder("Search full screen")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .test_id("m3-search-view-full")
                    .presentation(SearchViewPresentation::FullScreen)
                    .into_element(cx, |cx| {
                        vec![
                            cx.text_props(fret_ui::element::TextProps::new(Arc::<str>::from(
                                "Full-screen result",
                            ))),
                        ]
                    });
            vec![
                with_padding(cx, Px(32.0), view),
                with_padding(cx, Px(32.0), full_screen_view),
            ]
        })
    };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for _ in 0..4 {
        if live_test_id_exists(&ui, &app, window, "m3-search-view.overlay")
            && live_test_id_exists(&ui, &app, window, "m3-search-view-full.overlay.header")
        {
            break;
        }
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    for id in [
        "m3-search-view",
        "m3-search-view.chrome",
        "m3-search-view.leading-icon",
        "m3-search-view.trailing-icon",
        "m3-search-view.overlay",
        "m3-search-view.overlay.divider",
        "m3-search-view.overlay.body",
        "m3-search-view-full",
        "m3-search-view-full.chrome",
        "m3-search-view-full.leading-icon",
        "m3-search-view-full.trailing-icon",
        "m3-search-view-full.overlay",
        "m3-search-view-full.overlay.header-slot",
        "m3-search-view-full.overlay.header",
        "m3-search-view-full.overlay.header.chrome",
        "m3-search-view-full.overlay.header.leading-icon",
        "m3-search-view-full.overlay.header.trailing-icon",
        "m3-search-view-full.overlay.divider",
        "m3-search-view-full.overlay.body",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live SearchView part test_id {id}"
        );
    }

    let full_overlay = live_test_id_layout_bounds(&ui, &app, window, "m3-search-view-full.overlay");
    let full_header_slot =
        live_test_id_layout_bounds(&ui, &app, window, "m3-search-view-full.overlay.header-slot");
    let full_divider =
        live_test_id_layout_bounds(&ui, &app, window, "m3-search-view-full.overlay.divider");
    let full_body =
        live_test_id_layout_bounds(&ui, &app, window, "m3-search-view-full.overlay.body");
    assert_px_close(
        full_header_slot.origin.y.0 - full_overlay.origin.y.0,
        0.0,
        "full-screen SearchView header slot starts at overlay top",
    );
    assert_px_close(
        full_header_slot.size.height.0,
        72.0,
        "full-screen SearchView header slot height",
    );
    assert_px_close(
        full_divider.origin.y.0 - full_overlay.origin.y.0,
        72.0,
        "full-screen SearchView divider follows header slot",
    );
    assert_px_close(
        full_body.origin.y.0 - (full_divider.origin.y.0 + full_divider.size.height.0),
        0.0,
        "full-screen SearchView body follows divider",
    );
}

#[test]
fn material3_date_picker_exposes_stable_part_test_ids() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedDatePicker::new(month.clone(), selected.clone())
                        .variant(DatePickerVariant::Docked)
                        .today(Some(today))
                        .test_id("m3-date-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-date-picker",
            "m3-date-picker.chrome",
            "m3-date-picker.docked.month-label",
            "m3-date-picker.docked.prev",
            "m3-date-picker.docked.next",
            "m3-date-picker.docked.weekday.0",
            "m3-date-picker.docked.weekday.6",
            "m3-date-picker.cell.0.0",
            "m3-date-picker.cell.5.6",
            "m3-date-picker.cell.2025-12-29",
            "m3-date-picker.cell.2026-01-10",
            "m3-date-picker.cell.2026-01-15",
            "m3-date-picker.cell.2026-02-08",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DatePicker part test_id {id}"
            );
        }

        let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker.chrome");
        let weekday0 =
            live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker.docked.weekday.0");
        let weekday6 =
            live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker.docked.weekday.6");
        let cell00 = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker.cell.0.0");
        let cell06 = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker.cell.0.6");

        for (name, bounds) in [
            ("docked weekday 0", weekday0),
            ("docked weekday 6", weekday6),
            ("docked date cell 0,0", cell00),
            ("docked date cell 0,6", cell06),
        ] {
            assert_px_close(bounds.size.width.0, 48.0, &format!("{name} width"));
            assert_px_close(bounds.size.height.0, 48.0, &format!("{name} height"));
        }
        assert_px_close(
            weekday0.origin.x.0 - chrome.origin.x.0,
            12.0,
            "docked calendar horizontal padding",
        );
        assert_px_close(
            rect_center_x(weekday0),
            rect_center_x(cell00),
            "docked first weekday/cell center",
        );
        assert_px_close(
            rect_center_x(weekday6),
            rect_center_x(cell06),
            "docked last weekday/cell center",
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
                    .today(Some(today))
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-date-picker-modal")
                    .into_element(cx, |cx| {
                        Button::new("Underlay")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-date-picker-underlay")
                            .into_element(cx)
                    });
                vec![with_padding(cx, Px(32.0), dialog)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-date-picker-modal.panel") {
                break;
            }
        }

        for id in [
            "m3-date-picker-modal.scrim",
            "m3-date-picker-modal.scrim.chrome",
            "m3-date-picker-modal.panel",
            "m3-date-picker-modal.modal.title",
            "m3-date-picker-modal.modal.month-label",
            "m3-date-picker-modal.modal.prev",
            "m3-date-picker-modal.modal.next",
            "m3-date-picker-modal.modal.weekday.0",
            "m3-date-picker-modal.modal.weekday.6",
            "m3-date-picker-modal.cell.0.0",
            "m3-date-picker-modal.cell.2026-01-10",
            "m3-date-picker-modal.cell.2026-01-15",
            "m3-date-picker-modal.actions.cancel",
            "m3-date-picker-modal.actions.confirm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DatePicker dialog part test_id {id}"
            );
        }

        let panel = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker-modal.panel");
        let weekday0 =
            live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker-modal.modal.weekday.0");
        let weekday6 =
            live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker-modal.modal.weekday.6");
        let cell00 = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker-modal.cell.0.0");
        let cell06 = live_test_id_layout_bounds(&ui, &app, window, "m3-date-picker-modal.cell.0.6");

        for (name, bounds) in [
            ("modal weekday 0", weekday0),
            ("modal weekday 6", weekday6),
            ("modal date cell 0,0", cell00),
            ("modal date cell 0,6", cell06),
        ] {
            assert_px_close(bounds.size.width.0, 48.0, &format!("{name} width"));
            assert_px_close(bounds.size.height.0, 48.0, &format!("{name} height"));
        }
        assert_px_close(
            weekday0.origin.x.0 - panel.origin.x.0,
            12.0,
            "modal calendar horizontal padding",
        );
        assert_px_close(
            rect_center_x(weekday0),
            rect_center_x(cell00),
            "modal first weekday/cell center",
        );
        assert_px_close(
            rect_center_x(weekday6),
            rect_center_x(cell06),
            "modal last weekday/cell center",
        );
    }
}

#[test]
fn material3_date_picker_month_label_is_polite_live_region() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{DatePickerVariant, DockedDatePicker};
    use time::{Date, Month};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let month = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected = app.models_mut().insert(None::<Date>);
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedDatePicker::new(month.clone(), selected.clone())
                    .variant(DatePickerVariant::Docked)
                    .today(Some(today))
                    .test_id("m3-date-picker")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        semantics_node_label(&ui, "m3-date-picker.docked.month-label"),
        "January 2026",
        "initial month label should be exposed to semantics"
    );
    assert_eq!(
        semantics_node_live(&ui, "m3-date-picker.docked.month-label"),
        Some((SemanticsLive::Polite, true)),
        "month label should be a polite atomic live region"
    );

    click_semantics_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "m3-date-picker.docked.next",
        PointerId(50),
    );

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        semantics_node_label(&ui, "m3-date-picker.docked.month-label"),
        "February 2026",
        "month navigation should update the live region label"
    );
    assert_eq!(
        semantics_node_live(&ui, "m3-date-picker.docked.month-label"),
        Some((SemanticsLive::Polite, true)),
        "month label should remain live after navigation"
    );
}

#[test]
fn material3_date_picker_uses_material_string_registry_and_date_descriptions() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        install_date_picker_string_lookup(&mut app);
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedDatePicker::new(month.clone(), selected.clone())
                        .variant(DatePickerVariant::Docked)
                        .today(Some(today))
                        .test_id("m3-date-picker-i18n")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.month-label"),
            "Localized M01 2026"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.weekday.0"),
            "Localized Monday"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.weekday.6"),
            "Localized Sunday"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.prev"),
            "Localized previous month"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.next"),
            "Localized next month"
        );
        assert_eq!(
            semantics_node_role(&ui, "m3-date-picker-i18n.cell.1.5"),
            SemanticsRole::Button
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.cell.1.5"),
            "Localized today / Localized date 2026-01-10"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.cell.2.3"),
            "Localized date 2026-01-15"
        );
        assert!(
            semantics_node_selected(&ui, "m3-date-picker-i18n.cell.2.3"),
            "selected date cell should expose selected semantics"
        );

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-date-picker-i18n.docked.next",
            PointerId(51),
        );

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-i18n.docked.month-label"),
            "Localized M02 2026",
            "navigation should recompute the month label through the registry"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        install_date_picker_string_lookup(&mut app);
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
                    .today(Some(today))
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-date-picker-modal-i18n")
                    .into_element(cx, |cx| {
                        Button::new("Underlay")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-date-picker-modal-i18n-underlay")
                            .into_element(cx)
                    });
                vec![with_padding(cx, Px(32.0), dialog)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-date-picker-modal-i18n.panel") {
                break;
            }
        }

        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.scrim"),
            "Localized dismiss date picker"
        );
        assert_eq!(
            semantics_node_role(&ui, "m3-date-picker-modal-i18n.scrim"),
            SemanticsRole::Button
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.modal.title"),
            "Localized select date"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.modal.month-label"),
            "Localized M01 2026"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.modal.weekday.0"),
            "Localized Monday"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.modal.prev"),
            "Localized previous month"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.modal.next"),
            "Localized next month"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.cell.1.5"),
            "Localized today / Localized date 2026-01-10"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.actions.cancel"),
            "Localized cancel date"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-date-picker-modal-i18n.actions.confirm"),
            "Localized confirm date"
        );
    }
}

#[test]
fn material3_date_picker_respects_selectable_dates() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let blocked_date = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let allowed_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(None::<Date>);
        let selected_for_assert = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedDatePicker::new(month.clone(), selected.clone())
                        .variant(DatePickerVariant::Docked)
                        .today(Some(today))
                        .selectable_dates(move |date| date != blocked_date)
                        .test_id("m3-date-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            semantics_node_disabled(&ui, "m3-date-picker.cell.1.5"),
            "blocked Jan 10 cell should expose disabled semantics"
        );
        assert!(
            live_test_id_exists(&ui, &app, window, "m3-date-picker.cell.2026-01-10"),
            "blocked date should retain its value-level automation anchor"
        );

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-date-picker.cell.1.5",
            PointerId(30),
        );
        assert_eq!(
            app.models().get_copied(&selected_for_assert).flatten(),
            None,
            "blocked date activation must not update the selected model"
        );

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-date-picker.cell.2.3",
            PointerId(31),
        );
        assert_eq!(
            app.models().get_copied(&selected_for_assert).flatten(),
            Some(allowed_date),
            "allowed date activation should still update the selected model"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(None::<Date>);
        let selected_for_assert = selected.clone();
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
                    .today(Some(today))
                    .selectable_dates(move |date| date != blocked_date)
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-date-picker-modal")
                    .into_element(cx, |cx| {
                        Button::new("Underlay")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-date-picker-underlay")
                            .into_element(cx)
                    });
                vec![with_padding(cx, Px(32.0), dialog)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-date-picker-modal.panel") {
                break;
            }
        }

        assert!(
            semantics_node_disabled(&ui, "m3-date-picker-modal.cell.1.5"),
            "blocked modal Jan 10 cell should expose disabled semantics"
        );

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-date-picker-modal.cell.1.5",
            PointerId(40),
        );
        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-date-picker-modal.actions.confirm",
            PointerId(41),
        );

        assert_eq!(
            app.models().get_copied(&selected_for_assert).flatten(),
            None,
            "blocked modal date activation must not commit through OK"
        );
    }
}

#[test]
fn material3_time_picker_exposes_stable_part_test_ids() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedTimePicker, TimePickerDialog, TimePickerDisplayMode,
    };
    use time::Time;

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Dial)
                        .test_id("m3-time-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-time-picker",
            "m3-time-picker.chrome",
            "m3-time-picker.mode-toggle",
            "m3-time-picker.hour-selector",
            "m3-time-picker.hour-selector.chrome",
            "m3-time-picker.separator",
            "m3-time-picker.minute-selector",
            "m3-time-picker.minute-selector.chrome",
            "m3-time-picker.clock-dial",
            "m3-time-picker.clock-dial.chrome",
            "m3-time-picker.period",
            "m3-time-picker.period.am",
            "m3-time-picker.period.pm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker dial part test_id {id}"
            );
        }

        for id in [
            "m3-time-picker.clock-dial.hour.12",
            "m3-time-picker.clock-dial.hour.01",
            "m3-time-picker.clock-dial.hour.02",
            "m3-time-picker.clock-dial.hour.03",
            "m3-time-picker.clock-dial.hour.04",
            "m3-time-picker.clock-dial.hour.05",
            "m3-time-picker.clock-dial.hour.06",
            "m3-time-picker.clock-dial.hour.07",
            "m3-time-picker.clock-dial.hour.08",
            "m3-time-picker.clock-dial.hour.09",
            "m3-time-picker.clock-dial.hour.10",
            "m3-time-picker.clock-dial.hour.11",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker hour dial label test_id {id}"
            );
        }

        let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.chrome");
        let hour_bounds =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.hour-selector");
        let separator_bounds =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.separator");
        let minute_bounds =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.minute-selector");
        let period_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.period");
        let dial_bounds =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker.clock-dial");
        assert_px_close(
            hour_bounds.size.width.0,
            96.0,
            "time picker hour selector width",
        );
        assert_px_close(
            hour_bounds.size.height.0,
            80.0,
            "time picker hour selector height",
        );
        assert_px_close(
            separator_bounds.size.width.0,
            24.0,
            "time picker display separator width",
        );
        assert_px_close(
            separator_bounds.size.height.0,
            80.0,
            "time picker display separator height",
        );
        assert_px_close(
            separator_bounds.origin.x.0 - (hour_bounds.origin.x.0 + hour_bounds.size.width.0),
            0.0,
            "time picker display separator follows hour selector",
        );
        assert_px_close(
            minute_bounds.origin.x.0
                - (separator_bounds.origin.x.0 + separator_bounds.size.width.0),
            0.0,
            "time picker minute selector follows display separator",
        );
        assert_px_close(
            period_bounds.origin.y.0,
            hour_bounds.origin.y.0,
            "time picker period selector aligns with display row",
        );
        assert_px_close(
            period_bounds.origin.x.0 - (minute_bounds.origin.x.0 + minute_bounds.size.width.0),
            12.0,
            "time picker period selector display-row margin",
        );
        assert_px_close(
            rect_center_x(dial_bounds),
            rect_center_x(chrome),
            "time picker clock dial is centered in container",
        );

        let minute_selector = semantics_node_id_by_test_id(&ui, "m3-time-picker.minute-selector")
            .expect("expected TimePicker minute selector semantics node");
        let minute_bounds = ui
            .debug_node_visual_bounds(minute_selector)
            .expect("expected TimePicker minute selector bounds");
        let click_at = Point::new(
            Px(minute_bounds.origin.x.0 + minute_bounds.size.width.0 * 0.5),
            Px(minute_bounds.origin.y.0 + minute_bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-time-picker.clock-dial.minute.00",
            "m3-time-picker.clock-dial.minute.05",
            "m3-time-picker.clock-dial.minute.10",
            "m3-time-picker.clock-dial.minute.15",
            "m3-time-picker.clock-dial.minute.20",
            "m3-time-picker.clock-dial.minute.25",
            "m3-time-picker.clock-dial.minute.30",
            "m3-time-picker.clock-dial.minute.35",
            "m3-time-picker.clock-dial.minute.40",
            "m3-time-picker.clock-dial.minute.45",
            "m3-time-picker.clock-dial.minute.50",
            "m3-time-picker.clock-dial.minute.55",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker minute dial label test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Input)
                        .test_id("m3-time-picker-input")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-time-picker-input",
            "m3-time-picker-input.chrome",
            "m3-time-picker-input.mode-toggle",
            "m3-time-picker-input.input.hour",
            "m3-time-picker-input.input.hour.chrome",
            "m3-time-picker-input.input.hour.supporting-text",
            "m3-time-picker-input.input.separator",
            "m3-time-picker-input.input.minute",
            "m3-time-picker-input.input.minute.chrome",
            "m3-time-picker-input.input.minute.supporting-text",
            "m3-time-picker-input.input.period",
            "m3-time-picker-input.input.period.am",
            "m3-time-picker-input.input.period.pm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker input part test_id {id}"
            );
        }

        let input_hour =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker-input.input.hour");
        let input_separator =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker-input.input.separator");
        let input_minute =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker-input.input.minute");
        let input_period =
            live_test_id_layout_bounds(&ui, &app, window, "m3-time-picker-input.input.period");
        assert_px_close(input_hour.size.width.0, 96.0, "time input hour field width");
        assert_px_close(
            input_hour.size.height.0,
            72.0,
            "time input hour field height",
        );
        assert_px_close(
            input_separator.size.width.0,
            24.0,
            "time input separator width",
        );
        assert_px_close(
            input_separator.size.height.0,
            72.0,
            "time input separator height",
        );
        assert_px_close(
            input_separator.origin.x.0 - (input_hour.origin.x.0 + input_hour.size.width.0),
            0.0,
            "time input separator follows hour field",
        );
        assert_px_close(
            input_minute.origin.x.0 - (input_separator.origin.x.0 + input_separator.size.width.0),
            0.0,
            "time input minute field follows separator",
        );
        assert_px_close(
            input_period.origin.y.0,
            input_hour.origin.y.0,
            "time input period selector aligns to field top",
        );
        assert_px_close(
            input_period.origin.x.0 - (input_minute.origin.x.0 + input_minute.size.width.0),
            12.0,
            "time input period selector field-row margin",
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = TimePickerDialog::new(open.clone(), time.clone())
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .test_id("m3-time-picker-modal")
                        .into_element(cx, |cx| {
                            Button::new("Underlay")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-time-picker-underlay")
                                .into_element(cx)
                        });
                    vec![with_padding(cx, Px(32.0), dialog)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-time-picker-modal.panel") {
                break;
            }
        }

        for id in [
            "m3-time-picker-modal.scrim",
            "m3-time-picker-modal.scrim.chrome",
            "m3-time-picker-modal.panel",
            "m3-time-picker-modal.actions.cancel",
            "m3-time-picker-modal.actions.confirm",
            "m3-time-picker-modal.clock-dial",
            "m3-time-picker-modal.clock-dial.chrome",
            "m3-time-picker-modal.clock-dial.hour.09",
            "m3-time-picker-modal.hour-selector",
            "m3-time-picker-modal.minute-selector",
            "m3-time-picker-modal.period",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker dialog part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_time_picker_uses_compose_aligned_accessibility_labels() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app
            .models_mut()
            .insert(Time::from_hms(9, 41, 0).expect("valid time"));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Dial)
                        .test_id("m3-time-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_role(&ui, "m3-time-picker.hour-selector"),
            SemanticsRole::RadioButton
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker.hour-selector"),
            "Select hour"
        );
        assert_eq!(
            semantics_node_value(&ui, "m3-time-picker.hour-selector").as_deref(),
            Some("9 o'clock")
        );
        assert!(semantics_node_selected(&ui, "m3-time-picker.hour-selector"));
        assert_eq!(
            semantics_node_role(&ui, "m3-time-picker.minute-selector"),
            SemanticsRole::RadioButton
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker.minute-selector"),
            "Select minutes"
        );
        assert_eq!(
            semantics_node_value(&ui, "m3-time-picker.minute-selector").as_deref(),
            Some("41 minutes")
        );
        assert!(!semantics_node_selected(
            &ui,
            "m3-time-picker.minute-selector"
        ));
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker.clock-dial.hour.09"),
            "9 o'clock"
        );
        assert_eq!(
            semantics_node_role(&ui, "m3-time-picker.period"),
            SemanticsRole::Group
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker.period"),
            "Select AM or PM"
        );
        assert!(semantics_node_selected(&ui, "m3-time-picker.period.am"));
        assert!(!semantics_node_selected(&ui, "m3-time-picker.period.pm"));

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-time-picker.minute-selector",
            PointerId(1),
        );
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(!semantics_node_selected(
            &ui,
            "m3-time-picker.hour-selector"
        ));
        assert!(semantics_node_selected(
            &ui,
            "m3-time-picker.minute-selector"
        ));
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker.clock-dial.minute.55"),
            "55 minutes"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app
            .models_mut()
            .insert(Time::from_hms(21, 5, 0).expect("valid time"));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Input)
                        .test_id("m3-time-picker-input")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_role(&ui, "m3-time-picker-input.input.hour"),
            SemanticsRole::TextField
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input.input.hour"),
            "for hour"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input.input.minute"),
            "for minutes"
        );
        assert_eq!(
            semantics_node_role(&ui, "m3-time-picker-input.input.period"),
            SemanticsRole::Group
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input.input.period"),
            "Select AM or PM"
        );
        assert!(!semantics_node_selected(
            &ui,
            "m3-time-picker-input.input.period.am"
        ));
        assert!(semantics_node_selected(
            &ui,
            "m3-time-picker-input.input.period.pm"
        ));
    }
}

#[test]
fn material3_time_picker_uses_material_string_registry() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedTimePicker, TimePickerDialog, TimePickerDisplayMode,
    };
    use time::Time;

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        install_time_picker_string_lookup(&mut app);
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app
            .models_mut()
            .insert(Time::from_hms(9, 41, 0).expect("valid time"));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Dial)
                        .test_id("m3-time-picker-i18n")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.mode-toggle"),
            "Localized text input mode"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.hour-selector"),
            "Localized hour selector"
        );
        assert_eq!(
            semantics_node_value(&ui, "m3-time-picker-i18n.hour-selector").as_deref(),
            Some("Localized 12h hour 9")
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.minute-selector"),
            "Localized minute selector"
        );
        assert_eq!(
            semantics_node_value(&ui, "m3-time-picker-i18n.minute-selector").as_deref(),
            Some("Localized minute 41")
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.clock-dial.hour.09"),
            "Localized 12h hour 9"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.period"),
            "Localized period toggle"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.period.am"),
            "Localized AM"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.period.pm"),
            "Localized PM"
        );

        click_semantics_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "m3-time-picker-i18n.minute-selector",
            PointerId(1),
        );
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-i18n.clock-dial.minute.55"),
            "Localized minute 55"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        install_time_picker_string_lookup(&mut app);
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app
            .models_mut()
            .insert(Time::from_hms(21, 5, 0).expect("valid time"));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Input)
                        .test_id("m3-time-picker-input-i18n")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.mode-toggle"),
            "Localized clock mode"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.hour"),
            "Localized hour input"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.hour.supporting-text"),
            "Localized hour supporting text"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.minute"),
            "Localized minute input"
        );
        assert_eq!(
            semantics_node_label(
                &ui,
                "m3-time-picker-input-i18n.input.minute.supporting-text"
            ),
            "Localized minute supporting text"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.period"),
            "Localized period toggle"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.period.am"),
            "Localized AM"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-input-i18n.input.period.pm"),
            "Localized PM"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        install_time_picker_string_lookup(&mut app);
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let time = app
            .models_mut()
            .insert(Time::from_hms(9, 41, 0).expect("valid time"));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = TimePickerDialog::new(open.clone(), time.clone())
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .test_id("m3-time-picker-modal-i18n")
                        .into_element(cx, |cx| {
                            Button::new("Underlay")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-time-picker-underlay-i18n")
                                .into_element(cx)
                        });
                    vec![with_padding(cx, Px(32.0), dialog)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-time-picker-modal-i18n.panel") {
                break;
            }
        }

        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-modal-i18n.scrim"),
            "Localized dismiss"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-modal-i18n.actions.cancel"),
            "Localized cancel"
        );
        assert_eq!(
            semantics_node_label(&ui, "m3-time-picker-modal-i18n.actions.confirm"),
            "Localized ok"
        );
    }
}

#[test]
fn material3_menu_and_dropdown_expose_stable_part_test_ids() {
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};
    use fret_ui_material3::{Button, ButtonVariant, DropdownMenu};

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(260.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = Menu::new()
                        .a11y_label("Material menu")
                        .test_id("m3-menu")
                        .entries(vec![
                            MenuEntry::Item(MenuItem::new("Alpha").test_id("m3-menu-alpha")),
                            MenuEntry::Separator,
                            MenuEntry::Item(MenuItem::new("Beta").test_id("m3-menu-beta")),
                        ])
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), menu)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-menu",
            "m3-menu.chrome",
            "m3-menu-alpha",
            "m3-menu-alpha.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live Menu part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(360.0)),
        );

        let open = app.models_mut().insert(true);
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dropdown = DropdownMenu::new(open.clone())
                    .a11y_label("Material dropdown")
                    .test_id("m3-dropdown")
                    .into_element(
                        cx,
                        |cx| {
                            Button::new("Open")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-dropdown-trigger")
                                .into_element(cx)
                        },
                        |_cx| {
                            vec![
                                MenuEntry::Item(
                                    MenuItem::new("Alpha").test_id("m3-dropdown-alpha"),
                                ),
                                MenuEntry::Item(MenuItem::new("Beta").test_id("m3-dropdown-beta")),
                            ]
                        },
                    );
                vec![with_padding(cx, Px(32.0), dropdown)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-dropdown.chrome") {
                break;
            }
        }

        for id in [
            "m3-dropdown",
            "m3-dropdown.chrome",
            "m3-dropdown-alpha",
            "m3-dropdown-alpha.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DropdownMenu part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_dialog_and_bottom_sheet_expose_stable_part_test_ids() {
    use fret_ui_material3::{
        Button, ButtonVariant, Dialog, DialogAction, DockedBottomSheet, DockedBottomSheetVariant,
        ModalBottomSheet,
    };

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(420.0)),
        );

        let open = app.models_mut().insert(true);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = Dialog::new(open.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .actions(vec![
                            DialogAction::new("Cancel").test_id("m3-dialog-action-cancel"),
                            DialogAction::new("OK").test_id("m3-dialog-action-confirm"),
                        ])
                        .test_id("m3-dialog")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Underlay")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("m3-dialog-underlay")
                                    .into_element(cx)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![with_padding(cx, Px(32.0), dialog)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-dialog.panel") {
                break;
            }
        }

        for id in [
            "m3-dialog.scrim",
            "m3-dialog.scrim.chrome",
            "m3-dialog.panel",
            "m3-dialog.panel.chrome",
            "m3-dialog-action-cancel",
            "m3-dialog-action-cancel.chrome",
            "m3-dialog-action-confirm",
            "m3-dialog-action-confirm.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live Dialog part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(420.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let sheet = DockedBottomSheet::new()
                        .variant(DockedBottomSheetVariant::Standard)
                        .test_id("m3-bottom-sheet")
                        .into_element(cx, |cx| {
                            vec![
                                Button::new("Action")
                                    .variant(ButtonVariant::Filled)
                                    .test_id("m3-bottom-sheet-action")
                                    .into_element(cx),
                            ]
                        });
                    vec![with_padding(cx, Px(32.0), sheet)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-bottom-sheet",
            "m3-bottom-sheet.chrome",
            "m3-bottom-sheet.drag-handle",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DockedBottomSheet part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let sheet = ModalBottomSheet::new(open.clone())
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .test_id("m3-modal-bottom-sheet")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Underlay")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("m3-modal-bottom-sheet-underlay")
                                    .into_element(cx)
                            },
                            |cx| {
                                vec![
                                    Button::new("Action")
                                        .variant(ButtonVariant::Filled)
                                        .test_id("m3-modal-bottom-sheet-action")
                                        .into_element(cx),
                                ]
                            },
                        );
                    vec![with_padding(cx, Px(32.0), sheet)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-modal-bottom-sheet.sheet") {
                break;
            }
        }

        for id in [
            "m3-modal-bottom-sheet.scrim",
            "m3-modal-bottom-sheet.scrim.chrome",
            "m3-modal-bottom-sheet.sheet",
            "m3-modal-bottom-sheet.sheet.chrome",
            "m3-modal-bottom-sheet.sheet.drag-handle",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live ModalBottomSheet part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_tooltip_and_snackbar_expose_stable_part_test_ids() {
    use fret_runtime::CommandId;
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{
        Button, ButtonVariant, PlainTooltip, RichTooltip, Snackbar, SnackbarController,
        SnackbarHost, TooltipProvider,
    };

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(320.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with_elements(cx, |cx| {
                            let trigger = Button::new("Trigger")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-tooltip-trigger")
                                .into_element(cx);
                            let tooltip = PlainTooltip::new(trigger, "Tip")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .test_id("m3-tooltip")
                                .into_element(cx);
                            let rich_trigger = Button::new("Rich")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-rich-tooltip-trigger")
                                .into_element(cx);
                            let rich_tooltip = RichTooltip::new(rich_trigger, "Rich supporting")
                                .title("Rich title")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .test_id("m3-rich-tooltip")
                                .into_element(cx);
                            let rich_no_title_trigger = Button::new("Rich / no title")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-rich-tooltip-no-title-trigger")
                                .into_element(cx);
                            let rich_no_title = RichTooltip::new(
                                rich_no_title_trigger,
                                "Rich supporting without title",
                            )
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .test_id("m3-rich-tooltip-no-title")
                            .into_element(cx);

                            let mut props = fret_ui::element::FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                            let tooltips =
                                cx.flex(props, |_cx| vec![tooltip, rich_tooltip, rich_no_title]);
                            vec![with_padding(cx, Px(32.0), tooltips)]
                        })
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger = semantics_node_id_by_test_id(&ui, "m3-tooltip-trigger")
            .expect("expected m3-tooltip-trigger semantics node");
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger)
            .expect("expected tooltip trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-tooltip") {
                break;
            }
        }

        for id in ["m3-tooltip", "m3-tooltip.chrome"] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live PlainTooltip part test_id {id}"
            );
        }

        let rich_trigger = semantics_node_id_by_test_id(&ui, "m3-rich-tooltip-trigger")
            .expect("expected m3-rich-tooltip-trigger semantics node");
        let rich_trigger_bounds = ui
            .debug_node_visual_bounds(rich_trigger)
            .expect("expected rich tooltip trigger bounds");
        let hover_at = Point::new(
            Px(rich_trigger_bounds.origin.x.0 + rich_trigger_bounds.size.width.0 * 0.5),
            Px(rich_trigger_bounds.origin.y.0 + rich_trigger_bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-rich-tooltip") {
                break;
            }
        }

        for id in [
            "m3-rich-tooltip",
            "m3-rich-tooltip.chrome",
            "m3-rich-tooltip.title",
            "m3-rich-tooltip.supporting-text",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live RichTooltip part test_id {id}"
            );
        }

        let rich_no_title_trigger =
            semantics_node_id_by_test_id(&ui, "m3-rich-tooltip-no-title-trigger")
                .expect("expected m3-rich-tooltip-no-title-trigger semantics node");
        let rich_no_title_trigger_bounds = ui
            .debug_node_visual_bounds(rich_no_title_trigger)
            .expect("expected no-title rich tooltip trigger bounds");
        let hover_at = Point::new(
            Px(rich_no_title_trigger_bounds.origin.x.0
                + rich_no_title_trigger_bounds.size.width.0 * 0.5),
            Px(rich_no_title_trigger_bounds.origin.y.0
                + rich_no_title_trigger_bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-rich-tooltip-no-title") {
                break;
            }
        }

        for id in [
            "m3-rich-tooltip-no-title",
            "m3-rich-tooltip-no-title.chrome",
            "m3-rich-tooltip-no-title.supporting-text",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live RichTooltip no-title part test_id {id}"
            );
        }
        assert!(
            !live_test_id_exists(&ui, &app, window, "m3-rich-tooltip-no-title.title"),
            "no-title RichTooltip should not expose a synthetic title part"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

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
            let mut action_host = UiActionHostAdapter { app: &mut app };
            let _ = controller.show(
                &mut action_host,
                window,
                Snackbar::new("Saved")
                    .supporting_text("Synced")
                    .action_id("Undo", CommandId::new("m3.snackbar.undo"))
                    .test_id("m3-snackbar"),
            );
        }

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let store = store.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        for id in ["m3-snackbar", "m3-snackbar.action", "m3-snackbar.close"] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live Snackbar toast part test_id {id}"
            );
        }
    }
}
