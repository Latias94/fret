//! Accessibility helpers for the code editor surface.

use super::CodeEditorState;
use super::geom::RowFoldMap;
use super::paint::cached_row_text_with_range;
use fret_code_editor_buffer::TextBuffer;
use std::ops::Range;
use std::sync::Arc;

mod mapping;
mod window;

#[cfg(test)]
pub(super) use mapping::{map_a11y_offset_to_buffer, map_a11y_offset_to_buffer_with_preedit};
pub(super) use mapping::{
    map_a11y_offset_to_buffer_in_current_window, map_a11y_offsets_to_buffer_composed,
    map_buffer_offset_to_a11y_offset,
};
pub(super) use window::a11y_composed_text_window;
#[cfg(test)]
pub(super) use window::a11y_text_window_bounds;

#[cfg(test)]
mod tests {
    use super::super::{CodeEditorHandle, PreeditState, Selection};
    use super::a11y_composed_text_window;

    #[test]
    fn a11y_window_selection_preserves_direction_without_preedit() {
        let handle = CodeEditorHandle::new("hello world");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 8,
                focus: 3,
            };
        }

        let mut st = handle.state.borrow_mut();
        let (_value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
        assert_eq!(composition, None);
        assert_eq!(selection, Some((8, 3)));
    }

    #[test]
    fn a11y_window_selection_preserves_direction_for_preedit_cursor() {
        let handle = CodeEditorHandle::new("hello world");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 5,
                focus: 5,
            };
            st.preedit = Some(PreeditState {
                text: "yo".to_string(),
                cursor: Some((2, 0)),
            });
        }

        let mut st = handle.state.borrow_mut();
        let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
        assert_eq!(value, "helloyo world");
        assert_eq!(composition, Some((5, 7)));
        assert_eq!(selection, Some((7, 5)));
    }
}
