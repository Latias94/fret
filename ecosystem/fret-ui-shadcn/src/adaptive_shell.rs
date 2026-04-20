use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::adaptive::{DeviceShellMode, DeviceShellSwitchPolicy, device_shell_mode};

#[track_caller]
pub(crate) fn resolve_device_shell_mode<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    policy: DeviceShellSwitchPolicy,
) -> DeviceShellMode {
    device_shell_mode(cx, Invalidation::Layout, policy)
}

#[track_caller]
pub(crate) fn is_desktop_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    desktop_min_width: fret_core::Px,
) -> bool {
    resolve_device_shell_mode(
        cx,
        DeviceShellSwitchPolicy::default().desktop_min_width(desktop_min_width),
    )
    .is_desktop()
}
