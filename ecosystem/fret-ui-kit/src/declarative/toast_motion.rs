//! Motion token helpers for toast / Sonner-style surfaces.
//!
//! These keys are ecosystem-level and intentionally optional. shadcn-scoped keys remain the
//! preferred knobs for shadcn recipes, but the helpers also support canonical cross-ecosystem
//! semantic keys (e.g. `duration.motion.presence.*`) as a fallback.

use std::time::Duration;

use fret_ui::theme::CubicBezier;
use fret_ui::{ElementContext, Theme, UiHost};

const THEME_DURATION_SHADCN_MOTION_TOAST_ENTER: &str = "duration.shadcn.motion.toast.enter";
const THEME_DURATION_SHADCN_MOTION_TOAST_EXIT: &str = "duration.shadcn.motion.toast.exit";
const THEME_EASING_SHADCN_MOTION_TOAST: &str = "easing.shadcn.motion.toast";

const THEME_EASING_SHADCN_MOTION: &str = "easing.shadcn.motion";

const THEME_DURATION_MOTION_PRESENCE_ENTER: &str = "duration.motion.presence.enter";
const THEME_DURATION_MOTION_PRESENCE_EXIT: &str = "duration.motion.presence.exit";
const THEME_EASING_MOTION_STANDARD: &str = "easing.motion.standard";

pub const DEFAULT_SHADCN_TOAST_ENTER_DURATION: Duration = Duration::from_millis(200);
pub const DEFAULT_SHADCN_TOAST_EXIT_DURATION: Duration = Duration::from_millis(200);

fn theme_duration_ms_by_key<H: UiHost>(cx: &ElementContext<'_, H>, key: &str) -> Option<Duration> {
    let theme = Theme::global(&*cx.app);
    theme
        .duration_ms_by_key(key)
        .map(|ms| Duration::from_millis(ms as u64))
}

/// shadcn semantic toast enter duration (`duration.shadcn.motion.toast.enter`).
pub fn shadcn_toast_enter_duration_opt<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Duration> {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_TOAST_ENTER)
        .or_else(|| theme_duration_ms_by_key(cx, THEME_DURATION_MOTION_PRESENCE_ENTER))
}

/// shadcn semantic toast exit duration (`duration.shadcn.motion.toast.exit`).
pub fn shadcn_toast_exit_duration_opt<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Duration> {
    theme_duration_ms_by_key(cx, THEME_DURATION_SHADCN_MOTION_TOAST_EXIT)
        .or_else(|| theme_duration_ms_by_key(cx, THEME_DURATION_MOTION_PRESENCE_EXIT))
}

/// shadcn semantic toast easing curve (`easing.shadcn.motion.toast`, falling back to
/// `easing.shadcn.motion`).
pub fn shadcn_toast_ease_bezier<H: UiHost>(cx: &ElementContext<'_, H>) -> CubicBezier {
    let theme = Theme::global(&*cx.app);
    theme
        .easing_by_key(THEME_EASING_SHADCN_MOTION_TOAST)
        .or_else(|| theme.easing_by_key(THEME_EASING_SHADCN_MOTION))
        .or_else(|| theme.easing_by_key(THEME_EASING_MOTION_STANDARD))
        .unwrap_or(CubicBezier {
            x1: crate::headless::easing::SHADCN_EASE.x1,
            y1: crate::headless::easing::SHADCN_EASE.y1,
            x2: crate::headless::easing::SHADCN_EASE.x2,
            y2: crate::headless::easing::SHADCN_EASE.y2,
        })
}
