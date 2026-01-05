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

/// Radix-like image loading status for avatars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarImageLoadingStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

/// A frame-based fallback delay gate (Radix `delayMs` outcome).
///
/// This is driven by the caller once per frame (no timers). When a delay is configured, the
/// fallback becomes renderable only after `delay_frames` have elapsed since the first time the
/// caller requested it.
#[derive(Debug, Default, Clone, Copy)]
pub struct AvatarFallbackDelay {
    start_frame: Option<u64>,
}

impl AvatarFallbackDelay {
    /// Drives the delay gate.
    ///
    /// - `now_frame`: current monotonic frame id (`App::frame_id().0`).
    /// - `delay_frames`: `None` means render immediately (no delay).
    /// - `want_render`: whether fallback would be desired (e.g. image not loaded).
    pub fn drive(&mut self, now_frame: u64, delay_frames: Option<u64>, want_render: bool) -> bool {
        let Some(delay_frames) = delay_frames else {
            self.start_frame = None;
            return want_render;
        };

        if !want_render {
            self.start_frame = None;
            return false;
        }

        let start = self.start_frame.get_or_insert(now_frame);
        now_frame.saturating_sub(*start) >= delay_frames
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

    #[test]
    fn fallback_delay_gate_renders_immediately_without_delay() {
        let mut gate = AvatarFallbackDelay::default();
        assert!(!gate.drive(1, None, false));
        assert!(gate.drive(1, None, true));
    }

    #[test]
    fn fallback_delay_gate_waits_until_delay_elapses() {
        let mut gate = AvatarFallbackDelay::default();
        assert!(!gate.drive(10, Some(3), true));
        assert!(!gate.drive(11, Some(3), true));
        assert!(!gate.drive(12, Some(3), true));
        assert!(gate.drive(13, Some(3), true));
    }

    #[test]
    fn fallback_delay_gate_resets_when_not_wanted() {
        let mut gate = AvatarFallbackDelay::default();
        assert!(!gate.drive(10, Some(3), true));
        assert!(!gate.drive(11, Some(3), false));
        assert!(!gate.drive(12, Some(3), true));
        assert!(!gate.drive(13, Some(3), true));
        assert!(!gate.drive(14, Some(3), true));
        assert!(gate.drive(15, Some(3), true));
    }

    #[test]
    fn fallback_visible_hides_when_loaded() {
        assert!(!fallback_visible(AvatarImageLoadingStatus::Loaded, true));
        assert!(fallback_visible(AvatarImageLoadingStatus::Loading, true));
        assert!(!fallback_visible(AvatarImageLoadingStatus::Loading, false));
    }
}

