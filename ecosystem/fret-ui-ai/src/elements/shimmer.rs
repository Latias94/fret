use std::sync::Arc;

use fret_core::{
    DrawOrder, FontId, Point, Px, Rect, SemanticsRole, Size, TextConstraints, TextInput,
    TextOverflow, TextStyle, TextWrap,
};
use fret_ui::canvas::CanvasTextConstraints;
use fret_ui::element::{AnyElement, CanvasProps, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;

fn color_with_alpha(mut color: fret_core::Color, alpha: f32) -> fret_core::Color {
    color.a = (color.a * alpha).clamp(0.0, 1.0);
    color
}

fn resolve_background(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("background"))
}

fn resolve_muted_foreground(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn clamp_duration_secs(duration_secs: f32) -> f32 {
    if duration_secs.is_finite() && duration_secs > 0.0 {
        duration_secs.max(0.1)
    } else {
        2.0
    }
}

fn clamp_spread(spread: f32) -> f32 {
    if spread.is_finite() && spread > 0.0 {
        spread
    } else {
        2.0
    }
}

#[derive(Clone)]
/// Animated text shimmer aligned with AI Elements `shimmer.tsx`.
///
/// Upstream uses a CSS background-clip trick (muted text + animated “erase” band driven by
/// `duration` and `spread`). In Fret, we render the base text normally and paint an overlaid
/// highlight band using clipped `Canvas` slices that draw the same text in `background` color.
pub struct Shimmer {
    text: Arc<str>,
    duration_secs: f32,
    spread: f32,
    text_style: Option<TextStyle>,
    wrap: TextWrap,
    role: SemanticsRole,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Shimmer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shimmer")
            .field("text_len", &self.text.len())
            .field("duration_secs", &self.duration_secs)
            .field("spread", &self.spread)
            .field("role", &self.role)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Shimmer {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            duration_secs: 2.0,
            spread: 2.0,
            text_style: None,
            wrap: TextWrap::None,
            role: SemanticsRole::Text,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn duration_secs(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Override the text style used for both the base text and the overlaid "erase band".
    ///
    /// This is important for callers that want shimmer to inherit typography from a wrapper
    /// (e.g. shadcn CardTitle/CardDescription in AI Elements).
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Rough mapping of upstream `as` prop: set an accessibility role for the rendered text node.
    pub fn role(mut self, role: SemanticsRole) -> Self {
        self.role = role;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let layout = decl_style::layout_style(&theme, self.layout);
        let role = self.role;
        let test_id = self.test_id;
        let wrap = self.wrap;

        let text = Arc::clone(&self.text);
        let duration_secs = self.duration_secs;
        let spread = self.spread;

        let style = match self.text_style {
            Some(style) => style,
            None => {
                let font_size = theme
                    .metric_by_key("font.size")
                    .unwrap_or(theme.metrics.font_size);
                let line_height = theme
                    .metric_by_key("font.line_height")
                    .unwrap_or(theme.metrics.font_line_height);
                TextStyle {
                    font: FontId::ui(),
                    size: font_size,
                    line_height: Some(line_height),
                    ..Default::default()
                }
            }
        };
        let style = typography::as_control_text(style);
        let style_for_paint = style.clone();

        cx.semantics(
            SemanticsProps {
                layout,
                role,
                test_id,
                ..Default::default()
            },
            move |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let base_color = resolve_muted_foreground(&theme);

                let base = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::clone(&text),
                    style: Some(style.clone()),
                    color: Some(base_color),
                    wrap,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,

                    ink_overflow: fret_ui::element::TextInkOverflow::None,
                });

                let canvas_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .absolute()
                        .inset_px(Px(0.0))
                        .size_full(),
                );

                let shimmer_text = Arc::clone(&text);
                let overlay = cx.canvas(
                    CanvasProps {
                        layout: canvas_layout,
                        cache_policy: Default::default(),
                    },
                    move |painter| {
                        painter.request_animation_frame();

                        let bounds = painter.bounds();
                        if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                            return;
                        }

                        let text_str = shimmer_text.as_ref();
                        if text_str.is_empty() {
                            return;
                        }

                        let duration_secs = clamp_duration_secs(duration_secs);
                        let frames_per_cycle = (duration_secs * 60.0).round().max(1.0) as u64;
                        let frac = (painter.frame_id() % frames_per_cycle) as f32
                            / frames_per_cycle as f32;

                        let spread = clamp_spread(spread);
                        let len_chars = text_str.chars().count() as f32;
                        let dynamic_spread_px = (len_chars * spread).max(1.0);

                        let highlight_color = resolve_background(painter.theme());
                        let w = bounds.size.width.0.max(0.0);
                        let band_half = dynamic_spread_px;

                        // Upstream animates `background-position` from `100%` -> `0%` with
                        // `bg-[length:250%_100%]`. That effectively sweeps the highlight from
                        // slightly left of the glyphs to slightly right of the glyphs.
                        let center = (-0.25 * w) + frac * (1.5 * w);

                        let x0 = (center - band_half).min(center + band_half).max(0.0);
                        let x1 = (center - band_half).max(center + band_half).min(w);
                        if x1 <= x0 {
                            return;
                        }

                        let constraints = TextConstraints {
                            max_width: Some(bounds.size.width),
                            wrap,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            scale_factor: painter.scale_factor(),
                        };
                        let baseline = {
                            let (services, _scene) = painter.services_and_scene();
                            let input = TextInput::plain(
                                Arc::clone(&shimmer_text),
                                style_for_paint.clone(),
                            );
                            services.text().measure(&input, constraints).baseline
                        };
                        let origin =
                            Point::new(bounds.origin.x, Px(bounds.origin.y.0 + baseline.0));

                        let canvas_constraints = CanvasTextConstraints {
                            max_width: Some(bounds.size.width),
                            wrap,
                            overflow: TextOverflow::Clip,
                        };
                        let raster_scale_factor = painter.scale_factor();

                        // Approximate the upstream linear-gradient shimmer band by slicing the
                        // highlight region into a handful of constant-alpha rects. This keeps the
                        // implementation in the component layer (no per-glyph gradient fill yet)
                        // while producing a softer edge than a single hard clip rect.
                        const SLICES_PER_HALF: f32 = 6.0;
                        let slice_w = (band_half / SLICES_PER_HALF).max(1.0);

                        let mut x = x0;
                        while x < x1 {
                            let next = (x + slice_w).min(x1);
                            let mid = (x + next) * 0.5;
                            let alpha = (1.0 - ((mid - center).abs() / band_half)).clamp(0.0, 1.0);
                            if alpha > 0.0 {
                                let clip = Rect::new(
                                    Point::new(Px(bounds.origin.x.0 + x), bounds.origin.y),
                                    Size::new(Px(next - x), bounds.size.height),
                                );
                                painter.with_clip_rect(clip, |p| {
                                    p.shared_text(
                                        DrawOrder(0),
                                        origin,
                                        Arc::clone(&shimmer_text),
                                        style_for_paint.clone(),
                                        color_with_alpha(highlight_color, alpha),
                                        canvas_constraints,
                                        raster_scale_factor,
                                    );
                                });
                            }
                            x = next;
                        }
                    },
                );

                vec![base, overlay]
            },
        )
    }
}
