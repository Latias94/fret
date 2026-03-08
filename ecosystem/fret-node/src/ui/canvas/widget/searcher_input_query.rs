use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use super::*;

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

pub(super) fn try_activate_active_searcher_row<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    let Some(row_ix) = canvas
        .interaction
        .searcher
        .as_ref()
        .map(|searcher| searcher.active_row)
    else {
        return false;
    };
    canvas.try_activate_searcher_row(cx, row_ix)
}

pub(super) fn update_searcher_query_from_key<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    if !apply_searcher_query_key(&mut searcher.query, key, modifiers) {
        return false;
    }

    NodeGraphCanvasWith::<M>::rebuild_searcher_rows(searcher);
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
