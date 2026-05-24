use fret_core::AppWindowId;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::event_clipboard_feedback_cx::ClipboardFeedbackCx;

impl<H: UiHost> ClipboardFeedbackCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}
