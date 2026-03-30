use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{CommandId, Effect};
use fret_ui::tree::UiTree;
use fret_workspace::WorkspaceTabStrip;
use fret_workspace::commands::tab_activate_command;
use fret_workspace::tabs::WorkspaceTabs;
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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    tabs: &WorkspaceTabs,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    let tabs = tabs.clone();
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "workspace-tab-strip-keyboard-roving",
        move |cx| {
            let strip = WorkspaceTabStrip::from_workspace_tabs(&tabs, |id| Arc::<str>::from(id))
                .test_id_root("tabstrip")
                .tab_test_id_prefix("tabstrip-tab")
                .into_element(cx);
            vec![strip]
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
fn arrow_right_moves_focus_and_auto_activates_next_tab() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(120.0)),
    );

    let mut tabs = WorkspaceTabs::new();
    tabs.open_and_activate(Arc::from("a"));
    tabs.open_and_activate(Arc::from("b"));
    tabs.open_and_activate(Arc::from("c"));
    assert!(tabs.activate(Arc::from("a")));

    render_frame(&mut ui, &mut app, &mut services, window, bounds, &tabs);

    let a = find_node_by_test_id(&ui, "tabstrip-tab-a");
    let b = find_node_by_test_id(&ui, "tabstrip-tab-b");

    ui.set_focus(Some(a));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(b));

    let cmds = take_dispatched_commands(&mut app);
    assert!(
        cmds.iter()
            .any(|c| c.as_str() == tab_activate_command("b").unwrap().as_str()),
        "expected roving active change to auto-dispatch workspace.tab.activate.b"
    );
}
