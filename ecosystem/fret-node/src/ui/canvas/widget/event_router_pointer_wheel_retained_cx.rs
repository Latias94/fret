use fret_runtime::Platform;
use fret_ui::{EventCx, UiHost};

use super::event_router_pointer_wheel_cx::PointerWheelRoutePlatformCx;

impl<H: UiHost> PointerWheelRoutePlatformCx for EventCx<'_, H> {
    fn platform(&self) -> Platform {
        self.input_ctx.platform
    }
}
