use fret_ui::ElementContext;
use fret_ui::UiHost;

use crate::declarative::transition;
use crate::headless::presence::{PresenceOutput, ScaleFadePresenceOutput};

/// Drive a fade presence transition using the UI runtime's monotonic frame clock.
///
/// This helper keeps animation scheduling out of individual components:
/// - it is driven by runner monotonic clocks,
/// - it holds a continuous-frames lease while animating,
/// - and it requests redraws while animating.
#[track_caller]
pub fn fade_presence<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    fade_ticks: u64,
) -> PresenceOutput {
    fade_presence_with_durations(cx, open, fade_ticks, fade_ticks)
}

#[track_caller]
pub fn fade_presence_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
) -> PresenceOutput {
    let out = transition::drive_transition_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        crate::headless::easing::smoothstep,
    );
    PresenceOutput {
        present: out.present,
        opacity: out.progress,
        animating: out.animating,
    }
}

/// Drive a scale+fade presence transition using the UI runtime's monotonic frame clock.
#[track_caller]
pub fn scale_fade_presence<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    ticks: u64,
    from_scale: f32,
    to_scale: f32,
) -> ScaleFadePresenceOutput {
    scale_fade_presence_with_durations(cx, open, ticks, ticks, from_scale, to_scale)
}

/// Drive a scale+fade presence transition with separate open/close durations.
#[track_caller]
pub fn scale_fade_presence_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    from_scale: f32,
    to_scale: f32,
) -> ScaleFadePresenceOutput {
    scale_fade_presence_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        from_scale,
        to_scale,
        crate::headless::easing::smoothstep,
    )
}

#[track_caller]
pub fn scale_fade_presence_with_durations_and_easing<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    from_scale: f32,
    to_scale: f32,
    ease: fn(f32) -> f32,
) -> ScaleFadePresenceOutput {
    let out = transition::drive_transition_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        ease,
    );
    let scale = from_scale + (to_scale - from_scale) * out.progress;
    ScaleFadePresenceOutput {
        present: out.present,
        opacity: out.progress,
        scale,
        animating: out.animating,
    }
}
