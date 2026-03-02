use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::prefers_reduced_motion;
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
        let theme = Theme::global(&*cx.app).snapshot();

        let mut bg = theme
            .color_by_key("component.skeleton.bg")
            .unwrap_or_else(|| theme.color_token("accent"));

        let reduced_motion = prefers_reduced_motion(cx, Invalidation::Paint, false);
        let animate_pulse = self.animate_pulse && !reduced_motion;

        let mut alpha_mul = if self.secondary { 0.5 } else { 1.0 };
        if animate_pulse {
            // Approximate a 2s pulse cycle without storing state.
            let t = cx.app.frame_id().0 as f32;
            let phase = t * 0.12;
            let v = 0.75 + 0.25 * phase.sin(); // [0.5, 1.0]
            alpha_mul *= v;
        }
        scheduling::set_continuous_frames(cx, animate_pulse);
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::FrameId;
    use fret_ui::element::{ElementKind, Length};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    fn find_bg_alpha(el: &AnyElement) -> Option<f32> {
        let ElementKind::Container(props) = &el.kind else {
            return None;
        };
        props.background.map(|c| c.a)
    }

    #[test]
    fn skeleton_defaults_to_w_full_and_nonzero_height() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Skeleton::new().into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected Skeleton to build a Container element");
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        match props.layout.size.height {
            Length::Px(px) => assert!(px.0 > 0.0, "expected non-zero Skeleton height"),
            other => panic!("expected Skeleton height to be Px, got {other:?}"),
        }
    }

    #[test]
    fn skeleton_pulse_changes_background_alpha_across_frames() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_frame_id(FrameId(0));
        let a0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Skeleton::new().into_element(cx)
        });
        let a0 = find_bg_alpha(&a0).expect("background alpha");

        app.set_frame_id(FrameId(10));
        let a10 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Skeleton::new().into_element(cx)
        });
        let a10 = find_bg_alpha(&a10).expect("background alpha");

        assert_ne!(
            a0, a10,
            "expected animated Skeleton to modulate background alpha across frames"
        );
    }

    #[test]
    fn skeleton_without_pulse_is_stable_across_frames() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_frame_id(FrameId(0));
        let a0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Skeleton::new().animate_pulse(false).into_element(cx)
        });
        let a0 = find_bg_alpha(&a0).expect("background alpha");

        app.set_frame_id(FrameId(10));
        let a10 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Skeleton::new().animate_pulse(false).into_element(cx)
        });
        let a10 = find_bg_alpha(&a10).expect("background alpha");

        assert_eq!(
            a0, a10,
            "expected Skeleton(animate_pulse=false) to keep background alpha stable"
        );
    }
}
