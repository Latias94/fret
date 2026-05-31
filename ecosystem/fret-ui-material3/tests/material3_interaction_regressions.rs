use std::sync::Arc;

use fret_core::{
    AppWindowId, Event, KeyCode, NodeId, Point, PointerId, Px, Rect, SemanticsRole, Size,
    UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{key_down, key_up, pointer_down, pointer_up};
use support::goldens::run_overlay_frame_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

// Residual Material3 interaction regressions that still need owner-boundary audits.

#[test]
fn text_input_text_input_event_updates_model() {
    use fret_ui::element::TextInputProps;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let model = app.models_mut().insert(String::new());
    let model_for_render = model.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = TextInputProps::new(model_for_render.clone());
                props.layout.size.width = fret_ui::element::Length::Px(Px(200.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(40.0));
                props.a11y_label = Some(Arc::<str>::from("input"));
                props.test_id = Some(Arc::<str>::from("plain-text-input"));
                let input = cx.text_input(props);
                vec![with_padding(cx, Px(24.0), input)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("plain-text-input")).then_some(node.id)
            })
        })
        .expect("expected plain-text-input in semantics snapshot");

    ui.set_focus(Some(input_node));
    assert_eq!(
        ui.focus(),
        Some(input_node),
        "expected focus to be set to the input node",
    );

    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let value = app.models().get_cloned(&model).expect("model exists");
    assert_eq!(value, "a", "expected text input event to update the model");
}

#[test]
fn material3_autocomplete_semantics_v1() {
    use fret_core::SemanticsRole;
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
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
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    // Frame 1: build stable input id + bounds.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");

    ui.set_focus(Some(input_node));

    // Frame 2: focus visible to the widget.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    // Open via keyboard.
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    // Frame 3/4: overlay created, then relationships stabilize (controls/active-descendant).
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "combobox input should report expanded=true while open"
    );

    let list = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.listbox"))
        .expect("listbox node");
    assert!(
        input.controls.contains(&list.id),
        "combobox input should control the listbox"
    );
    assert!(
        list.labelled_by.contains(&input.id),
        "listbox should be labelled by the combobox input"
    );

    let active = input
        .active_descendant
        .expect("active_descendant should be set");
    let active_node = snap
        .nodes
        .iter()
        .find(|n| n.id == active)
        .expect("active_descendant should reference a node in the snapshot");
    assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

    let beta = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.option.beta"))
        .expect("expected beta option node");
    assert!(beta.flags.selected, "expected beta to be marked selected");

    // Typing still works while the overlay is open.
    ui.set_focus(Some(input.id));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("a".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after typing");
    assert_eq!(input.value.as_deref(), Some("a"));
}

#[test]
fn material3_autocomplete_filters_items_by_query_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
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
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("ga".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after typing"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.gamma") }),
        "expected gamma option after typing 'ga'"
    );
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.alpha") }),
        "expected alpha option to be filtered out after typing 'ga'"
    );
}

#[test]
fn material3_autocomplete_enter_commits_and_does_not_reopen_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
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
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let selected_value_for_render = selected_value.clone();
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value_for_render.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
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
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to remain closed after Enter commit"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after Enter");
    assert_eq!(input.value.as_deref(), Some("Alpha"));

    let selected = app.models_mut().get_cloned(&selected_value).unwrap_or(None);
    assert_eq!(
        selected.as_deref(),
        Some("alpha"),
        "expected selected_value model to be committed on Enter"
    );
}

#[test]
fn material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown, TextField, TextFieldVariant};

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

    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();
    let other = app.models_mut().insert(String::new());

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);

                let other = TextField::new(other.clone())
                    .variant(TextFieldVariant::Outlined)
                    .label("Other")
                    .test_id("other-field")
                    .into_element(cx);

                let mut column = FlexProps::default();
                column.direction = fret_core::Axis::Vertical;
                column.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                column.layout.size.width = Length::Fill;

                let content = cx.flex(column, |_cx| vec![exposed, other]);
                vec![with_padding(cx, Px(24.0), content)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to synchronize from the committed selection while blurred"
    );

    let (input_node, other_node): (NodeId, NodeId) = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            let input = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown")).then_some(node.id)
            })?;
            let other = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("other-field")).then_some(node.id)
            })?;
            Some((input, other))
        })
        .expect("expected input and other nodes in semantics snapshot");

    ui.set_focus(Some(input_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&query, |v| *v = "ga".to_string());
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "ga",
        "expected query to remain editable while focused"
    );

    ui.set_focus(Some(other_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to revert to the committed selection label on blur"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("combobox input node after blur");
    assert_eq!(input.value.as_deref(), Some("Beta"));
}

#[test]
fn material3_exposed_dropdown_trailing_icon_toggles_overlay_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
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
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), exposed)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let icon_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown.trailing-icon"))
                    .then_some(node.id)
            })
        })
        .expect("expected trailing icon node in semantics snapshot");

    let icon_bounds = ui
        .debug_node_visual_bounds(icon_node)
        .expect("expected trailing icon bounds");
    let click_at = Point::new(
        Px(icon_bounds.origin.x.0 + icon_bounds.size.width.0 * 0.5),
        Px(icon_bounds.origin.y.0 + icon_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be open after clicking the trailing icon"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("expected exposed dropdown input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "exposed dropdown input should report expanded=true while open"
    );

    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown.listbox"))
        .expect("expected exposed dropdown listbox node");
    assert_eq!(listbox.role, SemanticsRole::ListBox);
    assert!(
        input.controls.contains(&listbox.id),
        "exposed dropdown input should control its listbox"
    );
    assert!(
        listbox.labelled_by.contains(&input.id),
        "exposed dropdown listbox should be labelled by its input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be closed after clicking the trailing icon again"
    );
}
