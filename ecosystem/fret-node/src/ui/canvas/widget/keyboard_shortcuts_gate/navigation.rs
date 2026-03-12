pub(in super::super) fn allow_plain_tab_navigation(
    disable_keyboard_a11y: bool,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    !disable_keyboard_a11y
        && key == fret_core::KeyCode::Tab
        && !modifiers.ctrl
        && !modifiers.meta
        && !modifiers.alt
        && !modifiers.alt_gr
}

pub(in super::super) fn allow_arrow_nudging(
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::super::keyboard_shortcuts_map::is_arrow_key(key)
        && !modifiers.ctrl
        && !modifiers.meta
        && !modifiers.alt
        && !modifiers.alt_gr
}

#[cfg(test)]
mod tests;
