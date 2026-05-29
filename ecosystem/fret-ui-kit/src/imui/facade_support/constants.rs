use std::time::Duration;

pub(in crate::imui) const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0usize;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        i += 1;
    }
    hash
}

pub(in crate::imui) const KEY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.clicked.v1");
pub(in crate::imui) const KEY_CHANGED: u64 = fnv1a64(b"fret-ui-kit.imui.changed.v1");
pub(in crate::imui) const KEY_SECONDARY_CLICKED: u64 =
    fnv1a64(b"fret-ui-kit.imui.secondary_clicked.v1");
pub(in crate::imui) const KEY_DOUBLE_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.double_clicked.v1");
pub(in crate::imui) const KEY_LONG_PRESSED: u64 = fnv1a64(b"fret-ui-kit.imui.long_pressed.v1");
pub(in crate::imui) const KEY_CONTEXT_MENU_REQUESTED: u64 =
    fnv1a64(b"fret-ui-kit.imui.context_menu_requested.v1");
pub(in crate::imui) const KEY_POINTER_CLICKED: u64 =
    fnv1a64(b"fret-ui-kit.imui.pointer_clicked.v1");
pub(in crate::imui) const KEY_DRAG_STARTED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_started.v1");
pub(in crate::imui) const KEY_DRAG_STOPPED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_stopped.v1");
pub(in crate::imui) const KEY_SELECT_ALL_ON_FOCUS: u64 =
    fnv1a64(b"fret-ui-kit.imui.select_all_on_focus.v1");
pub(in crate::imui) const KEY_ACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.activated.v1");
pub(in crate::imui) const KEY_DEACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.deactivated.v1");
pub(in crate::imui) const KEY_DEACTIVATED_AFTER_EDIT: u64 =
    fnv1a64(b"fret-ui-kit.imui.deactivated_after_edit.v1");
pub(in crate::imui) const KEY_HOVER_STATIONARY_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.stationary_met.v1");
pub(in crate::imui) const KEY_HOVER_DELAY_SHORT_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.delay_short_met.v1");
pub(in crate::imui) const KEY_HOVER_DELAY_NORMAL_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.delay_normal_met.v1");

// ImGui default: `MouseDragThreshold = 6`.
pub(in crate::imui) const DEFAULT_DRAG_THRESHOLD_PX: f32 = 6.0;
// ImGui default: `ImGuiStyle::DisabledAlpha = 0.60f`.
pub(in crate::imui) const DEFAULT_DISABLED_ALPHA: f32 = 0.60;
pub(in crate::imui) const LONG_PRESS_DELAY: Duration = Duration::from_millis(450);
// ImGui defaults:
// - `HoverStationaryDelay ~= 0.15 sec`
// - `HoverDelayShort ~= 0.15 sec`
// - `HoverDelayNormal ~= 0.40 sec`
pub(in crate::imui) const HOVER_STATIONARY_DELAY: Duration = Duration::from_millis(150);
pub(in crate::imui) const HOVER_DELAY_SHORT: Duration = Duration::from_millis(150);
pub(in crate::imui) const HOVER_DELAY_NORMAL: Duration = Duration::from_millis(400);
pub(in crate::imui) const DRAG_KIND_MASK: u64 = 0x8000_0000_0000_0000;
