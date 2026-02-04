mod context_menu;
mod insert;
mod picker;
mod prelude;

pub(super) use context_menu::open_edge_insert_context_menu;
pub(super) use insert::insert_node_on_edge;
pub(super) use picker::open_edge_insert_node_picker;

// Keep this module as a thin facade. The individual steps are split into submodules to reduce drift
// across the edge-insert workflows (context menu vs searcher vs direct split).

// Intentionally empty: logic lives in submodules.
