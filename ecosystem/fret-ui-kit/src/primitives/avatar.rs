//! Avatar primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/avatar/src/avatar.tsx`
//!
//! Radix Avatar tracks image loading status (`idle`/`loading`/`loaded`/`error`) and uses an
//! optional delay before rendering fallback content.
//!
//! In Fret, `ImageId` represents an already-registered renderer resource. For apps that load
//! images asynchronously (e.g. decode/upload, network fetch), a common pattern is to store
//! `Option<ImageId>` (or an enum) in a model and update it when the image becomes available. This
//! facade provides the Radix-named status enum and a small, frame-based fallback delay helper.
use std::time::Duration;

/// Radix-like image loading status for avatars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarImageLoadingStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

/// A duration-driven fallback delay gate (Radix `delayMs` outcome).
///
/// This is driven by the caller once per frame (no timers). When a delay is configured, the
/// fallback becomes renderable only after the delay duration has elapsed since the first time the
/// caller requested it.
#[derive(Debug, Default, Clone, Copy)]
pub struct AvatarFallbackDelay {
    start_frame_id: Option<u64>,
    last_frame_id: Option<u64>,
    elapsed: Duration,
}

impl AvatarFallbackDelay {
    /// Drives the delay gate.
    ///
    /// - `frame_id`: current monotonic frame id (`App::frame_id().0`).
    /// - `dt`: effective per-frame delta (clamped; recommended: `motion::effective_frame_delta_for_cx`).
    /// - `delay`: `None` means render immediately (no delay).
    /// - `want_render`: whether fallback would be desired (e.g. image not loaded).
    pub fn drive(
        &mut self,
        frame_id: u64,
        dt: Duration,
        delay: Option<Duration>,
        want_render: bool,
    ) -> bool {
        let Some(delay) = delay else {
            *self = Self::default();
            return want_render;
        };

        if !want_render {
            *self = Self::default();
            return false;
        }

        if delay == Duration::ZERO {
            return true;
        }

        let _ = self.start_frame_id.get_or_insert(frame_id);
        match self.last_frame_id {
            None => {
                self.last_frame_id = Some(frame_id);
            }
            Some(prev) if prev != frame_id => {
                self.last_frame_id = Some(frame_id);
                self.elapsed = self.elapsed.saturating_add(dt);
            }
            Some(_) => {}
        }

        self.elapsed >= delay
    }
}

/// Returns `true` when avatar fallback content should be visible, matching Radix's
/// `canRender && status !== 'loaded'` outcome.
pub fn fallback_visible(status: AvatarImageLoadingStatus, delay_ready: bool) -> bool {
    delay_ready && status != AvatarImageLoadingStatus::Loaded
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT_16MS: Duration = Duration::from_millis(16);

    #[test]
    fn fallback_delay_gate_renders_immediately_without_delay() {
        let mut gate = AvatarFallbackDelay::default();
        assert!(!gate.drive(1, DT_16MS, None, false));
        assert!(gate.drive(1, DT_16MS, None, true));
    }

    #[test]
    fn fallback_delay_gate_waits_until_delay_elapses() {
        let mut gate = AvatarFallbackDelay::default();
        let delay = Some(Duration::from_millis(32));
        assert!(!gate.drive(10, DT_16MS, delay, true));
        assert!(!gate.drive(11, DT_16MS, delay, true));
        assert!(gate.drive(12, DT_16MS, delay, true));
    }

    #[test]
    fn fallback_delay_gate_resets_when_not_wanted() {
        let mut gate = AvatarFallbackDelay::default();
        let delay = Some(Duration::from_millis(32));
        assert!(!gate.drive(10, DT_16MS, delay, true));
        assert!(!gate.drive(11, DT_16MS, delay, false));
        assert!(!gate.drive(12, DT_16MS, delay, true));
        assert!(!gate.drive(13, DT_16MS, delay, true));
        assert!(gate.drive(14, DT_16MS, delay, true));
    }

    #[test]
    fn fallback_visible_hides_when_loaded() {
        assert!(!fallback_visible(AvatarImageLoadingStatus::Loaded, true));
        assert!(fallback_visible(AvatarImageLoadingStatus::Loading, true));
        assert!(!fallback_visible(AvatarImageLoadingStatus::Loading, false));
    }
}
