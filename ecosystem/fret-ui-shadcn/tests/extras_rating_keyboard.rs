use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerType,
    Px, Rect, Size as CoreSize,
};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(240.0)),
    )
}

fn render_frame<I, F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    render: F,
) where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    app.set_frame_id(frame_id);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "extras", render);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_key_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
}

fn dispatch_key_up(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}

fn dispatch_key_press(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    key: KeyCode,
) {
    dispatch_key_down(ui, app, services, key);
    dispatch_key_up(ui, app, services, key);
}

fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

fn click_center(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    center: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn find_semantics_by_label<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: fret_core::SemanticsRole,
    label: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

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
                size: CoreSize::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
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

fn get_rating(app: &mut App, model: &Model<u8>) -> u8 {
    app.models_mut().get_cloned(model).unwrap_or(0)
}

#[test]
fn rating_arrow_keys_update_value() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let rating = app.models_mut().insert(0u8);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let render = |cx: &mut ElementContext<'_, App>| {
        vec![
            fret_ui_shadcn::extras::Rating::new(rating.clone())
                .count(5)
                .into_element(cx),
        ]
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        render,
    );

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let first = find_semantics_by_label(&snap, fret_core::SemanticsRole::RadioButton, "1 star");
    click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(first.bounds),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        render,
    );
    assert_eq!(get_rating(&mut app, &rating), 1);

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3),
        render,
    );
    assert_eq!(get_rating(&mut app, &rating), 2);

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowLeft);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4),
        render,
    );
    assert_eq!(get_rating(&mut app, &rating), 1);
}
