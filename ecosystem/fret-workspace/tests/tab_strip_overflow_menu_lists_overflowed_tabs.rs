use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId};
use fret_core::{SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::CommandId;
use fret_ui::element::{ColumnProps, Length};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_workspace::{WorkspaceTab, WorkspaceTabStrip};
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
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    let changed_models = app.take_changed_models();
    let changed_globals = app.take_changed_globals();
    let _ = fret_ui::frame_pipeline::propagate_changes(ui, app, &changed_models, &changed_globals);

    OverlayController::begin_frame(app, window);

    let mut tabs = Vec::new();
    for i in 0..32 {
        let id: Arc<str> = Arc::from(format!("t{i:02}"));
        let title: Arc<str> = Arc::from(format!("Tab {i:02}"));
        tabs.push(
            WorkspaceTab::new(
                id.clone(),
                title,
                CommandId::from("test.workspace.tab.activate"),
            )
            .close_command(CommandId::new(format!("test.workspace.tab.close.{id}"))),
        );
    }

    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "workspace-tab-strip-overflow-menu",
        move |cx| {
            let tab_strip = WorkspaceTabStrip::new("t00")
                .test_id_root("tabstrip")
                .tabs(tabs)
                .into_element(cx);

            let mut col = ColumnProps::default();
            col.layout.size.width = Length::Fill;
            col.layout.size.height = Length::Fill;
            vec![cx.column(col, move |_cx| vec![tab_strip])]
        },
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_pointer_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    button: MouseButton,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position,
            button,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn dispatch_pointer_up(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    button: MouseButton,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: PointerId(0),
            position,
            button,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn center(bounds: &Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 / 2.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
    )
}

#[cfg(feature = "shadcn-context-menu")]
#[test]
fn tab_strip_overflow_menu_renders_entries() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(240.0), Px(240.0)),
    );

    // Two frames so scroll metrics + element bounds settle.
    render_frame(&mut ui, &mut app, &mut services, window, bounds);
    render_frame(&mut ui, &mut app, &mut services, window, bounds);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let overflow_button = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("tabstrip.overflow_button"))
        .expect("expected overflow button when tabs overflow");

    dispatch_pointer_down(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Left,
        center(&overflow_button.bounds),
    );
    dispatch_pointer_up(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Left,
        center(&overflow_button.bounds),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds);
    let _ = app.flush_effects();
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let overflow_entry_count = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("tabstrip.overflow_entry."))
        })
        .count();

    assert!(
        overflow_entry_count > 0,
        "expected overflow menu to render at least one entry; nodes={}",
        snap.nodes.len()
    );

    let close_button = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Button
                && n.test_id.as_deref().is_some_and(|id| {
                    id.starts_with("tabstrip.overflow_entry.") && id.ends_with(".close")
                })
        })
        .expect("expected overflow menu to expose close buttons");
    let close_test_id = close_button
        .test_id
        .as_deref()
        .expect("close button has test id");
    let tab_id = close_test_id
        .strip_prefix("tabstrip.overflow_entry.")
        .and_then(|rest| rest.strip_suffix(".close"))
        .expect("close button test id encodes tab id");

    let overflow_item_test_id = format!("tabstrip.overflow_entry.{tab_id}");
    let overflow_item = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(overflow_item_test_id.as_str()))
        .expect("expected overflow menu item node for close target");

    let click_pos = center(&close_button.bounds);
    dispatch_pointer_down(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Left,
        click_pos,
    );
    dispatch_pointer_up(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Left,
        click_pos,
    );

    let effects = app.flush_effects();
    let close_cmd = CommandId::new(format!("test.workspace.tab.close.{tab_id}"));
    assert!(
        effects.iter().any(|e| matches!(
            e,
            fret_runtime::Effect::Command { window: Some(w), command }
                if *w == window && *command == close_cmd
        )),
        "expected clicking overflow close to dispatch close command; tab_id={tab_id}; effects={effects:?}; close_bounds={:?}; item_bounds={:?}; click_local_x={}; item_w={}",
        close_button.bounds,
        overflow_item.bounds,
        (click_pos.x.0 - overflow_item.bounds.origin.x.0),
        overflow_item.bounds.size.width.0,
    );
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            fret_runtime::Effect::Command { command, .. }
                if *command == CommandId::from("test.workspace.tab.activate")
        )),
        "expected clicking overflow close to not dispatch activate command; effects={effects:?}"
    );
}
