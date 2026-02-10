use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_runtime::Model;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, OverlayController};
use std::sync::Arc;

use fret_ui_shadcn as shadcn;

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
    fractions: Model<Vec<f32>>,
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
        "resizable-panel-group-layout",
        move |cx| {
            let leaf = cx.container(
                ContainerProps {
                    layout: LayoutStyle::default(),
                    ..Default::default()
                },
                |_cx| [],
            );

            let group = shadcn::ResizablePanelGroup::new(fractions)
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
                .entries([
                    shadcn::ResizablePanel::new([leaf]).into(),
                    shadcn::ResizableHandle::new().into(),
                    shadcn::ResizablePanel::new([]).into(),
                ])
                .into_element(cx);

            let group = cx.semantics(
                SemanticsProps {
                    test_id: Some(Arc::<str>::from("resizable-panel-group")),
                    ..Default::default()
                },
                move |_cx| [group],
            );

            vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(600.0));
                        layout
                    },
                    ..Default::default()
                },
                move |_cx| [group],
            )]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn resizable_panel_group_respects_caller_height() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let fractions = app.models_mut().insert(vec![0.5, 0.5]);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(600.0)),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds, fractions);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("resizable-panel-group"))
        .expect("resizable-panel-group node");

    let height = node.bounds.size.height.0;
    assert!(
        (height - 320.0).abs() <= 0.5,
        "expected height ~= 320px, got {height}px"
    );
}
