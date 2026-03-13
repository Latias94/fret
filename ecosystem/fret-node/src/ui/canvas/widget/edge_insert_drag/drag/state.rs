use fret_core::Point;

use crate::ui::canvas::state::InteractionState;

pub(super) fn update_edge_insert_drag_position(
    interaction: &mut InteractionState,
    position: Point,
) -> bool {
    let Some(mut drag) = interaction.edge_insert_drag.clone() else {
        return false;
    };

    drag.pos = position;
    interaction.edge_insert_drag = Some(drag);
    true
}

#[cfg(test)]
mod tests;
