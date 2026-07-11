use fret_app::App;
use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerId, PointerType, Px,
    Rect, SemanticsRole, SemanticsSnapshot, Size,
};
use fret_runtime::Model;
use fret_ui::{Theme, UiTree, declarative};

use super::{EditorThemePresetPicker, EditorThemePresetPickerOptions};
use crate::primitives::EditorTokenKeys;
use crate::test_support::WrappingTextServices;
use crate::theme::{
    EditorThemePreset, installed_editor_theme_preset, reapply_installed_editor_theme_preset,
};

#[test]
fn editor_theme_preset_picker_stamps_listbox_options_and_selected_state() {
    let window = AppWindowId::default();
    let bounds = test_bounds();
    let mut app = App::new();
    let mut services = WrappingTextServices;
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let model = app.models_mut().insert(EditorThemePreset::ImguiLikeDense);

    render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");

    let listbox = node_by_test_id(snapshot, "tests.theme_preset");
    assert_eq!(listbox.role, SemanticsRole::ListBox);
    assert_eq!(listbox.label.as_deref(), Some("Editor theme preset"));

    let default = node_by_test_id(snapshot, "tests.theme_preset.item.default");
    assert_eq!(default.role, SemanticsRole::ListBoxOption);
    assert_eq!(default.label.as_deref(), Some("Default"));
    assert_eq!(default.pos_in_set, Some(1));
    assert_eq!(default.set_size, Some(2));
    assert!(!default.flags.selected);

    let dense = node_by_test_id(snapshot, "tests.theme_preset.item.imgui_like_dense");
    assert_eq!(dense.role, SemanticsRole::ListBoxOption);
    assert_eq!(dense.label.as_deref(), Some("ImGui-like dense"));
    assert_eq!(dense.pos_in_set, Some(2));
    assert_eq!(dense.set_size, Some(2));
    assert!(dense.flags.selected);
}

#[test]
fn editor_theme_preset_picker_rows_show_density_status_labels() {
    assert_eq!(EditorThemePreset::Default.picker_status_label(), "24px");
    assert_eq!(
        EditorThemePreset::ImguiLikeDense.picker_status_label(),
        "22px"
    );
}

#[test]
fn editor_theme_preset_picker_click_updates_model_and_replays_reversible_preset() {
    let window = AppWindowId::default();
    let bounds = test_bounds();
    let mut app = App::new();
    let mut services = WrappingTextServices;
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let model = app.models_mut().insert(EditorThemePreset::Default);

    render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
    pump_semantics(&mut ui, &mut app, &mut services, bounds);
    assert_eq!(
        installed_editor_theme_preset(&app),
        Some(EditorThemePreset::Default)
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "tests.theme_preset.item.imgui_like_dense",
    );
    assert_eq!(
        app.models().get_copied(&model),
        Some(EditorThemePreset::ImguiLikeDense)
    );

    render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
    pump_semantics(&mut ui, &mut app, &mut services, bounds);
    assert_eq!(
        installed_editor_theme_preset(&app),
        Some(EditorThemePreset::ImguiLikeDense)
    );
    assert_eq!(
        Theme::global(&app).metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
        Some(Px(22.0))
    );
    assert_eq!(
        Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
        Some(Px(2.0))
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "tests.theme_preset.item.default",
    );
    assert_eq!(
        app.models().get_copied(&model),
        Some(EditorThemePreset::Default)
    );

    render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
    pump_semantics(&mut ui, &mut app, &mut services, bounds);
    assert_eq!(
        reapply_installed_editor_theme_preset(&mut app),
        Some(EditorThemePreset::Default)
    );
    assert_eq!(
        Theme::global(&app).metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
        Some(Px(24.0))
    );
    assert_eq!(
        Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED),
        Some(Px(0.02))
    );
    assert_eq!(
        Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
        Some(Px(4.0))
    );
}

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(120.0)),
    )
}

fn render_picker_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut WrappingTextServices,
    window: AppWindowId,
    bounds: Rect,
    model: &Model<EditorThemePreset>,
) {
    let model = model.clone();
    let root = declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "editor-theme-preset-picker-test",
        move |cx| {
            vec![
                EditorThemePresetPicker::new(model.clone())
                    .options(EditorThemePresetPickerOptions::new().test_id("tests.theme_preset"))
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
}

#[test]
fn editor_theme_preset_picker_options_builder_projects_fields() {
    let layout = fret_ui::element::LayoutStyle::default();

    let options = EditorThemePresetPickerOptions::new()
        .layout(layout)
        .disabled()
        .focusable(false)
        .label("Theme")
        .test_id("tests.theme_preset")
        .item_test_id_prefix("tests.theme_preset.item");

    assert!(!options.enabled);
    assert!(!options.focusable);
    assert_eq!(options.label.as_deref(), Some("Theme"));
    assert_eq!(options.test_id.as_deref(), Some("tests.theme_preset"));
    assert_eq!(
        options.item_test_id_prefix.as_deref(),
        Some("tests.theme_preset.item")
    );

    let options = options.enabled(true).without_label();
    assert!(options.enabled);
    assert!(options.label.is_none());
}

fn pump_semantics(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut WrappingTextServices,
    bounds: Rect,
) {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let mut scene = fret_core::Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

fn click_test_id(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut WrappingTextServices,
    test_id: &str,
) {
    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let node = node_by_test_id(snapshot, test_id);
    let bounds = node.bounds;
    let position = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            is_click: true,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn node_by_test_id<'a>(
    snapshot: &'a SemanticsSnapshot,
    test_id: &str,
) -> &'a fret_core::SemanticsNode {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected semantics node `{test_id}`"))
}
