use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ColorRef, LayoutRefinement, MetricRef};
use fret_core::Px;
use fret_ui::element::{AnyElement, SpinnerProps};
use fret_ui::{ElementCx, Theme, UiHost};

/// shadcn/ui `Spinner` (v4).
///
/// Upstream uses a spinning lucide icon (`Loader2Icon` + `animate-spin`). Fret does not currently
/// support arbitrary transforms on text/image primitives, so the baseline implementation renders a
/// dot-ring spinner with frame-driven animation.
#[derive(Debug, Clone)]
pub struct Spinner {
    layout: LayoutRefinement,
    color: Option<ColorRef>,
    dot_count: u8,
    speed: f32,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            color: None,
            dot_count: 12,
            speed: 0.2,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn dot_count(mut self, dots: u8) -> Self {
        self.dot_count = dots;
        self
    }

    /// Phase increment per frame, in dot steps. (`0.0` disables animation.)
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default()
            .w_px(MetricRef::Px(Px(16.0)))
            .h_px(MetricRef::Px(Px(16.0)))
            .flex_shrink_0();

        let layout = decl_style::layout_style(&theme, base_layout.merge(self.layout));
        let color = self
            .color
            .map(|c| c.resolve(&theme))
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        cx.spinner_props(SpinnerProps {
            layout,
            color: Some(color),
            dot_count: self.dot_count,
            speed: self.speed,
        })
    }
}
