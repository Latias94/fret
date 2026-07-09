use fret_core::geometry::{Edges, Px, Rect};
use fret_core::scene::{Color, Paint};
use fret_core::{RenderTargetId, SemanticsRole, ViewportFit, ViewportMapping};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsDecoration};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::viewport_surface::{
    ViewportSurfacePanelProps, viewport_surface_panel,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plot3dViewport {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub opacity: f32,
}

impl Plot3dViewport {
    pub fn mapping(self, bounds: Rect) -> ViewportMapping {
        ViewportMapping {
            content_rect: bounds,
            target_px_size: self.target_px_size,
            fit: self.fit,
        }
    }

    pub fn draw_rect(self, bounds: Rect) -> Rect {
        self.mapping(bounds).map().draw_rect
    }
}

impl Default for Plot3dViewport {
    fn default() -> Self {
        Self {
            target: RenderTargetId::default(),
            target_px_size: (1, 1),
            fit: ViewportFit::Contain,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Plot3dModel {
    pub viewport: Plot3dViewport,
}

#[derive(Debug, Clone, Copy)]
pub struct Plot3dStyle {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Px,
}

impl Default for Plot3dStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            border_width: Px(1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plot3dPanelProps {
    pub model: Model<Plot3dModel>,
    pub style: Plot3dStyle,
    pub forward_input: bool,
}

impl Plot3dPanelProps {
    pub fn new(model: Model<Plot3dModel>) -> Self {
        Self {
            model,
            style: Plot3dStyle::default(),
            forward_input: true,
        }
    }

    pub fn style(mut self, style: Plot3dStyle) -> Self {
        self.style = style;
        self
    }

    pub fn forward_input(mut self, forward_input: bool) -> Self {
        self.forward_input = forward_input;
        self
    }
}

pub fn plot3d_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: Plot3dPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);

    let viewport = props
        .model
        .read(cx.app, |_app, model| model.viewport)
        .unwrap_or_default();
    let theme = cx.theme();
    let background = props
        .style
        .background
        .unwrap_or_else(|| theme.color_token("card"));
    let border = props
        .style
        .border
        .unwrap_or_else(|| theme.color_token("border"));

    let mut panel = ContainerProps::default();
    panel.layout.size.width = Length::Fill;
    panel.layout.size.height = Length::Fill;
    panel.background = Some(background);
    panel.border = Edges::all(props.style.border_width);
    panel.border_paint = Some(Paint::Solid(border));

    cx.container(panel, |cx| {
        let surface = ViewportSurfacePanelProps {
            target: viewport.target,
            target_px_size: viewport.target_px_size,
            fit: viewport.fit,
            opacity: viewport.opacity,
            forward_input: props.forward_input,
        };
        vec![
            viewport_surface_panel(cx, surface).attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Viewport)
                    .label("Plot3D"),
            ),
        ]
    })
}

pub fn plot3d_panel_with_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Plot3dModel>,
) -> AnyElement {
    plot3d_panel(cx, Plot3dPanelProps::new(model))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::scene::SceneOp;
    use fret_core::{
        AppWindowId, FrameId, PathConstraints, PathId, PathMetrics, PathService, Point, Px, Rect,
        Scene, Size, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_ui::UiTree;

    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
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

    #[test]
    fn plot3d_public_surface_stays_declarative_only() {
        assert!(LIB_RS.contains("pub mod declarative;"));
        assert!(LIB_RS.contains("pub use binding::Plot3dPanelBinding;"));
        assert!(!LIB_RS.contains("pub mod retained;"));
        assert!(!LIB_RS.contains("Plot3dCanvas"));
        assert!(!CARGO_TOML.contains("unstable-retained-bridge"));
    }

    #[test]
    fn plot3d_declarative_panel_paints_viewport_surface() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let target = fret_core::RenderTargetId::default();
        let mut app = App::new();
        app.set_frame_id(FrameId(1));
        let model = app.models_mut().insert(Plot3dModel {
            viewport: Plot3dViewport {
                target,
                target_px_size: (640, 360),
                fit: fret_core::ViewportFit::Contain,
                opacity: 0.5,
            },
        });
        let mut ui = UiTree::<App>::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "plot3d-test",
            |cx| vec![plot3d_panel_with_model(cx, model.clone())],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let viewport = scene.ops().iter().find_map(|op| match op {
            SceneOp::ViewportSurface {
                rect,
                target: op_target,
                opacity,
                ..
            } => Some((*rect, *op_target, *opacity)),
            _ => None,
        });

        let (rect, op_target, opacity) =
            viewport.expect("plot3d panel should emit a viewport surface scene op");
        assert_eq!(op_target, target);
        assert_eq!(opacity, 0.5);
        assert!(rect.size.width.0 > 0.0, "viewport width should be non-zero");
        assert!(
            rect.size.height.0 > 0.0,
            "viewport height should be non-zero"
        );
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Quad { .. })),
            "plot3d panel should preserve retained background chrome"
        );
    }
}
