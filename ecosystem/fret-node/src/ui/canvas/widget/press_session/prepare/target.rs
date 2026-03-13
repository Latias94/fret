#[path = "target_clear.rs"]
mod clear;
#[path = "target_focus.rs"]
mod focus;
#[cfg(test)]
#[path = "target_tests.rs"]
mod tests;

pub(in super::super::super) use focus::{
    prepare_for_edge_anchor_hit, prepare_for_edge_hit, prepare_for_node_hit, prepare_for_port_hit,
    prepare_for_resize_hit,
};
