use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_kit::{MetricRef, Space};
use fret_ui_shadcn::facade as shadcn;
use fret_ui_shadcn::facade::themes as shadcn_themes;
use shadcn::Empty;
use std::sync::Arc;

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

fn find_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
}

fn render_frame_and_measure_inset_x(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    container_width: Px,
    settle_frames: usize,
) -> Px {
    // Container queries depend on committed bounds, which are observed across frames.
    let settle_frames = settle_frames.max(1);
    let mut inset_x = Px(0.0);

    for _ in 0..settle_frames {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "empty", |cx| {
                let child = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(10.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                );

                let child = cx.semantics(
                    SemanticsProps {
                        test_id: Some(Arc::<str>::from("empty.child")),
                        ..Default::default()
                    },
                    move |_cx| [child],
                );

                let empty = Empty::new([child]).into_element(cx);
                let empty = cx.semantics(
                    SemanticsProps {
                        test_id: Some(Arc::<str>::from("empty.root")),
                        ..Default::default()
                    },
                    move |_cx| [empty],
                );

                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(container_width);
                            layout
                        },
                        ..Default::default()
                    },
                    move |_cx| [empty],
                )]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let empty_node = find_test_id(snap, "empty.root").expect("empty root semantics");
        let child_node = find_test_id(snap, "empty.child").expect("empty child semantics");

        inset_x =
            Px(((empty_node.bounds.size.width.0 - child_node.bounds.size.width.0) * 0.5).abs());
    }

    inset_x
}

#[test]
fn empty_padding_switches_at_md_using_container_queries() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let theme = Theme::global(&app).clone();
    let expected_delta = Px(MetricRef::space(Space::N12).resolve(&theme).0
        - MetricRef::space(Space::N6).resolve(&theme).0);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1400.0), Px(400.0)),
    );

    let inset_sm = render_frame_and_measure_inset_x(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        Px(600.0),
        2,
    );
    let inset_md = render_frame_and_measure_inset_x(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        Px(1000.0),
        2,
    );

    let observed_delta = Px(inset_md.0 - inset_sm.0);
    assert!(
        (observed_delta.0 - expected_delta.0).abs() <= 2.0,
        "expected inset delta to match `p-6 -> p-12` token delta; got {observed_delta:?}, expected {expected_delta:?} (sm inset {inset_sm:?}, md inset {inset_md:?})",
    );
}
