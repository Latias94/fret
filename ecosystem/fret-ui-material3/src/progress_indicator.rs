//! Material 3 progress indicators (linear + circular).

use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, Size, Transform2D};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, CanvasProps, Length, SemanticsProps};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, UiHost};
use fret_ui_kit::declarative::ElementContextThemeExt as _;

use crate::tokens::progress_indicator as progress_tokens;

#[derive(Debug, Clone)]
enum ProgressValue {
    Determinate(Model<f32>),
    Indeterminate,
}

fn rect_with_size(bounds: Rect, size: Px) -> Rect {
    let w = Px(size.0.min(bounds.size.width.0));
    let h = Px(size.0.min(bounds.size.height.0));
    Rect::new(bounds.origin, Size::new(w, h))
}

fn intersect_rect(a: Rect, b: Rect) -> Option<Rect> {
    let x0 = a.origin.x.0.max(b.origin.x.0);
    let y0 = a.origin.y.0.max(b.origin.y.0);
    let x1 = (a.origin.x.0 + a.size.width.0).min(b.origin.x.0 + b.size.width.0);
    let y1 = (a.origin.y.0 + a.size.height.0).min(b.origin.y.0 + b.size.height.0);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some(Rect::new(
        fret_core::Point::new(Px(x0), Px(y0)),
        Size::new(Px(x1 - x0), Px(y1 - y0)),
    ))
}

fn clip_corners_to_bounds(rect: Rect, clip: Rect, corners: Corners) -> Corners {
    let x0 = rect.origin.x.0;
    let x1 = rect.origin.x.0 + rect.size.width.0;
    let clip_x0 = clip.origin.x.0;
    let clip_x1 = clip.origin.x.0 + clip.size.width.0;

    let mut corners = corners;
    if x0 < clip_x0 {
        corners.top_left = Px(0.0);
        corners.bottom_left = Px(0.0);
    }
    if x1 > clip_x1 {
        corners.top_right = Px(0.0);
        corners.bottom_right = Px(0.0);
    }

    corners
}

fn paint_quad(
    scene: &mut fret_core::Scene,
    order: DrawOrder,
    rect: Rect,
    background: Color,
    corner_radii: Corners,
) {
    scene.push(fret_core::SceneOp::Quad {
        order,
        rect,
        background: fret_core::Paint::Solid(background),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT,
        corner_radii,
    });
}

#[derive(Debug, Clone)]
pub struct LinearProgressIndicator {
    progress: ProgressValue,
    four_color: bool,
    test_id: Option<Arc<str>>,
}

impl LinearProgressIndicator {
    pub fn new(progress: Model<f32>) -> Self {
        Self {
            progress: ProgressValue::Determinate(progress),
            four_color: false,
            test_id: None,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            progress: ProgressValue::Indeterminate,
            four_color: false,
            test_id: None,
        }
    }

    pub fn four_color(mut self, enabled: bool) -> Self {
        self.four_color = enabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (h, track_thickness, active_thickness) = cx.with_theme(|theme| {
            let h = progress_tokens::linear_height(theme);
            let track_thickness = progress_tokens::linear_track_thickness(theme).0.min(h.0);
            let active_thickness = progress_tokens::linear_active_thickness(theme).0.min(h.0);
            (h, track_thickness, active_thickness)
        });

        let is_indeterminate = matches!(&self.progress, ProgressValue::Indeterminate);

        #[derive(Default)]
        struct IndeterminateRuntime {
            start_frame: Option<u64>,
        }

        let now_frame = cx.frame_id.0;
        let indeterminate_start_frame = is_indeterminate
            .then(|| {
                cx.with_state(IndeterminateRuntime::default, |st| {
                    st.start_frame.get_or_insert(now_frame);
                    st.start_frame.unwrap_or(now_frame)
                })
            })
            .unwrap_or(0);
        let indeterminate_frame = now_frame.saturating_sub(indeterminate_start_frame);

        let progress = match &self.progress {
            ProgressValue::Determinate(m) => cx
                .get_model_copied(m, Invalidation::Paint)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0),
            ProgressValue::Indeterminate => 0.0,
        };

        let (track_color, active_color, four_colors, track_shape, active_shape) =
            cx.with_theme(|theme| {
                (
                    progress_tokens::track_color(theme),
                    progress_tokens::active_color(theme),
                    progress_tokens::four_color_palette(theme),
                    progress_tokens::track_shape(theme),
                    progress_tokens::active_shape(theme),
                )
            });

        let mut props = CanvasProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Px(h);

        let four_color = self.four_color;
        let content = cx.canvas(props, move |p| {
            use crate::motion::{cubic_bezier_ease, ms_to_frames};
            use fret_ui::theme::CubicBezier;

            fn sample_segment(
                t: f32,
                t0: f32,
                t1: f32,
                v0: f32,
                v1: f32,
                easing: CubicBezier,
            ) -> f32 {
                if t <= t0 {
                    return v0;
                }
                if t >= t1 {
                    return v1;
                }
                let span = (t1 - t0).max(1e-6);
                let local = (t - t0) / span;
                let e = cubic_bezier_ease(easing, local);
                v0 + (v1 - v0) * e
            }

            fn primary_translate_percent(t: f32) -> f32 {
                let t20 = 0.20;
                let t5915 = 0.5915;
                if t < t20 {
                    return 0.0;
                }
                if t < t5915 {
                    return sample_segment(
                        t,
                        t20,
                        t5915,
                        0.0,
                        83.6714,
                        CubicBezier {
                            x1: 0.5,
                            y1: 0.0,
                            x2: 0.701732,
                            y2: 0.495819,
                        },
                    );
                }
                sample_segment(
                    t,
                    t5915,
                    1.0,
                    83.6714,
                    200.611,
                    CubicBezier {
                        x1: 0.302435,
                        y1: 0.381352,
                        x2: 0.55,
                        y2: 0.956352,
                    },
                )
            }

            fn primary_scale(t: f32) -> f32 {
                let t3665 = 0.3665;
                let t6915 = 0.6915;
                if t < t3665 {
                    return 0.08;
                }
                if t < t6915 {
                    return sample_segment(
                        t,
                        t3665,
                        t6915,
                        0.08,
                        0.661479,
                        CubicBezier {
                            x1: 0.334731,
                            y1: 0.12482,
                            x2: 0.785844,
                            y2: 1.0,
                        },
                    );
                }
                sample_segment(
                    t,
                    t6915,
                    1.0,
                    0.661479,
                    0.08,
                    CubicBezier {
                        x1: 0.06,
                        y1: 0.11,
                        x2: 0.6,
                        y2: 1.0,
                    },
                )
            }

            fn secondary_translate_percent(t: f32) -> f32 {
                let t25 = 0.25;
                let t4835 = 0.4835;
                if t < t25 {
                    return sample_segment(
                        t,
                        0.0,
                        t25,
                        0.0,
                        37.6519,
                        CubicBezier {
                            x1: 0.15,
                            y1: 0.0,
                            x2: 0.515058,
                            y2: 0.409685,
                        },
                    );
                }
                if t < t4835 {
                    return sample_segment(
                        t,
                        t25,
                        t4835,
                        37.6519,
                        84.3862,
                        CubicBezier {
                            x1: 0.31033,
                            y1: 0.284058,
                            x2: 0.8,
                            y2: 0.733712,
                        },
                    );
                }
                sample_segment(
                    t,
                    t4835,
                    1.0,
                    84.3862,
                    160.278,
                    CubicBezier {
                        x1: 0.4,
                        y1: 0.627035,
                        x2: 0.6,
                        y2: 0.902026,
                    },
                )
            }

            fn secondary_scale(t: f32) -> f32 {
                let t1915 = 0.1915;
                let t4415 = 0.4415;
                if t < t1915 {
                    return sample_segment(
                        t,
                        0.0,
                        t1915,
                        0.08,
                        0.457104,
                        CubicBezier {
                            x1: 0.205028,
                            y1: 0.057051,
                            x2: 0.57661,
                            y2: 0.453971,
                        },
                    );
                }
                if t < t4415 {
                    return sample_segment(
                        t,
                        t1915,
                        t4415,
                        0.457104,
                        0.72796,
                        CubicBezier {
                            x1: 0.152313,
                            y1: 0.196432,
                            x2: 0.648374,
                            y2: 1.00432,
                        },
                    );
                }
                sample_segment(
                    t,
                    t4415,
                    1.0,
                    0.72796,
                    0.08,
                    CubicBezier {
                        x1: 0.257759,
                        y1: -0.003163,
                        x2: 0.211762,
                        y2: 1.38179,
                    },
                )
            }

            fn lerp_color(a: Color, b: Color, t: f32) -> Color {
                let t = t.clamp(0.0, 1.0);
                Color {
                    r: a.r + (b.r - a.r) * t,
                    g: a.g + (b.g - a.g) * t,
                    b: a.b + (b.b - a.b) * t,
                    a: a.a + (b.a - a.a) * t,
                }
            }

            fn four_color_at(t: f32, colors: [Color; 4]) -> Color {
                let t = t.rem_euclid(1.0);
                let points = [
                    (0.0, colors[0]),
                    (0.15, colors[0]),
                    (0.25, colors[1]),
                    (0.40, colors[1]),
                    (0.50, colors[2]),
                    (0.65, colors[2]),
                    (0.75, colors[3]),
                    (0.90, colors[3]),
                    (1.0, colors[0]),
                ];

                for i in 0..(points.len() - 1) {
                    let (t0, c0) = points[i];
                    let (t1, c1) = points[i + 1];
                    if t >= t0 && t <= t1 {
                        let span = (t1 - t0).max(1e-6);
                        let u = (t - t0) / span;
                        return lerp_color(c0, c1, u);
                    }
                }

                colors[0]
            }

            let bounds = p.bounds();
            let bounds = rect_with_size(bounds, Px(bounds.size.width.0));
            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                return;
            }

            let track_h = Px(track_thickness);
            let track_y = Px(bounds.origin.y.0 + (bounds.size.height.0 - track_h.0) * 0.5);
            let track = Rect::new(
                fret_core::Point::new(bounds.origin.x, track_y),
                Size::new(bounds.size.width, track_h),
            );
            paint_quad(p.scene(), DrawOrder(0), track, track_color, track_shape);

            let active_h = Px(active_thickness);
            let active_y = Px(bounds.origin.y.0 + (bounds.size.height.0 - active_h.0) * 0.5);

            if is_indeterminate {
                // Indeterminate linear animation alignment:
                // - Source: Material Web (progress/internal/_linear-progress.scss)
                // - Keyframes + constants match MDC's indeterminate linear progress.
                // - We sample those keyframes deterministically using `FrameId` (60Hz).
                if active_color.a <= 0.0 {
                    return;
                }

                let period_frames = ms_to_frames(2000).max(1);
                let t = (indeterminate_frame % period_frames) as f32 / period_frames as f32;

                let color_period_frames = ms_to_frames(4000).max(1);
                let color_t =
                    (indeterminate_frame % color_period_frames) as f32 / color_period_frames as f32;
                let active_color = if four_color {
                    four_color_at(color_t, four_colors)
                } else {
                    active_color
                };

                let primary_inset_percent = -145.167;
                let secondary_inset_percent = -54.8889;

                let primary_x_percent = primary_inset_percent + primary_translate_percent(t);
                let primary_scale = primary_scale(t).max(0.0);

                let secondary_x_percent = secondary_inset_percent + secondary_translate_percent(t);
                let secondary_scale = secondary_scale(t).max(0.0);

                let full_w = bounds.size.width.0;
                let mut paint_bar = |x_percent: f32, scale: f32| {
                    if scale <= 0.0 {
                        return;
                    }

                    let x = bounds.origin.x.0 + full_w * (x_percent / 100.0);
                    let outer = Rect::new(
                        fret_core::Point::new(Px(x), active_y),
                        Size::new(bounds.size.width, active_h),
                    );
                    let inner = Rect::new(
                        outer.origin,
                        Size::new(Px(outer.size.width.0 * scale), outer.size.height),
                    );

                    let Some(visible) = intersect_rect(inner, bounds) else {
                        return;
                    };
                    let corners = clip_corners_to_bounds(inner, bounds, active_shape);
                    paint_quad(p.scene(), DrawOrder(0), visible, active_color, corners);
                };

                paint_bar(primary_x_percent, primary_scale);
                paint_bar(secondary_x_percent, secondary_scale);
                p.request_animation_frame();
                return;
            }

            let w = (bounds.size.width.0 * progress).clamp(0.0, bounds.size.width.0);
            if w <= 0.0 || active_color.a <= 0.0 {
                return;
            }

            let active = Rect::new(
                fret_core::Point::new(bounds.origin.x, active_y),
                Size::new(Px(w), active_h),
            );
            paint_quad(p.scene(), DrawOrder(0), active, active_color, active_shape);
        });

        let Some(test_id) = self.test_id else {
            return content;
        };

        cx.semantics(
            SemanticsProps {
                test_id: Some(test_id),
                focusable: false,
                ..Default::default()
            },
            |_cx| vec![content],
        )
    }
}

#[derive(Debug, Clone)]
pub struct CircularProgressIndicator {
    progress: ProgressValue,
    four_color: bool,
    test_id: Option<Arc<str>>,
}

impl CircularProgressIndicator {
    pub fn new(progress: Model<f32>) -> Self {
        Self {
            progress: ProgressValue::Determinate(progress),
            four_color: false,
            test_id: None,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            progress: ProgressValue::Indeterminate,
            four_color: false,
            test_id: None,
        }
    }

    pub fn four_color(mut self, enabled: bool) -> Self {
        self.four_color = enabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (size, track_thickness, active_thickness) = cx.with_theme(|theme| {
            (
                progress_tokens::circular_size(theme),
                progress_tokens::circular_track_thickness(theme),
                progress_tokens::circular_active_thickness(theme),
            )
        });

        let is_indeterminate = matches!(&self.progress, ProgressValue::Indeterminate);

        #[derive(Default)]
        struct IndeterminateRuntime {
            start_frame: Option<u64>,
        }

        let now_frame = cx.frame_id.0;
        let indeterminate_start_frame = is_indeterminate
            .then(|| {
                cx.with_state(IndeterminateRuntime::default, |st| {
                    st.start_frame.get_or_insert(now_frame);
                    st.start_frame.unwrap_or(now_frame)
                })
            })
            .unwrap_or(0);
        let indeterminate_frame = now_frame.saturating_sub(indeterminate_start_frame);

        let progress = match &self.progress {
            ProgressValue::Determinate(m) => cx
                .get_model_copied(m, Invalidation::Paint)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0),
            ProgressValue::Indeterminate => 0.0,
        };

        let (track_color, active_color, four_colors, track_shape, active_shape) =
            cx.with_theme(|theme| {
                (
                    progress_tokens::track_color(theme),
                    progress_tokens::active_color(theme),
                    progress_tokens::four_color_palette(theme),
                    progress_tokens::track_shape(theme),
                    progress_tokens::active_shape(theme),
                )
            });

        let mut props = CanvasProps::default();
        props.layout.size.width = Length::Px(size);
        props.layout.size.height = Length::Px(size);

        let four_color = self.four_color;
        let content = cx.canvas(props, move |p| {
            use crate::motion::{cubic_bezier_ease, ms_to_frames};
            use fret_ui::theme::CubicBezier;

            fn lerp_color(a: Color, b: Color, t: f32) -> Color {
                let t = t.clamp(0.0, 1.0);
                Color {
                    r: a.r + (b.r - a.r) * t,
                    g: a.g + (b.g - a.g) * t,
                    b: a.b + (b.b - a.b) * t,
                    a: a.a + (b.a - a.a) * t,
                }
            }

            fn four_color_at(t: f32, colors: [Color; 4]) -> Color {
                let t = t.rem_euclid(1.0);
                let points = [
                    (0.0, colors[0]),
                    (0.15, colors[0]),
                    (0.25, colors[1]),
                    (0.40, colors[1]),
                    (0.50, colors[2]),
                    (0.65, colors[2]),
                    (0.75, colors[3]),
                    (0.90, colors[3]),
                    (1.0, colors[0]),
                ];

                for i in 0..(points.len() - 1) {
                    let (t0, c0) = points[i];
                    let (t1, c1) = points[i + 1];
                    if t >= t0 && t <= t1 {
                        let span = (t1 - t0).max(1e-6);
                        let u = (t - t0) / span;
                        return lerp_color(c0, c1, u);
                    }
                }

                colors[0]
            }

            let bounds = p.bounds();
            let bounds = rect_with_size(bounds, size);
            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                return;
            }

            let center = fret_core::Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
            );

            let max_thickness = Px(track_thickness.0.max(active_thickness.0));
            let radius = Px((bounds.size.width.0.min(bounds.size.height.0) * 0.5
                - max_thickness.0 * 0.5)
                .max(0.0));

            let segments: usize = 64;

            if is_indeterminate {
                // Indeterminate circular alignment:
                // - Source: Material Web (progress/internal/_circular-progress.scss)
                // - Approximate the 3 composed animations (expand-arc + rotate-arc + linear-rotate)
                //   with deterministic arc length + rotation derived from `FrameId`.
                if active_color.a <= 0.0 || radius.0 <= 0.0 {
                    return;
                }

                let easing = CubicBezier {
                    x1: 0.4,
                    y1: 0.0,
                    x2: 0.2,
                    y2: 1.0,
                };

                let arc_frames = ms_to_frames(1333).max(1);
                let cycle_frames = (arc_frames * 4).max(1);
                let linear_rotate_frames =
                    ms_to_frames(((1333.0f32 * 360.0) / 306.0).round() as u32).max(1);

                let base_rot = (indeterminate_frame % linear_rotate_frames) as f32
                    / linear_rotate_frames as f32
                    * std::f32::consts::TAU;

                let cycle_t = (indeterminate_frame % cycle_frames) as f32 / cycle_frames as f32;
                let cycle_e = cubic_bezier_ease(easing, cycle_t);
                let arc_rot = cycle_e * (std::f32::consts::TAU * 3.0); // 1080deg

                let arc_t = (indeterminate_frame % arc_frames) as f32 / arc_frames as f32;
                let tri = if arc_t <= 0.5 {
                    arc_t * 2.0
                } else {
                    (1.0 - arc_t) * 2.0
                };
                let arc_e = cubic_bezier_ease(easing, tri);
                let arc_deg = 10.0 + (270.0 - 10.0) * arc_e;
                let arc_segments =
                    (((arc_deg / 360.0) * segments as f32).round() as usize).clamp(1, segments);

                let start_angle = base_rot + arc_rot;
                let start_idx = (((start_angle / std::f32::consts::TAU) * segments as f32).round()
                    as i32)
                    .rem_euclid(segments as i32) as usize;

                let active_color = if four_color {
                    let color_t = (indeterminate_frame % cycle_frames) as f32 / cycle_frames as f32;
                    four_color_at(color_t, four_colors)
                } else {
                    active_color
                };

                let segment_len = Px(((2.0 * std::f32::consts::PI * radius.0) / segments as f32
                    * 0.9)
                    .max(active_thickness.0));

                for n in 0..arc_segments {
                    let i = (start_idx + n) % segments;
                    let theta = (i as f32) / (segments as f32) * std::f32::consts::TAU;

                    let seg = Rect::new(
                        fret_core::Point::new(
                            Px(center.x.0 - segment_len.0 * 0.5),
                            Px(center.y.0 - radius.0 - active_thickness.0 * 0.5),
                        ),
                        Size::new(segment_len, active_thickness),
                    );

                    p.with_transform(Transform2D::rotation_about_radians(theta, center), |p| {
                        paint_quad(p.scene(), DrawOrder(0), seg, active_color, active_shape);
                    });
                }

                p.request_animation_frame();
                return;
            }

            let active_segments = ((segments as f32) * progress).round() as usize;
            let segment_len = Px(
                ((2.0 * std::f32::consts::PI * radius.0) / segments as f32 * 0.9)
                    .max(max_thickness.0),
            );

            for i in 0..segments {
                let theta = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let is_active = i < active_segments;
                let thickness = if is_active {
                    active_thickness
                } else {
                    track_thickness
                };
                let color = if is_active { active_color } else { track_color };
                let corners = if is_active { active_shape } else { track_shape };

                if color.a <= 0.0 {
                    continue;
                }

                let seg = Rect::new(
                    fret_core::Point::new(
                        Px(center.x.0 - segment_len.0 * 0.5),
                        Px(center.y.0 - radius.0 - thickness.0 * 0.5),
                    ),
                    Size::new(segment_len, thickness),
                );

                p.with_transform(Transform2D::rotation_about_radians(theta, center), |p| {
                    paint_quad(p.scene(), DrawOrder(0), seg, color, corners);
                });
            }
        });

        let Some(test_id) = self.test_id else {
            return content;
        };

        cx.semantics(
            SemanticsProps {
                test_id: Some(test_id),
                focusable: false,
                ..Default::default()
            },
            |_cx| vec![content],
        )
    }
}
