use super::*;

pub(super) fn portal_roots<'a>(
    theme: &'a WebGoldenTheme,
) -> impl Iterator<Item = &'a WebNode> + 'a {
    theme.portals.iter().chain(theme.portal_wrappers.iter())
}

#[allow(dead_code)]
pub(super) fn find_portal_by_role<'a>(
    theme: &'a WebGoldenTheme,
    role: &str,
) -> Option<&'a WebNode> {
    portal_roots(theme).find(|n| n.attrs.get("role").is_some_and(|v| v == role))
}

#[allow(dead_code)]
pub(super) fn find_portal_by_slot<'a>(
    theme: &'a WebGoldenTheme,
    slot: &str,
) -> Option<&'a WebNode> {
    portal_roots(theme).find(|n| n.attrs.get("data-slot").is_some_and(|v| v == slot))
}
