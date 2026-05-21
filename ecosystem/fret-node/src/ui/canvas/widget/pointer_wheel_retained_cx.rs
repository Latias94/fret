use fret_runtime::Platform;
use fret_ui::{UiHost, retained_bridge::EventCx};

use super::pointer_wheel_cx::PointerWheelPlatformCx;

impl<H: UiHost> PointerWheelPlatformCx for EventCx<'_, H> {
    fn platform(&self) -> Platform {
        self.input_ctx.platform
    }
}
