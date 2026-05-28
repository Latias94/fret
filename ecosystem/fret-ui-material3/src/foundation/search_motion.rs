//! Material SearchBar/SearchView motion policy.
//!
//! Compose Material3 models search transitions with a dedicated `SearchBarState`: a spatial
//! progress channel for geometry and a separate effects channel for content fade. Fret keeps that
//! policy in Material3 instead of the core overlay primitives because the meaning of the progress
//! is recipe-specific.

use fret_core::{Point, Px, Rect, Transform2D};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};

use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchMotionKind {
    Docked,
    FullScreen,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SearchMotionFrame {
    pub present: bool,
    pub progress: f32,
    pub content_alpha: f32,
}

#[derive(Debug, Default)]
struct SearchMotionRuntime {
    progress: SpringAnimator,
    content_alpha: SpringAnimator,
    last_open: bool,
    closing_started_at: Option<u64>,
}

pub(crate) fn drive_search_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    kind: SearchMotionKind,
    close_grace_frames: Option<u64>,
) -> SearchMotionFrame {
    let now_frame = cx.frame_id.0;
    let target = if open { 1.0 } else { 0.0 };
    let (expand_spec, collapse_spec, content_in_spec, content_out_spec) = {
        let theme = Theme::global(&*cx.app);
        match kind {
            SearchMotionKind::Docked => (
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::DefaultSpatial),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastEffects),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastEffects),
            ),
            SearchMotionKind::FullScreen => (
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::SlowSpatial),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::DefaultSpatial),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastEffects),
                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastEffects),
            ),
        }
    };

    let (progress, content_alpha, animating, within_close_grace) =
        cx.slot_state(SearchMotionRuntime::default, |rt| {
            if open {
                rt.closing_started_at = None;
            } else if rt.last_open && rt.closing_started_at.is_none() {
                rt.closing_started_at = Some(now_frame);
            }

            if !rt.progress.is_initialized() {
                rt.progress.reset(now_frame, target);
            }
            if !rt.content_alpha.is_initialized() {
                rt.content_alpha.reset(now_frame, target);
            }

            let spatial_spec = if open { expand_spec } else { collapse_spec };
            let effects_spec = if open {
                content_in_spec
            } else {
                content_out_spec
            };
            rt.progress.set_target(now_frame, target, spatial_spec);
            rt.content_alpha.set_target(now_frame, target, effects_spec);

            rt.progress.advance(now_frame);
            rt.content_alpha.advance(now_frame);

            let animating = rt.progress.is_active() || rt.content_alpha.is_active();
            let within_close_grace = close_grace_frames.is_some_and(|frames| {
                rt.closing_started_at
                    .is_some_and(|start| now_frame.saturating_sub(start) <= frames)
            });
            rt.last_open = open;

            (
                rt.progress.value(),
                rt.content_alpha.value(),
                animating,
                within_close_grace,
            )
        });

    if animating {
        cx.request_animation_frame();
    }

    let present = if close_grace_frames.is_some() {
        open || within_close_grace
    } else {
        open || animating
    };

    SearchMotionFrame {
        present,
        progress: progress.clamp(0.0, 1.0),
        content_alpha: content_alpha.clamp(0.0, 1.0),
    }
}

pub(crate) fn search_full_screen_geometry_transform(
    progress: f32,
    viewport: Rect,
    collapsed: Rect,
) -> Transform2D {
    let progress = progress.clamp(0.0, 1.0);
    let viewport_w = viewport.size.width.0.max(1.0);
    let viewport_h = viewport.size.height.0.max(1.0);

    let start_sx = (collapsed.size.width.0 / viewport_w).clamp(0.05, 1.0);
    let start_sy = (collapsed.size.height.0 / viewport_h).clamp(0.05, 1.0);
    let sx = lerp(start_sx, 1.0, progress);
    let sy = lerp(start_sy, 1.0, progress);

    let viewport_center = Point::new(Px(viewport_w * 0.5), Px(viewport_h * 0.5));
    let collapsed_center = Point::new(
        Px(collapsed.origin.x.0 - viewport.origin.x.0 + collapsed.size.width.0 * 0.5),
        Px(collapsed.origin.y.0 - viewport.origin.y.0 + collapsed.size.height.0 * 0.5),
    );
    let center = Point::new(
        Px(lerp(collapsed_center.x.0, viewport_center.x.0, progress)),
        Px(lerp(collapsed_center.y.0, viewport_center.y.0, progress)),
    );

    Transform2D::translation(center)
        * Transform2D {
            a: sx,
            d: sy,
            ..Transform2D::IDENTITY
        }
        * Transform2D::translation(Point::new(
            Px(-viewport_center.x.0),
            Px(-viewport_center.y.0),
        ))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
