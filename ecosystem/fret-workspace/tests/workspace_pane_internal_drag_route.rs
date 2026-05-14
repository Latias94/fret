use std::sync::Arc;

use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, InternalDragEvent, InternalDragKind, Modifiers, MouseButton,
    MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, Size as CoreSize, SvgId, SvgService,
    TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{CommandId, Effect, TickId};
use fret_ui::element::{ContainerProps, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::tree::UiTree;
use fret_workspace::commands::{
    pane_activate_command, pane_move_active_tab_to_command, tab_activate_command,
};
use fret_workspace::layout::{WorkspacePaneLayout, WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{
    DRAG_KIND_WORKSPACE_TAB, WorkspaceTabStrip, workspace_pane_tree_element_with_resize,
};

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
        "workspace-pane-internal-drag-route",
        move |cx| {
            cx.observe_model(&window_layout, fret_ui::Invalidation::Layout);

            let mut render_pane = |cx: &mut fret_ui::ElementContext<'_, App>,
                                   _pane: &WorkspacePaneLayout,
                                   _is_active: bool,
                                   _tab_drag| {
                cx.container(
                    ContainerProps {
                        layout: fill_layout(),
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            };

            vec![workspace_pane_tree_element_with_resize(
                cx,
                window_layout.clone(),
                &mut render_pane,
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(app, services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

fn render_frame_with_tabs(
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
        "workspace-pane-internal-drag-route",
        move |cx| {
            cx.observe_model(&window_layout, fret_ui::Invalidation::Layout);

            let mut render_pane =
                |cx: &mut fret_ui::ElementContext<'_, App>,
                 pane: &WorkspacePaneLayout,
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

                    let content = cx.container(
                        ContainerProps {
                            layout: fill_layout(),
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );

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

            vec![workspace_pane_tree_element_with_resize(
                cx,
                window_layout.clone(),
                &mut render_pane,
            )]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

fn find_bounds_by_test_id(ui: &UiTree<App>, test_id: &str) -> Rect {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .map(|n| n.bounds)
        .unwrap_or_else(|| panic!("expected semantics node with test_id={test_id}"))
}

fn center(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 / 2.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
    )
}

fn take_dispatched_commands(app: &mut App) -> Vec<CommandId> {
    app.flush_effects()
        .into_iter()
        .filter_map(|e| match e {
            Effect::Command {
                window: Some(_),
                command,
            } => Some(command),
            _ => None,
        })
        .collect()
}

#[test]
fn workspace_pane_tree_installs_workspace_tab_drag_route_anchor() {
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
    let window_layout = app.models_mut().insert(layout);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout,
    );

    assert!(
        fret_ui::internal_drag::route(&app, window, DRAG_KIND_WORKSPACE_TAB).is_some(),
        "expected workspace pane tree root to install a stable route for workspace tab drags"
    );
}

#[test]
fn workspace_root_drop_after_tab_pointer_up_dispatches_split_and_move() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(220.0)),
    );

    let mut layout = WorkspaceWindowLayout::new("window-1", "pane-a");
    layout.pane_tree = WorkspacePaneTree::split(
        fret_core::Axis::Horizontal,
        0.5,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    layout.active_pane = Some(Arc::from("pane-a"));
    {
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a-1"));
        pane.tabs.open_and_activate(Arc::from("doc-a-2"));
        assert!(pane.tabs.activate_str("doc-a-1"));
    }
    {
        let pane = layout.pane_tree.find_pane_mut("pane-b").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-b-1"));
    }
    let window_layout = app.models_mut().insert(layout);

    render_frame_with_tabs(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );
    let _ = take_dispatched_commands(&mut app);

    let source_tab = find_bounds_by_test_id(&ui, "pane-pane-a-tab-doc-a-2");
    let target_pane = find_bounds_by_test_id(&ui, "pane-pane-b-tabstrip");
    let pointer_id = PointerId(0);
    let start = center(source_tab);
    let split_pos = Point::new(
        Px(target_pane.origin.x.0 + target_pane.size.width.0 - 2.0),
        Px(target_pane.origin.y.0 + target_pane.size.height.0 + 16.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_tick_id(TickId(1));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: split_pos,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let drag = app
        .drag_mut(pointer_id)
        .expect("expected workspace tab drag to start after threshold");
    assert_eq!(drag.kind, DRAG_KIND_WORKSPACE_TAB);
    drag.current_window = window;
    drag.cross_window_hover = true;
    drag.dragging = true;
    drag.position = split_pos;

    render_frame_with_tabs(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        window_layout.clone(),
    );
    let _ = take_dispatched_commands(&mut app);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id,
            position: split_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            pointer_id,
            position: split_pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let cmds = take_dispatched_commands(&mut app);
    let cmd_strings: Vec<&str> = cmds.iter().map(|cmd| cmd.as_str()).collect();
    assert!(
        cmd_strings.contains(&pane_activate_command("pane-b").unwrap().as_str()),
        "expected drop to activate the split target pane, got {cmd_strings:?}"
    );
    assert!(
        cmd_strings
            .iter()
            .any(|cmd| cmd.starts_with("workspace.pane.split.horizontal.second.window-1.pane.")),
        "expected right-edge drop to dispatch a horizontal second split, got {cmd_strings:?}"
    );
    assert!(
        cmd_strings.contains(&pane_activate_command("pane-a").unwrap().as_str()),
        "expected drop to reactivate the source pane before moving the tab, got {cmd_strings:?}"
    );
    assert!(
        cmd_strings.contains(&tab_activate_command("doc-a-2").unwrap().as_str()),
        "expected drop to activate the dragged tab before moving it, got {cmd_strings:?}"
    );
    assert!(
        cmd_strings.iter().any(|cmd| {
            cmd.starts_with(
                pane_move_active_tab_to_command("window-1.pane.")
                    .unwrap()
                    .as_str(),
            )
        }),
        "expected drop to move the active tab into the generated pane, got {cmd_strings:?}"
    );

    for cmd in &cmds {
        let _ = app.models_mut().update(&window_layout, |layout| {
            layout.apply_command(cmd);
        });
    }
    let layout = app
        .models_mut()
        .read(&window_layout, |layout| layout.clone())
        .unwrap();
    assert!(
        layout.pane_tree.find_pane("window-1.pane.1").is_some(),
        "expected split command to create the generated pane"
    );
    assert!(
        layout
            .pane_tree
            .find_pane("window-1.pane.1")
            .unwrap()
            .tabs
            .tabs()
            .iter()
            .any(|id| id.as_ref() == "doc-a-2"),
        "expected dragged tab to move into the generated pane"
    );
    assert!(
        !layout
            .pane_tree
            .find_pane("pane-a")
            .unwrap()
            .tabs
            .tabs()
            .iter()
            .any(|id| id.as_ref() == "doc-a-2"),
        "expected source pane to no longer contain the moved tab"
    );
    assert!(matches!(
        layout.pane_tree,
        WorkspacePaneTree::Split {
            axis: fret_core::Axis::Horizontal,
            b,
            ..
        } if matches!(*b, WorkspacePaneTree::Split {
            axis: fret_core::Axis::Horizontal,
            ..
        })
    ));
}
