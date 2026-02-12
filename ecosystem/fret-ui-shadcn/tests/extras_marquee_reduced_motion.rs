use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Scene, SceneOp, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
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

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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

fn first_translate_x(ops: &[SceneOp]) -> Option<f32> {
    ops.iter().find_map(|op| match op {
        SceneOp::PushTransform { transform } => Some(transform.tx),
        _ => None,
    })
}

#[test]
fn marquee_respects_reduced_motion_and_does_not_request_frames() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    app.with_global_mut(fret_ui::elements::ElementRuntime::new, |rt, _app| {
        rt.set_window_prefers_reduced_motion(window, Some(true));
    });
    app.with_global_mut(fret_ui::ElementRuntime::default, |rt, _app| {
        rt.set_window_prefers_reduced_motion(window, Some(true));
    });

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let (ops1, effects1) =
        render_frame(&mut ui, &mut app, &mut services, window, bounds, FrameId(1));
    assert!(
        !effects1
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected reduced-motion frame to not request animation frames; effects={effects1:?}"
    );

    let (ops2, effects2) =
        render_frame(&mut ui, &mut app, &mut services, window, bounds, FrameId(2));
    assert!(
        !effects2
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected reduced-motion frame to not request animation frames; effects={effects2:?}"
    );

    let tx1 = first_translate_x(&ops1).unwrap_or(0.0);
    let tx2 = first_translate_x(&ops2).unwrap_or(0.0);
    assert!(
        (tx1 - tx2).abs() <= 0.0001,
        "expected marquee translation to remain stable across frames under reduced motion; tx1={tx1} tx2={tx2}"
    );
    assert!(
        tx1.abs() <= 0.0001,
        "expected marquee to start at rest under reduced motion; tx={tx1}"
    );
}
