use std::sync::Arc;

use fret_ui::GlobalElementId;

const FLOAT_WINDOW_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;
const FLOAT_WINDOW_RESIZE_KIND_BASE: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.float_window.resize.v1");

pub(in crate::imui) const KEY_FLOAT_WINDOW_ACTIVATE: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.float_window.activate.v1");
pub(in crate::imui) const KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED: u64 =
    super::super::fnv1a64(b"fret-ui-kit.imui.float_window.toggle_collapsed.v1");

pub(in crate::imui) type OnFloatingAreaLeftDoubleClick =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiPointerActionHost, fret_ui::action::ActionCx) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::imui) enum FloatWindowResizeHandle {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub(in crate::imui) fn float_window_drag_kind_for_element(
    element: GlobalElementId,
) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(FLOAT_WINDOW_DRAG_KIND_MASK | element.0)
}

pub(in crate::imui) fn float_window_resize_kind_for_element(
    element: GlobalElementId,
    handle: FloatWindowResizeHandle,
) -> fret_runtime::DragKindId {
    let handle_tag = match handle {
        FloatWindowResizeHandle::Left => 1,
        FloatWindowResizeHandle::Right => 2,
        FloatWindowResizeHandle::Top => 3,
        FloatWindowResizeHandle::Bottom => 4,
        FloatWindowResizeHandle::TopLeft => 5,
        FloatWindowResizeHandle::TopRight => 6,
        FloatWindowResizeHandle::BottomLeft => 7,
        FloatWindowResizeHandle::BottomRight => 8,
    };
    fret_runtime::DragKindId(
        FLOAT_WINDOW_RESIZE_KIND_BASE ^ element.0.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ handle_tag,
    )
}
