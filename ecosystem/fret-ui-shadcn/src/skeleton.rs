use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

/// shadcn/ui `Skeleton` (v4).
///
/// Upstream defaults:
/// - `bg-accent`
/// - `rounded-md`
/// - `animate-pulse`
///
/// In Fret, animation is implemented by requesting animation frames while the skeleton is
/// rendered, and modulating background alpha as a pure function of `FrameId`.
#[derive(Debug, Clone)]
pub struct Skeleton {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    secondary: bool,
    animate_pulse: bool,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            secondary: false,
            animate_pulse: true,
        }
    }

    /// gpui-component parity: `secondary()` uses reduced opacity.
    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    pub fn animate_pulse(mut self, animate: bool) -> Self {
        self.animate_pulse = animate;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut bg = theme.color_required("accent");

        let mut alpha_mul = if self.secondary { 0.5 } else { 1.0 };
        if self.animate_pulse {
            // Approximate a 2s pulse cycle without storing state.
            let t = cx.app.frame_id().0 as f32;
            let phase = t * 0.12;
            let v = 0.75 + 0.25 * phase.sin(); // [0.5, 1.0]
            alpha_mul *= v;
        }
        scheduling::set_continuous_frames(cx, self.animate_pulse);
        bg.a = (bg.a * alpha_mul).clamp(0.0, 1.0);

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .bg(ColorRef::Color(bg))
            .merge(self.chrome);

        // `h-4 w-full` is a common upstream default for skeleton blocks (gpui-component uses it).
        let layout = LayoutRefinement::default()
            .w_full()
            .h_px(MetricRef::space(Space::N4))
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        cx.container(props, |_cx| Vec::new())
    }
}
