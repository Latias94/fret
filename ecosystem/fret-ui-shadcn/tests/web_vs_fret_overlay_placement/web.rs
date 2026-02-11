use super::*;

#[path = "../support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
pub(crate) use web_golden_shadcn::*;

#[path = "../support/web_query.rs"]
mod web_query;

#[path = "../support/web_portals.rs"]
mod web_portals;

#[path = "../support/web_tree.rs"]
mod web_tree;

pub(crate) fn web_find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    web_query::find_by_data_slot_and_state(root, slot, state)
}

pub(crate) fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    web_query::find_by_data_slot(root, slot)
}

pub(crate) fn web_portal_node_by_data_slot<'a>(
    theme: &'a WebGoldenTheme,
    slot: &str,
) -> &'a WebNode {
    for portal in web_portals::portal_roots(theme) {
        if let Some(found) = web_find_by_data_slot(portal, slot) {
            return found;
        }
    }
    panic!("missing web portal node with data-slot={slot}")
}

pub(crate) fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    web_tree::find_attr_in_subtree(node, key)
}
