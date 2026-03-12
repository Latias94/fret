pub(in super::super) fn allow_modifier_shortcut(modifiers: fret_core::Modifiers) -> bool {
    modifiers.ctrl || modifiers.meta
}

#[cfg(test)]
mod tests;
