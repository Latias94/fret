use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use fret_ui_shadcn::shadcn_themes;
use shadcn::{Field, FieldOrientation};

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

fn center_y(rect: Rect) -> f32 {
    rect.origin.y.0 + rect.size.height.0 * 0.5
}

fn render_field_and_measure(
    container_width: Px,
    settle_frames: usize,
) -> (fret_core::Rect, fret_core::Rect) {
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

    // Keep viewport width large so responsive behavior must follow the container query region,
    // not the viewport snapshot (ADR 0231 vs ADR 0232).
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1400.0), Px(240.0)),
    );

    let settle_frames = settle_frames.max(2);
    let mut label_bounds = None;
    let mut control_bounds = None;

    for frame in 1..=settle_frames {
        app.set_frame_id(FrameId(frame as u64));

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "field-responsive-orientation",
            |cx| {
                let fixed = |cx: &mut fret_ui::ElementContext<'_, App>, w: Px, h: Px| {
                    let inner = cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(w);
                                layout.size.height = Length::Px(h);
                                layout
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );
                    cx.container(ContainerProps::default(), move |_cx| vec![inner])
                };

                let label = fixed(cx, Px(120.0), Px(20.0)).test_id("field.label");
                let control = fixed(cx, Px(220.0), Px(44.0)).test_id("field.control");

                let field = Field::new([label, control])
                    .orientation(FieldOrientation::Responsive)
                    .into_element(cx);

                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(container_width);
                            layout.size.height = Length::Px(Px(200.0));
                            layout
                        },
                        ..Default::default()
                    },
                    move |_cx| vec![field],
                )]
            },
        );

        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        label_bounds = Some(
            find_test_id(snap, "field.label")
                .expect("field.label semantics node")
                .bounds,
        );
        control_bounds = Some(
            find_test_id(snap, "field.control")
                .expect("field.control semantics node")
                .bounds,
        );
    }

    (
        label_bounds.expect("label bounds"),
        control_bounds.expect("control bounds"),
    )
}

#[test]
fn field_orientation_responsive_follows_container_md_breakpoint() {
    // Narrow container (< md): vertical stack.
    let (label, control) = render_field_and_measure(Px(600.0), 2);
    assert!(
        control.origin.y.0 > label.origin.y.0 + 10.0,
        "expected responsive field to stack vertically for narrow containers; label={label:?} control={control:?}"
    );
    assert!(
        (control.origin.x.0 - label.origin.x.0).abs() < 2.0,
        "expected stacked children to share x origin; label={label:?} control={control:?}"
    );

    // Wide container (>= md): horizontal row.
    let (label, control) = render_field_and_measure(Px(1000.0), 2);
    assert!(
        control.origin.x.0 > label.origin.x.0 + 10.0,
        "expected responsive field to lay out as a row for wide containers; label={label:?} control={control:?}"
    );
    assert!(
        (center_y(control) - center_y(label)).abs() < 2.0,
        "expected row children to align on the cross axis; label={label:?} control={control:?}"
    );
}
