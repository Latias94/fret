use fret_core::{KeyCode, Modifiers};

pub(super) fn apply_searcher_query_key(
    query: &mut String,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    match key {
        KeyCode::Backspace => {
            if query.is_empty() {
                return false;
            }
            query.pop();
            return true;
        }
        _ => {}
    }

    if modifiers.ctrl || modifiers.meta {
        return false;
    }

    let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) else {
        return false;
    };
    query.push(ch);
    true
}

#[cfg(test)]
mod tests;
