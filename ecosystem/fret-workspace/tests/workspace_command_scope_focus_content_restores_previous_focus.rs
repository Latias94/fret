use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService,
    TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::CommandId;
use fret_ui::element::{FlexProps, LayoutStyle, Length, MainAlign, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::commands::{
    CMD_WORKSPACE_PANE_FOCUS_CONTENT, CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP,
};
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{
    WorkspaceCommandScope, WorkspaceTabStrip, workspace_pane_tree_element_with_resize,
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
        "workspace-command-scope-focus-content",
        move |cx| {
            cx.observe_model(&window_layout, fret_ui::Invalidation::Layout);

            let outside = cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Px(Px(40.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("Outside")),
                        test_id: Some(Arc::from("outside-focus-target")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx, _st| vec![],
            );

            let mut render_pane = |cx: &mut fret_ui::ElementContext<'_, App>,
                                   pane: &fret_workspace::layout::WorkspacePaneLayout,
                                   _is_active: bool,
                                   tab_drag| {
                WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, |id| Arc::<str>::from(id))
                    .pane_id(pane.id.clone())
                    .test_id_root(Arc::<str>::from(format!(
                        "pane-{}-tabstrip",
                        pane.id.as_ref()
                    )))
                    .tab_test_id_prefix(Arc::<str>::from(format!("pane-{}-tab", pane.id.as_ref())))
                    .tab_drag_model(tab_drag)
                    .into_element(cx)
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
                |_cx| vec![outside, panes],
            );

            vec![WorkspaceCommandScope::new(window_layout.clone(), body).into_element(cx)]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn find_node_by_test_id(ui: &UiTree<App>, test_id: &str) -> fret_core::NodeId {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .map(|n| n.id)
        .unwrap_or_else(|| panic!("expected semantics node with test_id={test_id}"))
}

fn dispatch_escape(ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
}

fn drain_dispatched_commands(app: &mut App) -> Vec<CommandId> {
    app.flush_effects()
        .into_iter()
        .filter_map(|e| match e {
            fret_runtime::Effect::Command {
                window: Some(_),
                command,
            } => Some(command),
            _ => None,
        })
        .collect()
}

#[test]
fn focus_content_restores_pre_focus_target_after_focus_tab_strip() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

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
        assert_eq!(pane.tabs.active().unwrap().as_ref(), "b");
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

    let outside = find_node_by_test_id(&ui, "outside-focus-target");
    ui.set_focus(Some(outside));

    // Render another frame so `WorkspaceCommandScope` can observe the focused element and record it
    // as the "return focus" target when `focus_tab_strip` is invoked.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let active_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-b");
    let focus_tab_strip = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP);
    ui.dispatch_command(&mut app, &mut services, &focus_tab_strip);
    assert_eq!(ui.focus(), Some(active_tab));

    let focus_content = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_CONTENT);
    ui.dispatch_command(&mut app, &mut services, &focus_content);
    assert_eq!(ui.focus(), Some(outside));

    // Escape should be equivalent to `focus_content` when the tab strip is focused.
    ui.dispatch_command(&mut app, &mut services, &focus_tab_strip);
    assert_eq!(ui.focus(), Some(active_tab));
    dispatch_escape(&mut ui, &mut app, &mut services);
    for cmd in drain_dispatched_commands(&mut app) {
        ui.dispatch_command(&mut app, &mut services, &cmd);
    }
    assert_eq!(ui.focus(), Some(outside));
}
