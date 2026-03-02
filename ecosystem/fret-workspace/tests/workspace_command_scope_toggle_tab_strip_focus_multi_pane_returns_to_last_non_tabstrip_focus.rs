use fret_app::App;
use fret_core::{
    AppWindowId, Axis, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::CommandId;
use fret_ui::element::{FlexProps, LayoutStyle, Length, MainAlign, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::commands::{CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS, pane_activate_command};
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
        "workspace-command-scope-toggle-tabstrip-multi-pane",
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
                    direction: Axis::Vertical,
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

#[test]
fn toggle_focus_is_pane_scoped_and_returns_to_last_non_tabstrip_focus() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(240.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("main", "pane-a");
    layout.pane_tree = WorkspacePaneTree::split(
        Axis::Horizontal,
        0.5,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    layout.active_pane = Some(Arc::from("pane-a"));
    {
        let a = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        a.tabs.open_and_activate(Arc::from("a1"));
        a.tabs.open_and_activate(Arc::from("a2"));
        let b = layout.pane_tree.find_pane_mut("pane-b").unwrap();
        b.tabs.open_and_activate(Arc::from("b1"));
        b.tabs.open_and_activate(Arc::from("b2"));
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

    // Render another frame so the command scope can record the last non-tabstrip focus target.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );

    let toggle = CommandId::from(CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS);
    let a_active_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-a2");
    ui.dispatch_command(&mut app, &mut services, &toggle);
    assert_eq!(ui.focus(), Some(a_active_tab));

    // Switch active pane to B while focus is still inside pane A's tab strip.
    let cmd = pane_activate_command("pane-b").unwrap();
    let _ = app
        .models_mut()
        .update(&window_layout, |w| w.apply_command(&cmd));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout,
    );

    let b_active_tab = find_node_by_test_id(&ui, "pane-pane-b-tab-b2");
    ui.dispatch_command(&mut app, &mut services, &toggle);
    assert_eq!(ui.focus(), Some(b_active_tab));

    // Toggling back should restore to the last *non-tabstrip* focus target (outside), not to pane
    // A's tab element.
    ui.dispatch_command(&mut app, &mut services, &toggle);
    assert_eq!(ui.focus(), Some(outside));
}
