use fret_core::Px;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::adaptive::{DeviceShellSwitchPolicy, device_shell_mode};

#[track_caller]
pub(crate) fn is_desktop_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    desktop_min_width: Px,
) -> bool {
    device_shell_mode(
        cx,
        Invalidation::Layout,
        DeviceShellSwitchPolicy::default().desktop_min_width(desktop_min_width),
    )
    .is_desktop()
}
