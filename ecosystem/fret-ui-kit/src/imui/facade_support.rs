//! Shared facade-local support helpers for the `fret-ui-kit::imui` root hub.

use std::time::Duration;

use fret_authoring::{UiWriter, mark_immediate_render_frame};
use fret_core::{Point, Px, Size};
use fret_interaction::dpi;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::IntoUiElement;

mod slider_math;

pub(super) use slider_math::{
    slider_clamp_and_snap, slider_normalize_range, slider_step_or_default,
    slider_value_from_pointer,
};

/// Extension trait bridging `fret-ui-kit` authoring (`UiBuilder<T>`) into an immediate-mode output.
pub trait UiWriterUiKitExt<H: UiHost>: UiWriter<H> {
    /// Render a `UiBuilder<T>` (or other supported authoring value) into the current output list.
    #[track_caller]
    fn add_ui<B>(&mut self, value: B)
    where
        B: IntoUiElement<H>,
    {
        let element = self.with_cx_mut(|cx| IntoUiElement::into_element(value, cx));
        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterUiKitExt<H> for W {}

pub(super) const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0usize;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        i += 1;
    }
    hash
}

pub(super) const KEY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.clicked.v1");
pub(super) const KEY_CHANGED: u64 = fnv1a64(b"fret-ui-kit.imui.changed.v1");
pub(super) const KEY_SECONDARY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.secondary_clicked.v1");
pub(super) const KEY_DOUBLE_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.double_clicked.v1");
pub(super) const KEY_LONG_PRESSED: u64 = fnv1a64(b"fret-ui-kit.imui.long_pressed.v1");
pub(super) const KEY_CONTEXT_MENU_REQUESTED: u64 =
    fnv1a64(b"fret-ui-kit.imui.context_menu_requested.v1");
pub(super) const KEY_POINTER_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.pointer_clicked.v1");
pub(super) const KEY_DRAG_STARTED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_started.v1");
pub(super) const KEY_DRAG_STOPPED: u64 = fnv1a64(b"fret-ui-kit.imui.drag_stopped.v1");
pub(super) const KEY_SELECT_ALL_ON_FOCUS: u64 = fnv1a64(b"fret-ui-kit.imui.select_all_on_focus.v1");
pub(super) const KEY_ACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.activated.v1");
pub(super) const KEY_DEACTIVATED: u64 = fnv1a64(b"fret-ui-kit.imui.deactivated.v1");
pub(super) const KEY_DEACTIVATED_AFTER_EDIT: u64 =
    fnv1a64(b"fret-ui-kit.imui.deactivated_after_edit.v1");
pub(super) const KEY_HOVER_STATIONARY_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.stationary_met.v1");
pub(super) const KEY_HOVER_DELAY_SHORT_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.delay_short_met.v1");
pub(super) const KEY_HOVER_DELAY_NORMAL_MET: u64 =
    fnv1a64(b"fret-ui-kit.imui.hover.delay_normal_met.v1");

// ImGui default: `MouseDragThreshold = 6`.
pub(super) const DEFAULT_DRAG_THRESHOLD_PX: f32 = 6.0;
// ImGui default: `ImGuiStyle::DisabledAlpha = 0.60f`.
pub(super) const DEFAULT_DISABLED_ALPHA: f32 = 0.60;
pub(super) const LONG_PRESS_DELAY: Duration = Duration::from_millis(450);
// ImGui defaults:
// - `HoverStationaryDelay ~= 0.15 sec`
// - `HoverDelayShort ~= 0.15 sec`
// - `HoverDelayNormal ~= 0.40 sec`
pub(super) const HOVER_STATIONARY_DELAY: Duration = Duration::from_millis(150);
pub(super) const HOVER_DELAY_SHORT: Duration = Duration::from_millis(150);
pub(super) const HOVER_DELAY_NORMAL: Duration = Duration::from_millis(400);
pub(super) const DRAG_KIND_MASK: u64 = 0x8000_0000_0000_0000;

pub(super) fn prepare_imui_runtime_for_frame<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    let _ = mark_immediate_render_frame(cx);
}

pub(super) fn snap_point_to_device_pixels(scale_factor: f32, p: Point) -> Point {
    dpi::snap_point_to_device_pixels(scale_factor, p)
}

pub(super) fn snap_size_to_device_pixels(scale_factor: f32, s: Size) -> Size {
    dpi::snap_size_to_device_pixels(scale_factor, s)
}

pub(super) fn point_sub(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 - b.x.0), Px(a.y.0 - b.y.0))
}

pub(super) fn point_add(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 + b.x.0), Px(a.y.0 + b.y.0))
}

pub(super) fn model_value_changed_for<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    current: T,
) -> bool
where
    T: Clone + PartialEq + 'static,
{
    cx.state_for(
        id,
        || current.clone(),
        |previous| {
            let changed = previous != &current;
            if changed {
                *previous = current.clone();
            }
            changed
        },
    )
}
