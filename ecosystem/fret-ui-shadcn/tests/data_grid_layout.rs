use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

use fret_ui_shadcn as shadcn;

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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "data-grid-layout",
        |cx| {
            let grid =
                shadcn::experimental::DataGridElement::new(["PID", "Name", "State", "CPU%"], 50)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_full()
                            .h_px(Px(320.0)),
                    )
                    .into_element(
                        cx,
                        1,
                        1,
                        |row| row as u64,
                        |_row| shadcn::experimental::DataGridRowState {
                            selected: false,
                            enabled: true,
                            on_click: None,
                        },
                        |cx, row, col| match col {
                            0 => cx.text((1000 + row as u64).to_string()),
                            1 => cx.text(format!("Process {row}")),
                            2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                            _ => cx.text(((row * 7) % 100).to_string()),
                        },
                    );

            let grid = cx.semantics(
                SemanticsProps {
                    test_id: Some(Arc::<str>::from("shadcn-data-grid-root")),
                    ..Default::default()
                },
                move |_cx| [grid],
            );

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(600.0));
                        layout
                    },
                    ..Default::default()
                },
                move |_cx| [grid],
            )]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn data_grid_header_does_not_overlap_body() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(600.0)),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let header = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("shadcn-data-grid-header"))
        .expect("shadcn-data-grid-header node");
    let body = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("shadcn-data-grid-body"))
        .expect("shadcn-data-grid-body node");

    let header_bottom = header.bounds.origin.y.0 + header.bounds.size.height.0;
    let body_top = body.bounds.origin.y.0;

    assert!(
        body_top + 0.5 >= header_bottom,
        "expected body top >= header bottom, got body_top={body_top} header_bottom={header_bottom}"
    );
}
