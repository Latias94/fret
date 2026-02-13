//! Motion helpers for shadcn/Radix-like overlay surfaces.
//!
//! Radix primitives expose `data-state`/`data-side` and transform-origin variables, but they do
//! not prescribe concrete animation durations/easing. shadcn/ui v4 standardizes a small motion
//! taxonomy (fade + zoom + side-based slide) across Popover/Tooltip/HoverCard/etc.
//!
//! This module provides reusable math for those effects so component wrappers don't drift.

use std::time::Duration;

use fret_core::{Edges, Point, Px, Rect, Transform2D};
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, Length, OpacityProps, RenderTransformProps,
    VisualTransformProps,
};
use fret_ui::overlay_placement::Side;
use fret_ui::theme::CubicBezier;
use fret_ui::{ElementContext, Theme, UiHost};

pub const SHADCN_SLIDE_PX: Px = Px(8.0);

pub const SHADCN_MOTION_DURATION_100: Duration = Duration::from_millis(100);
pub const SHADCN_MOTION_DURATION_200: Duration = Duration::from_millis(200);
pub const SHADCN_MOTION_DURATION_300: Duration = Duration::from_millis(300);
pub const SHADCN_MOTION_DURATION_500: Duration = Duration::from_millis(500);

const THEME_DURATION_SHADCN_MOTION_100: &str = "duration.shadcn.motion.100";
const THEME_DURATION_SHADCN_MOTION_200: &str = "duration.shadcn.motion.200";
const THEME_DURATION_SHADCN_MOTION_300: &str = "duration.shadcn.motion.300";
const THEME_DURATION_SHADCN_MOTION_500: &str = "duration.shadcn.motion.500";
const THEME_EASING_SHADCN_MOTION: &str = "easing.shadcn.motion";

const THEME_DURATION_SHADCN_MOTION_OVERLAY_OPEN: &str = "duration.shadcn.motion.overlay.open";
const THEME_DURATION_SHADCN_MOTION_OVERLAY_CLOSE: &str = "duration.shadcn.motion.overlay.close";
const THEME_EASING_SHADCN_MOTION_OVERLAY: &str = "easing.shadcn.motion.overlay";

const THEME_DURATION_SHADCN_MOTION_SIDEBAR_TOGGLE: &str = "duration.shadcn.motion.sidebar.toggle";
const THEME_EASING_SHADCN_MOTION_SIDEBAR: &str = "easing.shadcn.motion.sidebar";

fn theme_duration_ms_by_key<H: UiHost>(cx: &ElementContext<'_, H>, key: &str) -> Option<Duration> {
    let theme = Theme::global(&*cx.app);
    theme
        .duration_ms_by_key(key)
        .map(|ms| Duration::from_millis(ms as u64))
}

fn theme_duration_ms_by_keys<H: UiHost>(
    cx: &ElementContext<'_, H>,
    keys: &[&str],
) -> Option<Duration> {
    for key in keys {
        if let Some(duration) = theme_duration_ms_by_key(cx, key) {
            return Some(duration);
        }
    }
    None
}

/// shadcn overlay duration token (100ms).
pub fn shadcn_motion_duration_100<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_100)
        .unwrap_or(SHADCN_MOTION_DURATION_100)
}

/// shadcn overlay duration token (200ms).
pub fn shadcn_motion_duration_200<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_200)
        .unwrap_or(SHADCN_MOTION_DURATION_200)
}

/// shadcn overlay duration token (300ms).
pub fn shadcn_motion_duration_300<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_300)
        .unwrap_or(SHADCN_MOTION_DURATION_300)
}

/// shadcn overlay duration token (500ms).
pub fn shadcn_motion_duration_500<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_500)
        .unwrap_or(SHADCN_MOTION_DURATION_500)
}

/// shadcn semantic overlay open duration (defaults to `duration.shadcn.motion.200`).
pub fn shadcn_overlay_open_duration<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_keys(
        cx,
        &[
            THEME_DURATION_SHADCN_MOTION_OVERLAY_OPEN,
            THEME_DURATION_SHADCN_MOTION_200,
        ],
    )
    .unwrap_or(SHADCN_MOTION_DURATION_200)
}

/// shadcn semantic overlay close duration (defaults to `duration.shadcn.motion.200`).
pub fn shadcn_overlay_close_duration<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_keys(
        cx,
        &[
            THEME_DURATION_SHADCN_MOTION_OVERLAY_CLOSE,
            THEME_DURATION_SHADCN_MOTION_200,
        ],
    )
    .unwrap_or(SHADCN_MOTION_DURATION_200)
}

/// shadcn overlay cubic-bezier easing curve (`ease-[cubic-bezier(0.22,1,0.36,1)]` by default).
pub fn shadcn_motion_ease_bezier<H: UiHost>(cx: &ElementContext<'_, H>) -> CubicBezier {
    let theme = Theme::global(&*cx.app);
    theme
        .easing_by_key(THEME_EASING_SHADCN_MOTION)
        .unwrap_or(CubicBezier {
            x1: crate::headless::easing::SHADCN_EASE.x1,
            y1: crate::headless::easing::SHADCN_EASE.y1,
            x2: crate::headless::easing::SHADCN_EASE.x2,
            y2: crate::headless::easing::SHADCN_EASE.y2,
        })
}

/// shadcn semantic overlay easing curve (defaults to `easing.shadcn.motion`).
pub fn shadcn_overlay_ease_bezier<H: UiHost>(cx: &ElementContext<'_, H>) -> CubicBezier {
    let theme = Theme::global(&*cx.app);
    theme
        .easing_by_key(THEME_EASING_SHADCN_MOTION_OVERLAY)
        .or_else(|| theme.easing_by_key(THEME_EASING_SHADCN_MOTION))
        .unwrap_or(CubicBezier {
            x1: crate::headless::easing::SHADCN_EASE.x1,
            y1: crate::headless::easing::SHADCN_EASE.y1,
            x2: crate::headless::easing::SHADCN_EASE.x2,
            y2: crate::headless::easing::SHADCN_EASE.y2,
        })
}

/// shadcn semantic sidebar toggle duration (defaults to `duration.shadcn.motion.200`).
pub fn shadcn_sidebar_toggle_duration<H: UiHost>(cx: &ElementContext<'_, H>) -> Duration {
    theme_duration_ms_by_keys(
        cx,
        &[
            THEME_DURATION_SHADCN_MOTION_SIDEBAR_TOGGLE,
            THEME_DURATION_SHADCN_MOTION_200,
        ],
    )
    .unwrap_or(SHADCN_MOTION_DURATION_200)
}

/// shadcn semantic sidebar easing curve (defaults to `easing.shadcn.motion`).
pub fn shadcn_sidebar_ease_bezier<H: UiHost>(cx: &ElementContext<'_, H>) -> CubicBezier {
    let theme = Theme::global(&*cx.app);
    theme
        .easing_by_key(THEME_EASING_SHADCN_MOTION_SIDEBAR)
        .or_else(|| theme.easing_by_key(THEME_EASING_SHADCN_MOTION))
        // shadcn/ui v4 Sidebar uses `ease-linear` for width/position transitions.
        .unwrap_or(CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        })
}

/// shadcn/ui v4 default easing curve (`ease-out`-ish).
pub fn shadcn_ease(x: f32) -> f32 {
    crate::headless::easing::SHADCN_EASE.sample(x)
}

/// CSS `ease-in-out` (`cubic-bezier(0.42,0,0.58,1)`).
pub fn ease_in_out(x: f32) -> f32 {
    crate::headless::easing::EASE_IN_OUT.sample(x)
}

/// CSS `linear`.
pub fn ease_linear(x: f32) -> f32 {
    crate::headless::easing::linear(x)
}

fn fullscreen_motion_layout() -> LayoutStyle {
    // Motion wrappers are commonly used with absolutely positioned overlay content (popper-style
    // placement). Because absolute-positioned children do not contribute to intrinsic sizing, we
    // default to a full-window wrapper to keep hit-testing and focus traversal consistent.
    //
    // Components that need tighter hit-test bounds should avoid relying on the wrapper bounds for
    // input semantics, or use a dedicated hit-testable wrapper element (e.g. a popper wrapper that
    // expands for arrow protrusion).
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

/// Wraps interactive overlay content in opacity + render-transform layers.
///
/// In DOM/CSS, transforms affect pointer targeting (hit-testing follows the visual position). Fret
/// models that explicitly via `RenderTransform`, which participates in hit-testing and pointer
/// coordinate mapping.
///
/// Use this wrapper for overlay motion that should remain interactive while animating. For
/// paint-only transforms (spinners, arrows), prefer `VisualTransform`.
pub fn wrap_opacity_and_render_transform<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity: f32,
    transform: Transform2D,
    children: Vec<AnyElement>,
) -> AnyElement {
    let layout = fullscreen_motion_layout();
    wrap_opacity_and_render_transform_with_layouts(
        cx,
        layout,
        opacity,
        RenderTransformProps { layout, transform },
        children,
    )
}

/// Like [`wrap_opacity_and_render_transform`], but allows gating interactivity.
pub fn wrap_opacity_and_render_transform_gated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity: f32,
    transform: Transform2D,
    interactive: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    let layout = fullscreen_motion_layout();
    wrap_opacity_and_render_transform_with_layouts_gated(
        cx,
        layout,
        opacity,
        RenderTransformProps { layout, transform },
        interactive,
        children,
    )
}

/// Like [`wrap_opacity_and_render_transform`], but allows customizing layouts.
pub fn wrap_opacity_and_render_transform_with_layouts<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity_layout: LayoutStyle,
    opacity: f32,
    transform_props: RenderTransformProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.opacity_props(
        OpacityProps {
            layout: opacity_layout,
            opacity,
        },
        move |cx| vec![cx.render_transform_props(transform_props, move |_cx| children)],
    )
}

/// Like [`wrap_opacity_and_render_transform_with_layouts`], but allows gating interactivity.
///
/// This is useful for Radix-like `hideWhenDetached` behavior where content remains mounted for
/// measurement/state preservation but should not be interactable when hidden.
pub fn wrap_opacity_and_render_transform_with_layouts_gated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity_layout: LayoutStyle,
    opacity: f32,
    transform_props: RenderTransformProps,
    interactive: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: opacity_layout,
            present: true,
            interactive,
        },
        move |cx| {
            vec![cx.opacity_props(
                OpacityProps {
                    layout: opacity_layout,
                    opacity,
                },
                move |cx| vec![cx.render_transform_props(transform_props, move |_cx| children)],
            )]
        },
    )
}

/// Wraps interactive overlay content in opacity + visual-transform layers.
///
/// Unlike [`wrap_opacity_and_render_transform`], the transform does **not** participate in
/// hit-testing or pointer coordinate mapping (paint-only, like CSS `transform` would *not* be).
///
/// This is useful for overlays where we want stable pointer targeting during open/close motion
/// (e.g. outside-press semantics should be based on the steady-state geometry, not the animated
/// scale/slide).
pub fn wrap_opacity_and_visual_transform<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity: f32,
    transform: Transform2D,
    children: Vec<AnyElement>,
) -> AnyElement {
    let layout = fullscreen_motion_layout();
    wrap_opacity_and_visual_transform_with_layouts(
        cx,
        layout,
        opacity,
        VisualTransformProps { layout, transform },
        children,
    )
}

/// Like [`wrap_opacity_and_visual_transform`], but allows gating interactivity.
pub fn wrap_opacity_and_visual_transform_gated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity: f32,
    transform: Transform2D,
    interactive: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    let layout = fullscreen_motion_layout();
    wrap_opacity_and_visual_transform_with_layouts_gated(
        cx,
        layout,
        opacity,
        VisualTransformProps { layout, transform },
        interactive,
        children,
    )
}

/// Like [`wrap_opacity_and_visual_transform`], but allows customizing layouts.
pub fn wrap_opacity_and_visual_transform_with_layouts<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity_layout: LayoutStyle,
    opacity: f32,
    transform_props: VisualTransformProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.opacity_props(
        OpacityProps {
            layout: opacity_layout,
            opacity,
        },
        move |cx| vec![cx.visual_transform_props(transform_props, move |_cx| children)],
    )
}

/// Like [`wrap_opacity_and_visual_transform_with_layouts`], but allows gating interactivity.
pub fn wrap_opacity_and_visual_transform_with_layouts_gated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    opacity_layout: LayoutStyle,
    opacity: f32,
    transform_props: VisualTransformProps,
    interactive: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: opacity_layout,
            present: true,
            interactive,
        },
        move |cx| {
            vec![cx.opacity_props(
                OpacityProps {
                    layout: opacity_layout,
                    opacity,
                },
                move |cx| vec![cx.visual_transform_props(transform_props, move |_cx| children)],
            )]
        },
    )
}

pub fn shadcn_slide_insets(side: Side) -> Edges {
    match side {
        Side::Top => Edges {
            bottom: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Bottom => Edges {
            top: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Left => Edges {
            right: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Right => Edges {
            left: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
    }
}

pub fn shadcn_enter_slide_offset(side: Side, opacity: f32, opening: bool) -> Point {
    if !opening {
        return Point::new(Px(0.0), Px(0.0));
    }

    // shadcn/ui v4 uses `slide-in-from-*-2` (8px) keyed off `data-side`.
    // We approximate that by moving from 8px -> 0 as opacity approaches 1.
    let t = 1.0 - opacity.clamp(0.0, 1.0);
    match side {
        Side::Top => Point::new(Px(0.0), Px(SHADCN_SLIDE_PX.0 * t)),
        Side::Bottom => Point::new(Px(0.0), Px(-SHADCN_SLIDE_PX.0 * t)),
        Side::Left => Point::new(Px(SHADCN_SLIDE_PX.0 * t), Px(0.0)),
        Side::Right => Point::new(Px(-SHADCN_SLIDE_PX.0 * t), Px(0.0)),
    }
}

pub fn shadcn_enter_slide_transform(side: Side, opacity: f32, opening: bool) -> Transform2D {
    Transform2D::translation(shadcn_enter_slide_offset(side, opacity, opening))
}

pub fn shadcn_zoom_transform_with_scale(origin: Point, scale: f32) -> Transform2D {
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(Point::new(Px(-origin.x.0), Px(-origin.y.0)))
}

pub fn shadcn_modal_slide_offset(side: Side, distance: Px, opacity: f32) -> Point {
    // Used by modal panels like `Sheet`, which slide in/out from the same side.
    // This differs from popper overlays (Tooltip/HoverCard/Popover) that slide towards the anchor.
    let t = 1.0 - opacity.clamp(0.0, 1.0);
    match side {
        Side::Top => Point::new(Px(0.0), Px(-distance.0 * t)),
        Side::Bottom => Point::new(Px(0.0), Px(distance.0 * t)),
        Side::Left => Point::new(Px(-distance.0 * t), Px(0.0)),
        Side::Right => Point::new(Px(distance.0 * t), Px(0.0)),
    }
}

pub fn shadcn_modal_slide_transform(side: Side, distance: Px, opacity: f32) -> Transform2D {
    Transform2D::translation(shadcn_modal_slide_offset(side, distance, opacity))
}

pub fn shadcn_zoom_transform(origin: Point, opacity: f32) -> Transform2D {
    // shadcn/ui v4 uses a small zoom-in (95% -> 100%) plus opacity transitions.
    // We approximate that with a fade-driven zoom transform around a popper-style transform origin
    // (Radix exposes this via `--radix-*-transform-origin`).
    let scale = 0.95 + 0.05 * opacity.clamp(0.0, 1.0);
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(Point::new(Px(-origin.x.0), Px(-origin.y.0)))
}

pub fn shadcn_popper_presence_transform(
    side: Side,
    origin: Point,
    opacity: f32,
    scale: f32,
    opening: bool,
) -> Transform2D {
    shadcn_enter_slide_transform(side, opacity, opening)
        * shadcn_zoom_transform_with_scale(origin, scale)
}

/// Infer the anchored placement side from the relative positions of the reference and floating
/// rects.
///
/// This is a geometry-only heuristic used by shadcn-style overlays (including submenus) to pick
/// the slide direction when we don't have a full Popper placement result.
pub fn anchored_side(reference: Rect, floating: Rect) -> Side {
    let ref_center_x = reference.origin.x.0 + reference.size.width.0 * 0.5;
    let ref_center_y = reference.origin.y.0 + reference.size.height.0 * 0.5;
    let float_center_x = floating.origin.x.0 + floating.size.width.0 * 0.5;
    let float_center_y = floating.origin.y.0 + floating.size.height.0 * 0.5;

    let dx = float_center_x - ref_center_x;
    let dy = float_center_y - ref_center_y;

    if dx.abs() >= dy.abs() {
        if dx >= 0.0 { Side::Right } else { Side::Left }
    } else if dy >= 0.0 {
        Side::Bottom
    } else {
        Side::Top
    }
}

/// Compute a popper-like transform origin for an anchored floating rect, expressed in the local
/// coordinate space of the floating rect.
///
/// This mirrors Radix's `--radix-*-transform-origin` outcome in a renderer-agnostic way.
pub fn shadcn_transform_origin_for_anchored_rect(
    reference: Rect,
    floating: Rect,
    side: Side,
) -> Point {
    let w = floating.size.width.0.max(0.0);
    let h = floating.size.height.0.max(0.0);

    let ref_center_x = reference.origin.x.0 + reference.size.width.0 * 0.5;
    let ref_center_y = reference.origin.y.0 + reference.size.height.0 * 0.5;

    let local_x = (ref_center_x - floating.origin.x.0).clamp(0.0, w);
    let local_y = (ref_center_y - floating.origin.y.0).clamp(0.0, h);

    match side {
        Side::Right => Point::new(Px(0.0), Px(local_y)),
        Side::Left => Point::new(Px(w), Px(local_y)),
        Side::Top => Point::new(Px(local_x), Px(h)),
        Side::Bottom => Point::new(Px(local_x), Px(0.0)),
    }
}
