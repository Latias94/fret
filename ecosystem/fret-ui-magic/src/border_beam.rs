use fret_core::Color;
use fret_core::geometry::{Corners, Edges, Point, Px, Size};
use fret_core::scene::{
    BlendMode, ColorSpace, EffectChain, EffectMode, EffectStep, GradientStop, MAX_STOPS, Paint,
    RadialGradient, TileMode,
};
use fret_ui::element::{
    AnyElement, ContainerProps, FocusTraversalGateProps, HitTestGateProps, InsetStyle, LayoutStyle,
    Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::reduced_motion_queries;
use fret_ui_kit::declarative::scheduling::set_continuous_frames;

#[derive(Debug, Clone)]
pub struct BorderBeamProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub background: Color,
    pub border: Edges,
    pub border_base: Color,
    pub beam_color: Color,
    pub beam_radius: Px,
    /// Revolutions per second around the rectangle perimeter.
    pub speed_rps: f32,
    /// Glow blur radius in pixels. Set to 0 to disable glow.
    pub glow_blur_radius: Px,
    /// Glow opacity multiplier (applied to `beam_color.a`).
    pub glow_opacity: f32,
}

impl Default for BorderBeamProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            background: Color::TRANSPARENT,
            border: Edges::all(Px(1.0)),
            border_base: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.12,
            },
            beam_color: Color {
                r: 0.95,
                g: 0.95,
                b: 1.0,
                a: 0.8,
            },
            beam_radius: Px(140.0),
            speed_rps: 0.25,
            glow_blur_radius: Px(18.0),
            glow_opacity: 0.55,
        }
    }
}

fn radial(center: Point, radius: Px, inner: Color, outer: Color) -> Paint {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, inner);
    stops[1] = GradientStop::new(1.0, outer);
    Paint::RadialGradient(RadialGradient {
        center,
        radius: Size::new(radius, radius),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 2,
        stops,
    })
}

fn perimeter_point_clockwise(bounds: fret_core::Rect, t01: f32) -> Point {
    let w = bounds.size.width.0.max(0.0);
    let h = bounds.size.height.0.max(0.0);
    let perim = (2.0 * (w + h)).max(1.0);

    let mut d = (t01.rem_euclid(1.0) * perim).clamp(0.0, perim);

    let (x, y) = if d <= w {
        (d, 0.0)
    } else if d <= w + h {
        d -= w;
        (w, d)
    } else if d <= 2.0 * w + h {
        d -= w + h;
        (w - d, h)
    } else {
        d -= 2.0 * w + h;
        (0.0, h - d)
    };

    Point::new(Px(x), Px(y))
}

pub fn border_beam<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: BorderBeamProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let prefers_reduced_motion =
        reduced_motion_queries::prefers_reduced_motion(cx, Invalidation::Paint, false);

    let clock = cx
        .app
        .global::<fret_core::WindowFrameClockService>()
        .and_then(|svc| svc.snapshot(cx.window));

    let can_animate = !prefers_reduced_motion && clock.is_some() && props.speed_rps.is_finite();
    set_continuous_frames(cx, can_animate);
    if can_animate {
        cx.notify_for_animation_frame();
    }

    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Clip,
            ..props.layout
        },
        padding: props.padding,
        corner_radii: props.corner_radii,
        background: Some(props.background),
        border: props.border,
        border_color: Some(props.border_base),
        ..Default::default()
    };

    cx.container(outer, move |cx| {
        let id = cx.root_id();
        let bounds = cx.last_bounds_for_element(id);

        let center = if let (Some(clock), Some(bounds)) = (clock.filter(|_| can_animate), bounds) {
            let seconds = clock.now_monotonic.as_secs_f32();
            let t = (seconds * props.speed_rps).rem_euclid(1.0);
            perimeter_point_clockwise(bounds, t)
        } else if let Some(bounds) = bounds {
            Point::new(
                Px(bounds.size.width.0 * 0.5),
                Px(bounds.size.height.0 * 0.5),
            )
        } else {
            Point::new(Px(0.0), Px(0.0))
        };

        let inner = props.beam_color;
        let outer = Color::TRANSPARENT;
        let crisp_paint = radial(center, props.beam_radius, inner, outer);

        let mut glow_color = inner;
        glow_color.a = (glow_color.a * props.glow_opacity).clamp(0.0, 1.0);
        let glow_paint = radial(center, props.beam_radius, glow_color, Color::TRANSPARENT);

        let overlay_layout = LayoutStyle {
            position: PositionStyle::Absolute,
            inset: InsetStyle {
                top: Some(Px(0.0)),
                right: Some(Px(0.0)),
                bottom: Some(Px(0.0)),
                left: Some(Px(0.0)),
            },
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        };

        let overlay = cx.focus_traversal_gate_props(
            FocusTraversalGateProps {
                layout: overlay_layout,
                traverse: false,
            },
            |cx| {
                let mut hit_layout = LayoutStyle::default();
                hit_layout.size.width = Length::Fill;
                hit_layout.size.height = Length::Fill;

                [cx.hit_test_gate_props(
                    HitTestGateProps {
                        layout: hit_layout,
                        hit_test: false,
                    },
                    |cx| {
                        let mut overlay_fill = LayoutStyle::default();
                        overlay_fill.size.width = Length::Fill;
                        overlay_fill.size.height = Length::Fill;

                        let mut out: Vec<AnyElement> = Vec::new();

                        if props.glow_blur_radius.0 > 0.0 {
                            let chain = EffectChain::from_steps(&[EffectStep::GaussianBlur {
                                radius_px: props.glow_blur_radius,
                                downsample: 1,
                            }]);
                            let glow = cx.composite_group(BlendMode::Add, |cx| {
                                vec![cx.effect_layer(EffectMode::FilterContent, chain, |cx| {
                                    let glow_border = cx.container(
                                        ContainerProps {
                                            layout: overlay_fill,
                                            background: None,
                                            border: props.border,
                                            border_color: None,
                                            border_paint: Some(glow_paint),
                                            corner_radii: props.corner_radii,
                                            ..Default::default()
                                        },
                                        |_| Vec::new(),
                                    );
                                    vec![glow_border]
                                })]
                            });
                            out.push(glow);
                        }

                        // Crisp beam on top (no blur).
                        let crisp_border = cx.container(
                            ContainerProps {
                                layout: overlay_fill,
                                background: None,
                                border: props.border,
                                border_color: None,
                                border_paint: Some(crisp_paint),
                                corner_radii: props.corner_radii,
                                ..Default::default()
                            },
                            |_| Vec::new(),
                        );
                        out.push(crisp_border);

                        out
                    },
                )]
            },
        );

        let mut out: Vec<AnyElement> = children(cx).into_iter().collect();
        out.push(overlay);
        out
    })
}
