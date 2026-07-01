use fret_ui::UiHost;

use super::AppUi;

impl<'cx, 'a, H: UiHost> AppUi<'cx, 'a, H> {
    /// Request the next animation frame from the default app-facing render lane.
    ///
    /// Use this for frame-driven progression that must continue without fresh input events.
    pub fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }

    /// Toggle the continuous-frames lease for the current view root without reopening
    /// `ElementContext`.
    ///
    /// Use this for app-facing surfaces that need ongoing frame delivery while a mode remains
    /// active. Advanced/component code can still opt into the lower-level helper directly.
    pub fn set_continuous_frames(&mut self, enabled: bool) {
        fret_ui_kit::declarative::scheduling::set_continuous_frames(self.cx, enabled);
    }
}
