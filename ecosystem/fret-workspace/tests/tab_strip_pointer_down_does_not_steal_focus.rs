use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId};
use fret_core::{SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::CommandId;
use fret_ui::element::{ColumnProps, LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::{WorkspaceTab, WorkspaceTabStrip};
use std::sync::Arc;

#[derive(Default)]
struct FakeServices;

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
        "workspace-tab-strip-focus",
        move |cx| {
            let focus_target = cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Px(Px(80.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("FocusTarget")),
                        test_id: Some(Arc::from("focus-target")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx, _st| vec![],
            );

            let tab_strip = WorkspaceTabStrip::new("a")
                .tabs([WorkspaceTab::new(
                    Arc::from("a"),
                    Arc::from("TabA"),
                    CommandId::from("test.workspace.tab.activate"),
                )
                .close_command(CommandId::from("test.workspace.tab.close"))])
                .into_element(cx);

            let mut col = ColumnProps::default();
            col.layout.size.width = Length::Fill;
            col.layout.size.height = Length::Fill;
            vec![cx.column(col, move |_cx| vec![focus_target, tab_strip])]
        },
    );

    ui.set_root(root);
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

fn center(bounds: &Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 / 2.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
    )
}

#[test]
fn tab_strip_right_and_middle_mouse_down_do_not_steal_focus() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(400.0), Px(240.0)),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let (focus_target_node, focus_target_bounds) = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("focus-target"))
        .map(|n| (n.id, n.bounds))
        .expect("focus target node");
    let tab_node = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("TabA"))
        .expect("tab node");

    assert_eq!(ui.focus(), None);
    dispatch_pointer_down(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Left,
        center(&focus_target_bounds),
    );
    assert_eq!(ui.focus(), Some(focus_target_node));

    dispatch_pointer_down(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Right,
        center(&tab_node.bounds),
    );
    assert_eq!(ui.focus(), Some(focus_target_node));

    dispatch_pointer_down(
        &mut ui,
        &mut app,
        &mut services,
        MouseButton::Middle,
        center(&tab_node.bounds),
    );
    assert_eq!(ui.focus(), Some(focus_target_node));
}
