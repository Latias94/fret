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
    panel.background = Some(background.into());
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
    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");

    #[test]
    fn plot3d_public_surface_stays_declarative_only() {
        assert!(LIB_RS.contains("pub mod declarative;"));
        assert!(!LIB_RS.contains("pub mod retained;"));
        assert!(!LIB_RS.contains("Plot3dCanvas"));
        assert!(!CARGO_TOML.contains("unstable-retained-bridge"));
    }
}
