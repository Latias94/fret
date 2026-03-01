use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::CommandId;
use fret_ui::element::{FlexProps, LayoutStyle, Length, MainAlign, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::commands::CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP;
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{WorkspaceTabStrip, workspace_pane_tree_element_with_resize};
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
        "workspace-pane-focus-tabstrip",
        move |cx| {
            cx.observe_model(&window_layout, fret_ui::Invalidation::Layout);

            let mut render_pane =
                |cx: &mut fret_ui::ElementContext<'_, App>,
                 pane: &fret_workspace::layout::WorkspacePaneLayout,
                 _is_active: bool,
                 tab_drag| {
                    let strip = WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, |id| {
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

                    let content_test_id: Arc<str> =
                        Arc::from(format!("pane-{}-content", pane.id.as_ref()));
                    let content = cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = fill_layout();
                                layout.size.height = Length::Px(Px(40.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("PaneContent")),
                                test_id: Some(content_test_id),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx, _st| vec![],
                    );

                    cx.flex(
                        FlexProps {
                            layout: fill_layout(),
                            direction: fret_core::Axis::Vertical,
                            justify: MainAlign::Start,
                            ..Default::default()
                        },
                        |_cx| vec![strip, content],
                    )
                };

            let center = workspace_pane_tree_element_with_resize(
                cx,
                window_layout.clone(),
                &mut render_pane,
            );
            vec![center]
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
fn focus_tab_strip_command_focuses_active_tab_in_focused_pane() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(140.0)),
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

    let content = find_node_by_test_id(&ui, "pane-pane-a-content");
    let active_tab = find_node_by_test_id(&ui, "pane-pane-a-tab-b");

    ui.set_focus(Some(content));
    let cmd = CommandId::from(CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP);
    ui.dispatch_command(&mut app, &mut services, &cmd);

    assert_eq!(
        ui.focus(),
        Some(active_tab),
        "expected focus to move to the active tab when focusing the pane tab strip"
    );
}
