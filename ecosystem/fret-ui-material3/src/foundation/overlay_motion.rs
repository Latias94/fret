//! Material3 overlay motion helpers.
//!
//! Compose Material3 drives many overlay open/close animations (menu/tooltip, etc.) using the
//! MotionScheme `{FastSpatial, FastEffects}` springs and scales from `0.8 -> 1.0` while fading
//! from `0.0 -> 1.0`.

use fret_ui::{ElementContext, Theme, UiHost};

use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;

pub const OVERLAY_CLOSED_SCALE: f32 = 0.8;
pub const OVERLAY_OPEN_SCALE: f32 = 1.0;
pub const OVERLAY_CLOSED_ALPHA: f32 = 0.0;
pub const OVERLAY_OPEN_ALPHA: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct OverlayOpenCloseMotion {
    pub present: bool,
    pub alpha: f32,
    pub scale: f32,
}

pub fn drive_overlay_open_close_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    close_grace_frames: Option<u64>,
) -> OverlayOpenCloseMotion {
    #[derive(Default)]
    struct State {
        scale: SpringAnimator,
        alpha: SpringAnimator,
        last_open: bool,
        closing_started_at: Option<u64>,
    }

    let now_frame = cx.frame_id.0;
    let target_scale = if open {
        OVERLAY_OPEN_SCALE
    } else {
        OVERLAY_CLOSED_SCALE
    };
    let target_alpha = if open {
        OVERLAY_OPEN_ALPHA
    } else {
        OVERLAY_CLOSED_ALPHA
    };

    let (scale_spec, alpha_spec) = {
        let theme = Theme::global(&*cx.app);
        (
            sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial),
            sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastEffects),
        )
    };

    // Compose's `spring()` for Float uses a visibility threshold that is notably larger than
    // 1e-3. Keep overlay mount/unmount decisions stable by snapping close-to-target values.
    let settle_epsilon = 0.01;

    let (scale, alpha, animating, within_close_grace) = cx.with_state(State::default, |st| {
        if open {
            st.closing_started_at = None;
        } else if st.last_open && st.closing_started_at.is_none() {
            st.closing_started_at = Some(now_frame);
        }

        if !st.scale.is_initialized() {
            st.scale.reset(now_frame, OVERLAY_CLOSED_SCALE);
        }
        if !st.alpha.is_initialized() {
            st.alpha.reset(now_frame, OVERLAY_CLOSED_ALPHA);
        }

        st.scale.set_target(now_frame, target_scale, scale_spec);
        st.alpha.set_target(now_frame, target_alpha, alpha_spec);
        st.scale.advance(now_frame);
        st.alpha.advance(now_frame);

        let mut scale = st.scale.value();
        if (scale - target_scale).abs() <= settle_epsilon {
            st.scale.reset(now_frame, target_scale);
            scale = target_scale;
        }

        let mut alpha = st.alpha.value();
        if (alpha - target_alpha).abs() <= settle_epsilon {
            st.alpha.reset(now_frame, target_alpha);
            alpha = target_alpha;
        }

        let animating = st.scale.is_active() || st.alpha.is_active();
        let within_close_grace = close_grace_frames.is_some_and(|frames| {
            st.closing_started_at
                .is_some_and(|start| now_frame.saturating_sub(start) <= frames)
        });
        st.last_open = open;
        (scale, alpha, animating, within_close_grace)
    });

    if animating {
        cx.request_frame();
    }

    let present = if close_grace_frames.is_some() {
        open || within_close_grace
    } else {
        open || animating
    };

    OverlayOpenCloseMotion {
        present,
        alpha: alpha.clamp(0.0, 1.0),
        scale: scale.max(0.0),
    }
}
