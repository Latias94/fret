use std::sync::Arc;

use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, MouseButtons, Point, PointerEvent,
    PointerId, PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId};
use fret_core::{SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::{CommandId, TickId};
use fret_ui::tree::UiTree;
use fret_workspace::{DRAG_KIND_WORKSPACE_TAB, WorkspaceTab, WorkspaceTabStrip};

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

    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "workspace-tab-strip-drag-threshold",
        move |cx| {
            vec![
                WorkspaceTabStrip::new("a")
                    .tabs([WorkspaceTab::new(
                        Arc::from("a"),
                        Arc::from("TabA"),
                        CommandId::from("test.workspace.tab.activate"),
                    )])
                    .into_element(cx),
            ]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn center(bounds: &Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 / 2.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
    )
}

#[test]
fn tab_strip_drag_starts_after_activation_threshold() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(400.0), Px(120.0)),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let tab_bounds = snap
        .nodes
        .iter()
        .find(|node| node.role == SemanticsRole::Tab && node.label.as_deref() == Some("TabA"))
        .map(|node| node.bounds)
        .expect("tab bounds");

    let pointer_id = PointerId(0);
    let start = center(&tab_bounds);

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

    assert!(
        ui.captured_for(pointer_id).is_some(),
        "tab pointer down should capture the pointer for pending drag tracking"
    );
    assert!(
        app.drag(pointer_id).is_none(),
        "pending tab press should not start drag before threshold"
    );

    let under_threshold = Point::new(Px(start.x.0 + 5.0), start.y);
    app.set_tick_id(TickId(1));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: under_threshold,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    assert!(
        app.drag(pointer_id).is_none(),
        "tab drag should stay pending below the activation threshold"
    );

    let activate = Point::new(Px(start.x.0 + 6.0), start.y);
    app.set_tick_id(TickId(2));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: activate,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let drag = app
        .drag(pointer_id)
        .expect("expected tab drag session after activation");
    assert_eq!(drag.kind, DRAG_KIND_WORKSPACE_TAB);
    assert!(drag.dragging, "expected tab drag session to be active");
}
