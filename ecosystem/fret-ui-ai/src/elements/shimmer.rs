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

fn resolve_background(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("background"))
}

fn resolve_muted_foreground(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
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
/// “erase band” using a clipped `Canvas` that draws the same text in `background` color.
pub struct Shimmer {
    text: Arc<str>,
    duration_secs: f32,
    spread: f32,
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

        let text = Arc::clone(&self.text);
        let duration_secs = self.duration_secs;
        let spread = self.spread;

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
                    style: None,
                    color: Some(base_color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
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

                        let w = bounds.size.width.0.max(0.0);
                        let band_w = (dynamic_spread_px * 2.0).clamp(1.0, w.max(1.0));

                        // Sweep the band from right → left, matching the upstream background-position
                        // animation from `100%` to `0%`.
                        let center = (1.0 - frac) * (w + band_w) - band_w * 0.5;
                        let x0 = (center - band_w * 0.5).clamp(0.0, w);
                        let x1 = (center + band_w * 0.5).clamp(0.0, w);
                        if x1 <= x0 {
                            return;
                        }

                        let clip = Rect::new(
                            Point::new(Px(bounds.origin.x.0 + x0), bounds.origin.y),
                            Size::new(Px(x1 - x0), bounds.size.height),
                        );

                        let highlight_color = resolve_background(painter.theme());

                        let theme = painter.theme();
                        let font_size = theme
                            .metric_by_key("font.size")
                            .unwrap_or(theme.metrics.font_size);
                        let line_height = theme
                            .metric_by_key("font.line_height")
                            .unwrap_or(theme.metrics.font_line_height);
                        let style = TextStyle {
                            font: FontId::default(),
                            size: font_size,
                            line_height: Some(line_height),
                            ..Default::default()
                        };

                        let constraints = TextConstraints {
                            max_width: Some(bounds.size.width),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            scale_factor: painter.scale_factor(),
                        };
                        let baseline = {
                            let (services, _scene) = painter.services_and_scene();
                            let input = TextInput::plain(Arc::clone(&shimmer_text), style.clone());
                            services.text().measure(&input, constraints).baseline
                        };
                        let origin =
                            Point::new(bounds.origin.x, Px(bounds.origin.y.0 + baseline.0));

                        let canvas_constraints = CanvasTextConstraints {
                            max_width: Some(bounds.size.width),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        };
                        let raster_scale_factor = painter.scale_factor();

                        painter.with_clip_rect(clip, |p| {
                            p.shared_text(
                                DrawOrder(0),
                                origin,
                                Arc::clone(&shimmer_text),
                                style,
                                highlight_color,
                                canvas_constraints,
                                raster_scale_factor,
                            );
                        });
                    },
                );

                vec![base, overlay]
            },
        )
    }
}
