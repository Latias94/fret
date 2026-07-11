use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::CommandId;
use fret_ui::element::{FlexProps, LayoutStyle, Length, MainAlign, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::commands::{
    CMD_WORKSPACE_PANE_FOCUS_CONTENT, CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP,
    CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS, act, typed_command_id,
};
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{
    WorkspaceCommandScope, WorkspacePaneContentFocusTarget, WorkspaceTabStrip, WorkspaceWorkbench,
    WorkspaceWorkbenchFocusFallback, workspace_pane_tree_element_with_resize,
};
use std::sync::Arc;

#[derive(Default)]
struct FakeServices;

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
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

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    window_layout: fret_runtime::Model<WorkspaceWindowLayout>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "workspace-command-scope-focus-content-fallback",
        move |cx| {
            cx.observe_model(&window_layout, fret_ui::Invalidation::Layout);

            let mut render_pane =
                |cx: &mut fret_ui::ElementContext<'_, App>,
                 pane: &fret_workspace::layout::WorkspacePaneLayout,
                 _is_active: bool,
                 tab_drag| {
                    let tab_strip = WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, |id| {
                        Arc::<str>::from(id)
                    })
                    .pane_id(pane.id.clone())
                    .test_id_root(Arc::<str>::from(format!(
                        "pane-{}-tabstrip",
                        pane.id.as_ref()
                    )))
                    .tab_test_id_prefix(Arc::<str>::from(format!("pane-{}-tab", pane.id.as_ref())))
                    .tab_drag_model(tab_drag)
                    .into_element(cx);

                    let active_tab = pane
                        .tabs
                        .active()
                        .cloned()
                        .unwrap_or_else(|| Arc::from("empty"));
                    let content_child_test_id = Arc::<str>::from(format!(
                        "pane-{}-content-child-{active_tab}",
                        pane.id.as_ref()
                    ));
                    let content_test_id =
                        Arc::<str>::from(format!("pane-{}-content-{active_tab}", pane.id.as_ref()));
                    let content = cx.keyed(active_tab, move |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::TextField),
                                    label: Some(Arc::from("Pane content")),
                                    test_id: Some(content_test_id),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, _st| {
                                vec![cx.pressable(
                                    PressableProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Px(Px(32.0));
                                            layout
                                        },
                                        enabled: true,
                                        focusable: true,
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::TextField),
                                            label: Some(Arc::from("Nested pane editor")),
                                            test_id: Some(content_child_test_id.clone()),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx, _st| vec![],
                                )]
                            },
                        )
                    });

                    let content = WorkspacePaneContentFocusTarget::new(pane.id.clone(), content)
                        .into_element(cx);

                    cx.flex(
                        FlexProps {
                            layout: fill_layout(),
                            direction: fret_core::Axis::Vertical,
                            justify: MainAlign::Start,
                            ..Default::default()
                        },
                        |_cx| vec![tab_strip, content],
                    )
                };

            let panes = workspace_pane_tree_element_with_resize(
                cx,
                window_layout.clone(),
                &mut render_pane,
            );

            let body = cx.flex(
                FlexProps {
                    layout: fill_layout(),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Start,
                    ..Default::default()
                },
                |_cx| vec![panes],
            );

            vec![WorkspaceCommandScope::new(window_layout.clone(), body).into_element(cx)]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn workbench_recognizes_a_nested_editor_as_active_pane_content() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(220.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::split(
        fret_core::Axis::Horizontal,
        0.5,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    layout.active_pane = Some(Arc::from("pane-a"));
    layout
        .pane_tree
        .find_pane_mut("pane-a")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("a"));
    layout
        .pane_tree
        .find_pane_mut("pane-b")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("b"));
    let window_layout = app.models_mut().insert(layout);
    let workbench = WorkspaceWorkbench::new(app.models_mut(), window_layout.clone(), false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );
    let nested_editor = find_node_by_test_id(&ui, "pane-pane-a-content-child-a");
    ui.set_focus(Some(nested_editor));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout,
    );

    let outcome = workbench.apply_command(
        &mut app,
        window,
        &typed_command_id::<act::WorkspacePaneFocusRight>(),
    );

    assert!(outcome.applied);
    assert_eq!(
        outcome.focus.as_ref().and_then(|focus| focus.fallback),
        Some(WorkspaceWorkbenchFocusFallback::ActivePaneContent)
    );
    ui.dispatch_command(
        &mut app,
        &mut services,
        &CommandId::from(CMD_WORKSPACE_PANE_FOCUS_CONTENT),
    );
    let pane_b_content = find_node_by_test_id(&ui, "pane-pane-b-content-b");
    assert_eq!(ui.focus(), Some(pane_b_content));

    let return_outcome = workbench.apply_command(
        &mut app,
        window,
        &typed_command_id::<act::WorkspacePaneFocusLeft>(),
    );
    assert!(return_outcome.applied);
    assert_eq!(
        return_outcome
            .focus
            .as_ref()
            .and_then(|focus| focus.fallback),
        Some(WorkspaceWorkbenchFocusFallback::ActivePaneContent),
        "the UI focus command must publish the new pane lane before another frame"
    );
}

#[test]
fn command_scope_leaves_model_commands_for_the_workbench_driver() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(220.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::split(
        fret_core::Axis::Horizontal,
        0.5,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    layout.active_pane = Some(Arc::from("pane-a"));
    layout
        .pane_tree
        .find_pane_mut("pane-a")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("a"));
    layout
        .pane_tree
        .find_pane_mut("pane-b")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("b"));
    let window_layout = app.models_mut().insert(layout);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let command = typed_command_id::<act::WorkspacePaneFocusRight>();
    assert!(!ui.dispatch_command(&mut app, &mut services, &command));
    assert_eq!(
        app.models()
            .read(&window_layout, |layout| layout.active_pane_id().cloned())
            .unwrap(),
        Some(Arc::from("pane-a")),
        "model commands must bubble to WorkspaceApp so Workbench owns policy and diagnostics"
    );
}

#[test]
fn cross_pane_move_rebuilds_the_active_tab_focus_target() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(220.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::split(
        fret_core::Axis::Horizontal,
        0.5,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    layout.active_pane = Some(Arc::from("pane-b"));
    layout
        .pane_tree
        .find_pane_mut("pane-a")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("a"));
    layout
        .pane_tree
        .find_pane_mut("pane-b")
        .unwrap()
        .tabs
        .open_and_activate(Arc::from("b"));
    let window_layout = app.models_mut().insert(layout);
    let workbench = WorkspaceWorkbench::new(app.models_mut(), window_layout.clone(), false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );
    let pane_b_tab = find_node_by_test_id(&ui, "pane-pane-b-tab-b");
    ui.set_focus(Some(pane_b_tab));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let outcome = workbench.apply_command(
        &mut app,
        window,
        &typed_command_id::<act::WorkspacePaneMoveActiveTabLeft>(),
    );
    assert!(outcome.applied);
    assert_eq!(
        app.models()
            .read(&window_layout, |layout| {
                let pane = layout.active_pane_id()?.clone();
                let tab = layout
                    .pane_tree
                    .find_pane(pane.as_ref())?
                    .tabs
                    .active()
                    .cloned()?;
                Some((pane, tab))
            })
            .unwrap(),
        Some((Arc::from("pane-a"), Arc::from("b")))
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout,
    );
    let moved_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-b");
    assert_eq!(
        ui.focus(),
        None,
        "moving the focused tab across panes must exercise the no-focus action route"
    );
    assert!(ui.dispatch_command(
        &mut app,
        &mut services,
        &CommandId::from(CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP),
    ));
    assert_eq!(ui.focus(), Some(moved_tab));
}

fn find_node_by_test_id(ui: &UiTree<App>, test_id: &str) -> fret_core::NodeId {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .map(|n| n.id)
        .unwrap_or_else(|| panic!("expected semantics node with test_id={test_id}"))
}

#[test]
fn focus_content_falls_back_to_registered_pane_content_when_no_return_target_recorded() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(180.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::leaf("pane-a");
    layout.active_pane = Some(Arc::from("pane-a"));
    {
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("a"));
        pane.tabs.open_and_activate(Arc::from("b"));
    }
    let window_layout = app.models_mut().insert(layout);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    // Simulate a “tab strip focused but no return target” situation (e.g. focus was moved into a
    // tab via non-command routing).
    let active_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-b");
    ui.set_focus(Some(active_tab));

    // Render another frame so `WorkspaceCommandScope` snapshots the focused element.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let focus_content = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_CONTENT);
    ui.dispatch_command(&mut app, &mut services, &focus_content);

    let content = find_node_by_test_id(&ui, "pane-pane-a-content-b");
    assert_eq!(ui.focus(), Some(content));
}

#[test]
fn toggle_focus_exits_to_registered_pane_content_when_no_return_target_recorded() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(180.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::leaf("pane-a");
    layout.active_pane = Some(Arc::from("pane-a"));
    {
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("a"));
        pane.tabs.open_and_activate(Arc::from("b"));
    }
    let window_layout = app.models_mut().insert(layout);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let active_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-b");
    ui.set_focus(Some(active_tab));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let toggle = CommandId::from(CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS);
    ui.dispatch_command(&mut app, &mut services, &toggle);

    let content = find_node_by_test_id(&ui, "pane-pane-a-content-b");
    assert_eq!(ui.focus(), Some(content));
}

#[test]
fn focus_content_discards_a_return_target_from_the_previous_active_tab() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(180.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::leaf("pane-a");
    layout.active_pane = Some(Arc::from("pane-a"));
    {
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("a"));
        pane.tabs.open_and_activate(Arc::from("b"));
    }
    let window_layout = app.models_mut().insert(layout);
    let workbench = WorkspaceWorkbench::new(app.models_mut(), window_layout.clone(), false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );
    let content_b = find_node_by_test_id(&ui, "pane-pane-a-content-b");
    ui.set_focus(Some(content_b));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let focus_tab_strip = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP);
    assert!(ui.dispatch_command(&mut app, &mut services, &focus_tab_strip));
    let activate_a = fret_workspace::commands::tab_activate_command("a").unwrap();
    let outcome = workbench.apply_command(&mut app, window, &activate_a);
    assert!(outcome.applied);
    assert_eq!(
        outcome.focus.as_ref().and_then(|focus| focus.fallback),
        Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip)
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout,
    );
    let focus_content = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_CONTENT);
    assert!(ui.dispatch_command(&mut app, &mut services, &focus_content));

    let content_a = find_node_by_test_id(&ui, "pane-pane-a-content-a");
    assert_eq!(ui.focus(), Some(content_a));
}
