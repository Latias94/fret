mod edit;
mod instant;
mod pointer_edges;
mod response;

pub(in super::super) use edit::mark_lifecycle_edit;
pub(in super::super) use instant::mark_lifecycle_instant_if_inactive;
pub(in super::super) use pointer_edges::{
    mark_lifecycle_activated_on_left_pointer_down, mark_lifecycle_deactivated_on_left_pointer_up,
};
pub(in super::super) use response::{
    populate_response_lifecycle_from_active_state, populate_response_lifecycle_transients,
};
