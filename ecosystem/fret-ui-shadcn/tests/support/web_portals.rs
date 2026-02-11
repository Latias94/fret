use super::*;

pub(super) fn portal_roots<'a>(
    theme: &'a WebGoldenTheme,
) -> impl Iterator<Item = &'a WebNode> + 'a {
    theme.portals.iter().chain(theme.portal_wrappers.iter())
}
