use crate::ui::canvas::state::InteractionState;
use crate::ui::canvas::widget::*;

pub(super) fn resolve_paint_root_hovered_edge(interaction: &InteractionState) -> Option<EdgeId> {
    let edge_insert_target = interaction
        .edge_insert_drag
        .as_ref()
        .map(|drag| drag.edge)
        .or_else(|| {
            interaction
                .pending_edge_insert_drag
                .as_ref()
                .map(|drag| drag.edge)
        });
    let insert_node_drag_edge = interaction
        .insert_node_drag_preview
        .as_ref()
        .and_then(|preview| preview.edge);

    edge_insert_target
        .or(insert_node_drag_edge)
        .or(interaction.hover_edge)
}

#[cfg(test)]
mod tests;
