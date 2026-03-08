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
    use_resolved_passive_text: bool,
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
            use_resolved_passive_text: false,
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

    /// Resolve base/overlay typography from the passive-text cascade of the surrounding subtree.
    ///
    /// This keeps the legacy explicit `.text_style(...)` path intact, but allows semantic call
    /// sites (e.g. streaming card descriptions) to reuse inherited text-style / foreground scopes.
    pub fn use_resolved_passive_text(mut self) -> Self {
        self.use_resolved_passive_text = true;
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

        let use_resolved_passive_text = self.use_resolved_passive_text;
        let explicit_style = self.text_style;
        let legacy_style = match explicit_style.clone() {
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
        let legacy_style = typography::as_control_text(legacy_style);

        cx.semantics(
            SemanticsProps {
                layout,
                role,
                test_id,
                ..Default::default()
            },
            move |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let base = if use_resolved_passive_text {
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::clone(&text),
                        style: explicit_style.clone(),
                        color: None,
                        wrap,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: fret_ui::element::TextInkOverflow::None,
                    })
                } else {
                    let base_color = resolve_muted_foreground(&theme);
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::clone(&text),
                        style: Some(legacy_style.clone()),
                        color: Some(base_color),
                        wrap,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: fret_ui::element::TextInkOverflow::None,
                    })
                };

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

                        let style_for_paint = if use_resolved_passive_text {
                            painter.resolved_passive_text_style(explicit_style.clone())
                        } else {
                            legacy_style.clone()
                        };

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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Color, FontWeight, MaterialDescriptor, MaterialId, MaterialRegistrationError,
        MaterialService, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Rect, Scene, SceneOp, Size, SvgId, SvgService, TextBlobId, TextLineHeightPolicy,
        TextService, TextStyleRefinement,
    };
    use fret_ui::UiTree;
    use fret_ui::declarative::render_root;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum CallPhase {
        Layout,
        Paint,
    }

    #[derive(Debug, Clone)]
    struct RecordedTextCall {
        phase: CallPhase,
        text: Arc<str>,
        style: TextStyle,
        constraints: TextConstraints,
        metrics: fret_core::TextMetrics,
    }

    #[derive(Default)]
    struct RecordingServices {
        phase: Option<CallPhase>,
        measured: Vec<RecordedTextCall>,
        prepared: Vec<RecordedTextCall>,
    }

    impl RecordingServices {
        fn set_phase(&mut self, phase: CallPhase) {
            self.phase = Some(phase);
        }

        fn current_phase(&self) -> CallPhase {
            self.phase
                .expect("recording phase must be set before layout/paint")
        }

        fn unpack_input(input: &TextInput) -> (Arc<str>, TextStyle) {
            match input {
                TextInput::Plain { text, style } => (Arc::clone(text), style.clone()),
                TextInput::Attributed { text, base, .. } => (Arc::clone(text), base.clone()),
                _ => panic!("unsupported non-exhaustive TextInput variant in shimmer test"),
            }
        }

        fn synthetic_metrics(
            text: &str,
            style: &TextStyle,
            constraints: TextConstraints,
        ) -> fret_core::TextMetrics {
            let line_height = style
                .line_height
                .or_else(|| style.line_height_em.map(|em| Px(style.size.0 * em)))
                .unwrap_or(Px(style.size.0 * 1.4));
            let approx_glyph_width = (style.size.0 * 0.55).max(1.0);
            let natural_width = text.chars().count() as f32 * approx_glyph_width;
            let wrap_bias = match constraints.wrap {
                TextWrap::None => 0.0,
                TextWrap::Word => 0.11,
                TextWrap::Balance => 0.21,
                TextWrap::WordBreak => 0.31,
                TextWrap::Grapheme => 0.41,
            };
            let max_width = constraints.max_width.map(|w| w.0.max(1.0));
            let lines = match (constraints.wrap, max_width) {
                (TextWrap::None, _) => 1.0,
                (_, Some(limit)) => (natural_width / limit).ceil().max(1.0),
                (_, None) => 1.0,
            };
            let width = max_width
                .map(|limit| natural_width.min(limit))
                .unwrap_or(natural_width)
                .max(1.0);
            let baseline = Px((line_height.0 * 0.7)
                + style.weight.0 as f32 * 0.001
                + max_width.unwrap_or(0.0) * 0.002
                + wrap_bias);
            fret_core::TextMetrics {
                size: Size::new(Px(width), Px((line_height.0 * lines).max(line_height.0))),
                baseline,
            }
        }

        fn find_call(
            &self,
            phase: CallPhase,
            text: &str,
            expected_constraints: TextConstraints,
        ) -> &RecordedTextCall {
            self.measured
                .iter()
                .find(|call| {
                    call.phase == phase
                        && call.text.as_ref() == text
                        && call.constraints == expected_constraints
                })
                .expect("expected matching text call")
        }
    }

    impl TextService for RecordingServices {
        fn prepare(
            &mut self,
            input: &TextInput,
            constraints: TextConstraints,
        ) -> (TextBlobId, fret_core::TextMetrics) {
            let (text, style) = Self::unpack_input(input);
            let metrics = Self::synthetic_metrics(text.as_ref(), &style, constraints);
            self.prepared.push(RecordedTextCall {
                phase: self.current_phase(),
                text,
                style,
                constraints,
                metrics,
            });
            (TextBlobId::default(), metrics)
        }

        fn measure(
            &mut self,
            input: &TextInput,
            constraints: TextConstraints,
        ) -> fret_core::TextMetrics {
            let (text, style) = Self::unpack_input(input);
            let metrics = Self::synthetic_metrics(text.as_ref(), &style, constraints);
            self.measured.push(RecordedTextCall {
                phase: self.current_phase(),
                text,
                style,
                constraints,
                metrics,
            });
            metrics
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for RecordingServices {
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

    impl SvgService for RecordingServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl MaterialService for RecordingServices {
        fn register_material(
            &mut self,
            _desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Err(MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            false
        }
    }

    #[test]
    fn shimmer_resolved_mode_keeps_wrap_overflow_and_baseline_aligned() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let text: Arc<str> = Arc::<str>::from("wrapped shimmer parity gate");
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(140.0)),
        );
        let expected_constraints = TextConstraints {
            max_width: Some(Px(64.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let inherited = TextStyleRefinement {
            font: Some(FontId::ui()),
            size: Some(Px(17.0)),
            weight: Some(FontWeight::SEMIBOLD),
            line_height: Some(Px(25.0)),
            line_height_policy: Some(TextLineHeightPolicy::FixedFromStyle),
            ..Default::default()
        };
        let base_color = Color {
            r: 0.35,
            g: 0.42,
            b: 0.58,
            a: 1.0,
        };
        let overlay_color = Color {
            r: 0.96,
            g: 0.97,
            b: 0.98,
            a: 1.0,
        };
        let mut services = RecordingServices::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shimmer-resolved-wrap-overflow-baseline",
            |cx| {
                let mut group = fret_ui::element::ContainerProps::default();
                group.layout.size.width = fret_ui::element::Length::Px(Px(64.0));

                let mut text_props = fret_ui::element::TextProps::new(Arc::clone(&text));
                text_props.layout.size.width = fret_ui::element::Length::Px(Px(64.0));
                text_props.wrap = TextWrap::Word;
                text_props.overflow = TextOverflow::Clip;

                let theme = Theme::global(&*cx.app).clone();
                let canvas_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .absolute()
                        .inset_px(Px(0.0))
                        .size_full(),
                );
                let overlay_text = Arc::clone(&text);

                vec![
                    cx.container(group, move |cx| {
                        let overlay_text = Arc::clone(&overlay_text);
                        vec![
                            cx.text_props(text_props.clone()),
                            cx.canvas(
                                CanvasProps {
                                    layout: canvas_layout,
                                    cache_policy: Default::default(),
                                },
                                move |painter| {
                                    let bounds = painter.bounds();
                                    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                                        return;
                                    }

                                    let style_for_paint = painter.resolved_passive_text_style(None);
                                    let constraints = TextConstraints {
                                        max_width: Some(bounds.size.width),
                                        wrap: TextWrap::Word,
                                        overflow: TextOverflow::Clip,
                                        align: fret_core::TextAlign::Start,
                                        scale_factor: painter.scale_factor(),
                                    };
                                    let baseline = {
                                        let (services, _scene) = painter.services_and_scene();
                                        let input = TextInput::plain(
                                            Arc::clone(&overlay_text),
                                            style_for_paint.clone(),
                                        );
                                        services.text().measure(&input, constraints).baseline
                                    };
                                    let origin = Point::new(
                                        bounds.origin.x,
                                        Px(bounds.origin.y.0 + baseline.0),
                                    );
                                    let raster_scale_factor = painter.scale_factor();

                                    painter.with_clip_rect(bounds, |p| {
                                        p.shared_text(
                                            DrawOrder(0),
                                            origin,
                                            Arc::clone(&overlay_text),
                                            style_for_paint.clone(),
                                            overlay_color,
                                            CanvasTextConstraints {
                                                max_width: Some(bounds.size.width),
                                                wrap: TextWrap::Word,
                                                overflow: TextOverflow::Clip,
                                            },
                                            raster_scale_factor,
                                        );
                                    });
                                },
                            ),
                        ]
                    })
                    .inherit_text_style(inherited.clone())
                    .inherit_foreground(base_color),
                ]
            },
        );
        ui.set_root(root);

        services.set_phase(CallPhase::Layout);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        services.set_phase(CallPhase::Paint);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let layout_measure =
            services.find_call(CallPhase::Layout, text.as_ref(), expected_constraints);
        let paint_measure =
            services.find_call(CallPhase::Paint, text.as_ref(), expected_constraints);

        let layout_prepare = services
            .prepared
            .iter()
            .find(|record| {
                record.phase == CallPhase::Layout
                    && record.text.as_ref() == text.as_ref()
                    && record.constraints == expected_constraints
            })
            .expect("expected base text prepare to use the wrapped constraints");
        let paint_prepare = services
            .prepared
            .iter()
            .find(|record| {
                record.phase == CallPhase::Paint
                    && record.text.as_ref() == text.as_ref()
                    && record.constraints == expected_constraints
            })
            .expect("expected overlay shared_text prepare to use the wrapped constraints");

        assert_eq!(layout_measure.style, layout_prepare.style);
        assert_eq!(paint_measure.style, layout_prepare.style);
        assert_eq!(paint_prepare.style, layout_prepare.style);
        assert_eq!(layout_measure.constraints, expected_constraints);
        assert_eq!(paint_measure.constraints, expected_constraints);
        assert_eq!(layout_prepare.constraints, expected_constraints);
        assert_eq!(paint_prepare.constraints, expected_constraints);

        let mut base_origin = None;
        let mut overlay_origin = None;
        for op in scene.ops() {
            if let SceneOp::Text { origin, paint, .. } = op {
                match paint.paint {
                    fret_core::scene::Paint::Solid(color) if color == base_color => {
                        base_origin = Some(*origin);
                    }
                    fret_core::scene::Paint::Solid(color) if color == overlay_color => {
                        overlay_origin = Some(*origin);
                    }
                    _ => {}
                }
            }
        }

        let base_origin = base_origin.expect("expected base text scene op");
        let overlay_origin = overlay_origin.expect("expected overlay text scene op");
        assert!(
            (base_origin.y.0 - overlay_origin.y.0).abs() < 0.01,
            "expected base and overlay baselines to stay aligned; base={base_origin:?} overlay={overlay_origin:?}",
        );
        assert!(
            (overlay_origin.y.0 - paint_measure.metrics.baseline.0).abs() < 0.01,
            "expected overlay origin to follow the measured baseline; origin={overlay_origin:?} baseline={:?}",
            paint_measure.metrics.baseline,
        );
    }
}
