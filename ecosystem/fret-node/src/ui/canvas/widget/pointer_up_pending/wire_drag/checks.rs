use crate::ui::canvas::state::WireDragKind;

pub(super) fn should_promote_pending_wire_drag(
    connect_on_click: bool,
    kind: &WireDragKind,
) -> bool {
    connect_on_click && matches!(kind, WireDragKind::New { .. })
}
