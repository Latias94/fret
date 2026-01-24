//! Presence (Radix-aligned outcomes).
//!
//! Radix's Presence primitive is used to coordinate mount/unmount transitions while keeping
//! behavioral outcomes consistent (e.g. closing animations that remain paintable but not
//! interactive).
//!
//! Fret splits this into:
//!
//! - `crate::headless::transition`: deterministic timeline (`TransitionTimeline`).
//! - `crate::declarative::transition`: runtime-driven driver (frame clock + redraw scheduling).
//! - `crate::declarative::presence`: thin mapping helpers (fade / scale+fade).
//!
//! This module provides a stable, Radix-named facade surface and keeps call sites from reaching
//! into the `declarative` module directly. See <https://github.com/radix-ui/primitives>.

use fret_ui::{ElementContext, UiHost};

pub use crate::headless::presence::{
    FadePresence, PresenceOutput, ScaleFadePresence, ScaleFadePresenceOutput,
};

/// Drive a fade presence transition using the UI runtime's monotonic frame clock.
///
/// This is a thin facade around `crate::declarative::presence::fade_presence`.
#[track_caller]
pub fn fade_presence<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    fade_ticks: u64,
) -> PresenceOutput {
    crate::declarative::presence::fade_presence(cx, open, fade_ticks)
}

/// Drive a fade presence transition with separate open/close durations.
#[track_caller]
pub fn fade_presence_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
) -> PresenceOutput {
    crate::declarative::presence::fade_presence_with_durations(cx, open, open_ticks, close_ticks)
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
    crate::declarative::presence::scale_fade_presence(cx, open, ticks, from_scale, to_scale)
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
    crate::declarative::presence::scale_fade_presence_with_durations(
        cx,
        open,
        open_ticks,
        close_ticks,
        from_scale,
        to_scale,
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
    crate::declarative::presence::scale_fade_presence_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        from_scale,
        to_scale,
        ease,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::{Effect, FrameId, TickId};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn fade_presence_requests_redraw_while_animating() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fn render(app: &mut App, window: AppWindowId) -> PresenceOutput {
            fret_ui::elements::with_element_cx(app, window, bounds(), "p0", |cx| {
                fade_presence(cx, true, 3)
            })
        }

        // Simulate the runner's monotonic clock.
        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        // First call enters the opening phase: request animation frames + redraw.

        let out0 = render(&mut app, window);
        let effects0 = app.flush_effects();
        assert!(out0.present);
        assert!(out0.animating);
        assert!(
            effects0
                .iter()
                .any(|e| *e == Effect::RequestAnimationFrame(window))
        );
        assert!(effects0.iter().any(|e| *e == Effect::Redraw(window)));

        // While animating: keep requesting redraw, but do not reacquire a new RAF lease.

        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let out1 = render(&mut app, window);
        let effects1 = app.flush_effects();
        assert!(out1.present);
        assert!(out1.animating);
        assert!(
            !effects1
                .iter()
                .any(|e| *e == Effect::RequestAnimationFrame(window))
        );
        assert!(effects1.iter().any(|e| *e == Effect::Redraw(window)));

        // Stable open: no longer animating, so no more redraw requests.

        app.set_tick_id(TickId(3));
        app.set_frame_id(FrameId(3));
        let out2 = render(&mut app, window);
        let effects2 = app.flush_effects();
        assert!(out2.present);
        assert!(!out2.animating);
        assert!(effects2.is_empty());
    }

    #[test]
    fn fade_presence_reacquires_animation_frame_on_new_animation() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fn render(app: &mut App, window: AppWindowId, open: bool, ticks: u64) -> PresenceOutput {
            fret_ui::elements::with_element_cx(app, window, bounds(), "p1", |cx| {
                fade_presence(cx, open, ticks)
            })
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        // Open first and reach a stable state.
        let _ = render(&mut app, window, true, 1);
        let _ = app.flush_effects();
        let _ = render(&mut app, window, true, 1);
        let _ = app.flush_effects();

        // Trigger a close animation: reacquire a RAF lease and request a redraw.
        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let out = render(&mut app, window, false, 3);
        let effects = app.flush_effects();
        assert!(out.present);
        assert!(out.animating);
        assert!(
            effects
                .iter()
                .any(|e| *e == Effect::RequestAnimationFrame(window))
        );
        assert!(effects.iter().any(|e| *e == Effect::Redraw(window)));
    }

    #[test]
    fn scale_fade_presence_requests_redraw_while_animating() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let out0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "p2", |cx| {
            scale_fade_presence(cx, true, 3, 0.95, 1.0)
        });
        let effects0 = app.flush_effects();
        assert!(out0.present);
        assert!(out0.animating);
        assert!(
            effects0
                .iter()
                .any(|e| *e == Effect::RequestAnimationFrame(window))
        );
        assert!(effects0.iter().any(|e| *e == Effect::Redraw(window)));
    }

    #[test]
    fn scale_fade_presence_with_durations_and_easing_applies_custom_ease() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let out0 = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "p3", |cx| {
            scale_fade_presence_with_durations_and_easing(
                cx,
                true,
                4,
                4,
                0.8,
                1.0,
                crate::headless::easing::linear,
            )
        });
        assert!(out0.present);
        assert!(out0.animating);
        assert!((out0.opacity - 0.25).abs() < 1e-6);
        assert!((out0.scale - 0.85).abs() < 1e-6);
    }

    #[test]
    fn separate_presence_callsites_do_not_share_state() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fn render_pair(
            app: &mut App,
            window: AppWindowId,
            open_a: bool,
            open_b: bool,
        ) -> (PresenceOutput, PresenceOutput) {
            fret_ui::elements::with_element_cx(app, window, bounds(), "p_callsites", |cx| {
                let a = fade_presence_with_durations(cx, open_a, 2, 2);
                let b = fade_presence_with_durations(cx, open_b, 2, 2);
                (a, b)
            })
        }

        // Frame 1: open both.
        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let (a0, b0) = render_pair(&mut app, window, true, true);
        assert!(a0.present);
        assert!(b0.present);

        // Frame 2: keep A open, start closing B.
        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let (a1, b1) = render_pair(&mut app, window, true, false);
        assert!(a1.present);
        assert!(b1.present, "B should remain present while closing");

        // Frame 3: B should now be unmounted, A remains open.
        app.set_tick_id(TickId(3));
        app.set_frame_id(FrameId(3));
        let (a2, b2) = render_pair(&mut app, window, true, false);
        assert!(a2.present);
        assert!(!b2.present, "B should unmount after close ticks");
    }
}
