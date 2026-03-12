use crate::ui::canvas::state::{InteractionState, PendingWireDrag, WireDrag, WireDragKind};

pub(super) fn activate_pending_wire_drag(
    interaction: &mut InteractionState,
    pending: PendingWireDrag,
) -> WireDragKind {
    interaction.pending_wire_drag = None;
    let kind = pending.kind.clone();
    interaction.wire_drag = Some(WireDrag {
        kind: pending.kind,
        pos: pending.start_pos,
    });
    kind
}

#[cfg(test)]
mod tests;
