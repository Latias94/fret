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
mod tests {
    use super::*;

    #[test]
    fn apply_searcher_query_key_handles_ascii_and_backspace() {
        let mut query = String::from("ab");

        assert!(apply_searcher_query_key(
            &mut query,
            KeyCode::Backspace,
            Modifiers::default(),
        ));
        assert_eq!(query, "a");

        assert!(apply_searcher_query_key(
            &mut query,
            KeyCode::KeyC,
            Modifiers::default(),
        ));
        assert_eq!(query, "ac");

        assert!(!apply_searcher_query_key(
            &mut query,
            KeyCode::KeyV,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        ));
        assert_eq!(query, "ac");
    }
}
