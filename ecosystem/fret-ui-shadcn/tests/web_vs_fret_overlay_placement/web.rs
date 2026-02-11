use super::*;

#[path = "../support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
pub(crate) use web_golden_shadcn::*;

#[path = "../support/web_query.rs"]
mod web_query;

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
    for portal in &theme.portals {
        if let Some(found) = web_find_by_data_slot(portal, slot) {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) = web_find_by_data_slot(wrapper, slot) {
            return found;
        }
    }
    panic!("missing web portal node with data-slot={slot}")
}

pub(crate) fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        for child in &node.children {
            if let Some(found) = find_attr_in_subtree(child, key) {
                return Some(found);
            }
        }
        None
    })
}
