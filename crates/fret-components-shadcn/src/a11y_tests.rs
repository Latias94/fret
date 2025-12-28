use crate::test_host::TestHost;
use crate::{AccordionTrigger, CollapsibleTrigger, DatePicker, InputOTP, Toggle};
use fret_components_ui::{PopoverService, PopoverSurfaceRequest, PopoverSurfaceService};
use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    PathStyle, Px, Rect, SemanticsRole, Size, TextConstraints, TextMetrics, TextService, TextStyle,
    geometry::Point,
};
use fret_runtime::{Model, UiHost as _};
use fret_ui::UiTree;

struct FakeUiServices;

impl TextService for FakeUiServices {
    fn prepare(
        &mut self,
        _text: &str,
        _style: TextStyle,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeUiServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl fret_core::SvgService for FakeUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

fn run_frame(ui: &mut UiTree<TestHost>, host: &mut TestHost, services: &mut FakeUiServices) {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );
    ui.layout_all(host, services, bounds, 1.0);
}

fn request_snapshot(
    ui: &mut UiTree<TestHost>,
    host: &mut TestHost,
    services: &mut FakeUiServices,
) -> std::sync::Arc<fret_core::SemanticsSnapshot> {
    ui.request_semantics_snapshot();
    run_frame(ui, host, services);
    ui.semantics_snapshot_arc().expect("semantics snapshot")
}

fn a11y_invoke(
    ui: &mut UiTree<TestHost>,
    host: &mut TestHost,
    services: &mut FakeUiServices,
    id: fret_core::NodeId,
) {
    ui.set_focus(Some(id));
    ui.dispatch_event(
        host,
        services,
        &Event::KeyDown {
            key: KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        host,
        services,
        &Event::KeyUp {
            key: KeyCode::Space,
            modifiers: Modifiers::default(),
        },
    );
}

#[test]
fn collapsible_trigger_exposes_invoke_action_and_expanded_state() {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let open: Model<bool> = host.models_mut().insert(false);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);
    let trigger = ui.create_node(CollapsibleTrigger::new(open));
    ui.add_child(root, trigger);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == trigger)
        .expect("trigger node");
    assert_eq!(node.role, SemanticsRole::Button);
    assert!(node.actions.invoke);
    assert!(!node.flags.expanded);

    a11y_invoke(&mut ui, &mut host, &mut services, trigger);
    run_frame(&mut ui, &mut host, &mut services);
    assert_eq!(host.models().get(open).copied(), Some(true));

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == trigger)
        .expect("trigger node");
    assert!(node.flags.expanded);
}

#[test]
fn accordion_trigger_exposes_invoke_action_and_expanded_state() {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let selection: Model<Option<std::sync::Arc<str>>> = host.models_mut().insert(None);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);
    let trigger = ui.create_node(AccordionTrigger::single(selection, "item"));
    ui.add_child(root, trigger);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == trigger)
        .expect("trigger node");
    assert_eq!(node.role, SemanticsRole::Button);
    assert!(node.actions.invoke);
    assert!(!node.flags.expanded);

    a11y_invoke(&mut ui, &mut host, &mut services, trigger);
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == trigger)
        .expect("trigger node");
    assert!(node.flags.expanded);
}

#[test]
fn toggle_sets_label_and_exposes_invoke_action() {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let model: Model<bool> = host.models_mut().insert(false);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);
    let toggle = ui.create_node(Toggle::new(model, "Bold"));
    ui.add_child(root, toggle);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == toggle)
        .expect("toggle node");
    assert_eq!(node.role, SemanticsRole::Button);
    assert_eq!(node.label.as_deref(), Some("Bold"));
    assert!(node.actions.invoke);
}

#[test]
fn date_picker_sets_label_and_expanded_state() {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    host.set_global(PopoverSurfaceService::default());

    let model: Model<Option<crate::Date>> = host.models_mut().insert(None);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);

    let content = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.add_child(root, content);

    let picker = ui.create_node(DatePicker::new(model, content));
    ui.add_child(root, picker);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == picker)
        .expect("picker node");
    assert_eq!(node.role, SemanticsRole::Button);
    assert_eq!(node.label.as_deref(), Some("Pick a date"));
    assert!(node.actions.invoke);
    assert!(!node.flags.expanded);

    host.with_global_mut(PopoverSurfaceService::default, |service, _app| {
        service.set_request(
            window,
            PopoverSurfaceRequest::new(
                picker,
                Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(20.0), Px(10.0)),
                ),
                content,
            ),
        );
    });

    run_frame(&mut ui, &mut host, &mut services);
    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == picker)
        .expect("picker node");
    assert!(node.flags.expanded);
}

#[test]
fn input_otp_sets_label_value_and_editable_actions() {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let model: Model<String> = host.models_mut().insert("12".to_string());

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);
    let otp = ui.create_node(InputOTP::new(model));
    ui.add_child(root, otp);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let node = snap.nodes.iter().find(|n| n.id == otp).expect("otp node");
    assert_eq!(node.role, SemanticsRole::TextField);
    assert_eq!(node.label.as_deref(), Some("One-time password"));
    assert_eq!(node.value.as_deref(), Some("12"));
    assert!(node.actions.focus);
    assert!(node.actions.set_value);
}

#[test]
fn radio_group_exposes_list_items_and_a11y_invoke_selects_item() {
    use crate::radio_group::install_radio_group;
    use crate::{RadioGroup, RadioGroupItem};

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let model: Model<Option<std::sync::Arc<str>>> = host.models_mut().insert(None);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);

    let group = RadioGroup::new(model)
        .item(RadioGroupItem::new("a", "Apple"))
        .item(RadioGroupItem::new("b", "Banana"))
        .item(RadioGroupItem::new("c", "Cherry").disabled(true));
    let group_node = install_radio_group(&mut ui, root, group);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let group_sem = snap
        .nodes
        .iter()
        .find(|n| n.id == group_node)
        .expect("radio group semantics node");
    assert_eq!(group_sem.role, SemanticsRole::List);

    let cherry = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("Cherry"))
        .expect("Cherry item");
    assert!(cherry.flags.disabled);

    let banana_a11y_node = ui
        .children(group_node)
        .get(1)
        .copied()
        .expect("Banana a11y node");
    a11y_invoke(&mut ui, &mut host, &mut services, banana_a11y_node);
    run_frame(&mut ui, &mut host, &mut services);

    assert_eq!(
        host.models().get(model).cloned().flatten().as_deref(),
        Some("b")
    );

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let banana = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("Banana"))
        .expect("Banana item");
    assert!(banana.flags.selected);
    assert_eq!(banana.flags.checked, Some(true));
}

#[test]
fn toggle_group_exposes_button_items_and_a11y_invoke_toggles_selection() {
    use crate::toggle_group::install_toggle_group;
    use crate::{ToggleGroup, ToggleGroupItem};

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let model: Model<Option<std::sync::Arc<str>>> = host.models_mut().insert(None);

    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);

    let group = ToggleGroup::single(model)
        .item(ToggleGroupItem::new("bold", "Bold"))
        .item(ToggleGroupItem::new("italic", "Italic"));
    let group_node = install_toggle_group(&mut ui, root, group);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let bold = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Bold"))
        .expect("Bold button item");
    assert!(!bold.flags.selected);

    let bold_a11y_node = ui
        .children(group_node)
        .first()
        .copied()
        .expect("Bold a11y node");
    a11y_invoke(&mut ui, &mut host, &mut services, bold_a11y_node);
    run_frame(&mut ui, &mut host, &mut services);
    assert_eq!(
        host.models().get(model).cloned().flatten().as_deref(),
        Some("bold")
    );

    a11y_invoke(&mut ui, &mut host, &mut services, bold_a11y_node);
    run_frame(&mut ui, &mut host, &mut services);
    assert_eq!(host.models().get(model).cloned().flatten(), None);
}

#[test]
fn button_group_exposes_button_items_and_a11y_invoke_dispatches_command() {
    use crate::button_group::install_button_group;
    use crate::{ButtonGroup, ButtonGroupItem};
    use fret_runtime::{CommandId, Effect};

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);

    let cmd = CommandId::from("demo.button_group.bold");
    let group = ButtonGroup::new()
        .item(ButtonGroupItem::new("Bold").on_click(cmd.clone()))
        .item(ButtonGroupItem::new("Italic").disabled(true))
        .item(ButtonGroupItem::new("NoOp"));
    let group_node = install_button_group(&mut ui, root, group);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let bold = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Bold"))
        .expect("Bold button item");
    assert!(bold.actions.invoke);

    let italic = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Italic"))
        .expect("Italic button item");
    assert!(italic.flags.disabled);
    assert!(!italic.actions.invoke);

    let bold_a11y_node = ui
        .children(group_node)
        .first()
        .copied()
        .expect("Bold a11y node");
    a11y_invoke(&mut ui, &mut host, &mut services, bold_a11y_node);
    run_frame(&mut ui, &mut host, &mut services);

    let dispatched = host.effects().iter().any(|e| {
        matches!(
            e,
            Effect::Command { window: Some(w), command } if *w == window && command.as_str() == cmd.as_str()
        )
    });
    assert!(dispatched, "expected command to dispatch via a11y invoke");
}

#[test]
fn navigation_menu_exposes_menu_items_and_a11y_invoke_dispatches_link_command() {
    use crate::navigation_menu::install_navigation_menu;
    use crate::{NavigationMenu, NavigationMenuItem, NavigationMenuLink};
    use fret_runtime::{CommandId, Effect};

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut host = TestHost::default();
    let root = ui.create_node(fret_components_ui::widget_primitives::Column::new());
    ui.set_root(root);

    let new_cmd = CommandId::from("demo.nav.new");
    let open_cmd = CommandId::from("demo.nav.open");

    let menu = NavigationMenu::new()
        .item(
            NavigationMenuItem::new("File")
                .link(NavigationMenuLink::new("New").on_click(new_cmd.clone()))
                .link(
                    NavigationMenuLink::new("Open")
                        .on_click(open_cmd.clone())
                        .disabled(true),
                ),
        )
        .item(
            NavigationMenuItem::new("Edit")
                .link(NavigationMenuLink::new("Copy").on_click(CommandId::from("demo.nav.copy"))),
        );
    let menu_node = install_navigation_menu(&mut ui, root, menu);

    let mut services = FakeUiServices;
    run_frame(&mut ui, &mut host, &mut services);

    let snap = request_snapshot(&mut ui, &mut host, &mut services);
    let menu_sem = snap
        .nodes
        .iter()
        .find(|n| n.id == menu_node)
        .expect("menu semantics node");
    assert_eq!(menu_sem.role, SemanticsRole::MenuBar);

    let file_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("File trigger");
    assert!(file_trigger.actions.invoke);
    assert!(!file_trigger.flags.expanded);

    a11y_invoke(&mut ui, &mut host, &mut services, file_trigger.id);
    run_frame(&mut ui, &mut host, &mut services);

    let requested = host
        .global::<PopoverService>()
        .and_then(|s| s.request(window))
        .map(|(_, req)| req.clone());
    let req = requested.expect("popover request");
    assert_eq!(req.owner, file_trigger.id);
    assert!(req.items.iter().any(|it| it.label.as_ref() == "New"));
    assert!(req.items.iter().any(|it| it.label.as_ref() == "Open"));

    let dispatched_open = host.effects().iter().any(|e| {
        matches!(
            e,
            Effect::Command { window: Some(w), command } if *w == window && command.as_str() == "popover.open"
        )
    });
    assert!(dispatched_open, "expected popover.open command to dispatch");

    // Simulate selecting "New" (row 0), then any subsequent key event should allow the trigger
    // to consume the result and dispatch the link command.
    host.with_global_mut(PopoverService::default, |service, _app| {
        service.set_result(window, file_trigger.id, 0);
    });
    ui.set_focus(Some(file_trigger.id));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let dispatched_new = host.effects().iter().any(|e| {
        matches!(
            e,
            Effect::Command { window: Some(w), command } if *w == window && command.as_str() == new_cmd.as_str()
        )
    });
    assert!(dispatched_new, "expected selected link command to dispatch");

    // Disabled link should not dispatch.
    host.with_global_mut(PopoverService::default, |service, _app| {
        service.set_result(window, file_trigger.id, 1);
    });
    ui.set_focus(Some(file_trigger.id));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let dispatched_open_link = host.effects().iter().any(|e| {
        matches!(
            e,
            Effect::Command { window: Some(w), command } if *w == window && command.as_str() == open_cmd.as_str()
        )
    });
    assert!(
        !dispatched_open_link,
        "expected disabled link to not dispatch"
    );
}
