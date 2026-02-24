use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{MaterialParams, Paint};
use fret_core::{Color, MaterialDescriptor, MaterialId, MaterialKind, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle};
use fret_ui::{ElementContext, Invalidation, UiHost};

use fret_ui_kit::declarative::reduced_motion_queries;
use fret_ui_kit::declarative::scheduling::set_continuous_frames;
use fret_ui_kit::recipes::catalog::VisualCatalog;
use fret_ui_kit::recipes::resolve::{
    DegradationReason, RecipeDegradedEvent, report_recipe_degraded,
};

fn material_id_from_catalog<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    desc: MaterialDescriptor,
) -> Option<MaterialId> {
    cx.app
        .global::<VisualCatalog>()
        .and_then(|cat| cat.materials.get(desc))
}

fn report_degraded_missing_material<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
) {
    report_recipe_degraded(
        cx.app,
        RecipeDegradedEvent {
            label,
            reason: DegradationReason::UnsupportedCapability,
        },
    );
}

fn rgba(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

#[derive(Debug, Clone, Copy)]
pub struct PatternMotionProps {
    /// If true and the runner provides a frame clock snapshot, the pattern will animate by
    /// requesting continuous frames.
    pub enabled: bool,
    /// Pattern scroll speed in pixels per second, in the pattern's local coordinate space.
    pub scroll_px_per_s: (f32, f32),
}

impl Default for PatternMotionProps {
    fn default() -> Self {
        Self {
            enabled: false,
            scroll_px_per_s: (0.0, 0.0),
        }
    }
}

fn resolve_pattern_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    motion: PatternMotionProps,
) -> (f32, (f32, f32)) {
    if !motion.enabled {
        return (0.0, (0.0, 0.0));
    }

    let prefers_reduced_motion =
        reduced_motion_queries::prefers_reduced_motion(cx, Invalidation::Paint, false);
    let clock = cx
        .app
        .global::<fret_core::WindowFrameClockService>()
        .and_then(|svc| svc.snapshot(cx.window));

    let (vx, vy) = motion.scroll_px_per_s;
    // Bootstrap note: the per-window frame clock snapshot is recorded during paint.
    // If we require it to exist before requesting frames, the first mount cannot start animation
    // on otherwise-idle screens (a common case for web perf evidence pages).
    let wants_motion = !prefers_reduced_motion && (vx != 0.0 || vy != 0.0);

    set_continuous_frames(cx, wants_motion);
    if wants_motion {
        cx.notify_for_animation_frame();
    }

    let Some(clock) = clock.filter(|_| wants_motion) else {
        return (0.0, (0.0, 0.0));
    };

    let seconds = clock.now_monotonic.as_secs_f32();
    (seconds, (seconds * vx, seconds * vy))
}

#[derive(Debug, Clone, Copy)]
pub struct DotPatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub dots: Color,
    pub spacing: Px,
    pub radius: Px,
    pub seed: u32,
    pub motion: PatternMotionProps,
}

impl Default for DotPatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            dots: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.16,
            },
            spacing: Px(18.0),
            radius: Px(2.2),
            seed: 0,
            motion: PatternMotionProps::default(),
        }
    }
}

pub fn dot_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: DotPatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.dot_grid";
    let desc = MaterialDescriptor::new(MaterialKind::DotGrid);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding.into(),
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let (t, (ox, oy)) = resolve_pattern_motion(cx, props.motion);

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.dots),
            [
                props.spacing.0,
                props.spacing.0,
                props.radius.0,
                props.seed as f32,
            ],
            [t, 0.0, ox, oy],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding.into(),
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct GridPatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub lines: Color,
    pub spacing: (Px, Px),
    pub line_width: Px,
    pub seed: u32,
    pub motion: PatternMotionProps,
}

impl Default for GridPatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            lines: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.12,
            },
            spacing: (Px(22.0), Px(22.0)),
            line_width: Px(1.0),
            seed: 0,
            motion: PatternMotionProps::default(),
        }
    }
}

pub fn grid_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: GridPatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.grid";
    let desc = MaterialDescriptor::new(MaterialKind::Grid);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding.into(),
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let (t, (ox, oy)) = resolve_pattern_motion(cx, props.motion);

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.lines),
            [
                props.spacing.0.0,
                props.spacing.1.0,
                props.line_width.0,
                props.seed as f32,
            ],
            [t, 0.0, ox, oy],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding.into(),
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct StripePatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub stripes: Color,
    pub spacing: Px,
    pub stripe_width: Px,
    pub angle_radians: f32,
    pub seed: u32,
    pub motion: PatternMotionProps,
}

impl Default for StripePatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            stripes: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.10,
            },
            spacing: Px(20.0),
            stripe_width: Px(6.0),
            angle_radians: std::f32::consts::FRAC_PI_4,
            seed: 0,
            motion: PatternMotionProps::default(),
        }
    }
}

pub fn stripe_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: StripePatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.stripe";
    let desc = MaterialDescriptor::new(MaterialKind::Stripe);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding.into(),
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let (t, (ox, oy)) = resolve_pattern_motion(cx, props.motion);

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.stripes),
            [
                props.spacing.0,
                0.0,
                props.stripe_width.0,
                props.seed as f32,
            ],
            [t, props.angle_radians, ox, oy],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding.into(),
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}
