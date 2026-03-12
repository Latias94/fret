use crate::io::NodeGraphDeleteKey;

pub(in super::super) fn matches_delete_shortcut(
    delete_key: NodeGraphDeleteKey,
    key: fret_core::KeyCode,
) -> bool {
    delete_key.matches(key)
}

#[cfg(test)]
mod tests;
