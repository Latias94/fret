#![allow(dead_code)]

use super::*;

fn node_contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| node_contains_text(c, needle))
}

pub(super) fn find_by_tag_and_text<'a>(
    root: &'a WebNode,
    tag: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| n.tag == tag && node_contains_text(n, text))
}

pub(super) fn find_by_class_token<'a>(root: &'a WebNode, token: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_token(n, token))
}

pub(super) fn find_by_class_token_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    token: &str,
) -> Option<&'a WebNode> {
    find_first_in_theme(theme, &|n| class_has_token(n, token))
}

pub(super) fn find_by_class_tokens<'a>(root: &'a WebNode, tokens: &[&str]) -> Option<&'a WebNode> {
    find_first(root, &|n| tokens.iter().all(|t| class_has_token(n, t)))
}
