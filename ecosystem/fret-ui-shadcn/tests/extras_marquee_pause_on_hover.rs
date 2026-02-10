use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp,
    Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::Effect;
use fret_ui::tree::UiTree;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(240.0)),
    )
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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
) -> (Vec<SceneOp>, Vec<Effect>) {
    app.set_frame_id(frame_id);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "extras", |cx| {
            vec![
                fret_ui_shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta"])
                    .speed_px_per_frame(Px(1.0))
                    .pause_on_hover(true)
                    .into_element(cx),
            ]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);

    (scene.ops().to_vec(), app.flush_effects())
}

fn first_non_zero_translate_x(ops: &[SceneOp]) -> Option<f32> {
    ops.iter().find_map(|op| match op {
        SceneOp::PushTransform { transform } if transform.tx != 0.0 => Some(transform.tx),
        _ => None,
    })
}

#[test]
fn marquee_pauses_on_hover() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let (ops1, effects1) =
        render_frame(&mut ui, &mut app, &mut services, window, bounds, FrameId(1));
    assert!(
        effects1
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected first frame to request animation frame; effects={effects1:?}"
    );

    let tx1 = first_non_zero_translate_x(&ops1).expect("expected a non-zero push transform (tx)");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(2.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    let _ = app.flush_effects();

    let (ops2, effects2) =
        render_frame(&mut ui, &mut app, &mut services, window, bounds, FrameId(2));
    assert!(
        !effects2
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected hovered frame to stop requesting animation frames; effects={effects2:?}"
    );

    let tx2 = first_non_zero_translate_x(&ops2).expect("expected a non-zero push transform (tx)");
    assert_eq!(
        tx1, tx2,
        "expected marquee translation to freeze while hovered"
    );
}
