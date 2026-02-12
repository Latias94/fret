use super::*;

pub(super) fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

#[allow(dead_code)]
pub(super) fn contains_id(node: &WebNode, needle: &str) -> bool {
    if node
        .id
        .as_deref()
        .or_else(|| node.attrs.get("id").map(String::as_str))
        .is_some_and(|id| id == needle)
    {
        return true;
    }
    node.children.iter().any(|c| contains_id(c, needle))
}

#[allow(dead_code)]
pub(super) fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        node.children
            .iter()
            .find_map(|child| find_attr_in_subtree(child, key))
    })
}
