use crate::element::SelectableTextState;
use crate::text_edit::{buffer, utf8};
use fret_runtime::TextBoundaryMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectableTextCommandOutcome {
    NotHandled,
    Handled {
        needs_repaint: bool,
        copy_range: Option<(usize, usize)>,
    },
}

fn line_start(text: &str, caret: usize) -> usize {
    let caret = utf8::clamp_to_char_boundary(text, caret);
    text[..caret]
        .rfind('\n')
        .map(|i| (i + 1).min(text.len()))
        .unwrap_or(0)
}

fn line_end(text: &str, caret: usize) -> usize {
    let caret = utf8::clamp_to_char_boundary(text, caret);
    text[caret..]
        .find('\n')
        .map(|i| (caret + i).min(text.len()))
        .unwrap_or(text.len())
}

pub(crate) fn apply_selectable_text_command(
    text: &str,
    state: &mut SelectableTextState,
    command: &str,
    boundary_mode: TextBoundaryMode,
) -> SelectableTextCommandOutcome {
    let command = match command {
        "edit.copy" => "text.copy",
        "edit.select_all" => "text.select_all",
        other => other,
    };

    state.caret = utf8::clamp_to_char_boundary(text, state.caret);
    state.selection_anchor = utf8::clamp_to_char_boundary(text, state.selection_anchor);
    state.dragging = false;

    let mut needs_repaint = false;
    let mut copy_range: Option<(usize, usize)> = None;

    match command {
        "text.copy" => {
            let (start, end) = buffer::selection_range(state.selection_anchor, state.caret);
            if start < end {
                copy_range = Some((start, end));
            }
        }
        "text.select_all" => {
            state.selection_anchor = 0;
            state.caret = text.len();
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_left" => {
            let next = utf8::prev_char_boundary(text, state.caret);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_right" => {
            let next = utf8::next_char_boundary(text, state.caret);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_left" => {
            state.caret = utf8::prev_char_boundary(text, state.caret);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_right" => {
            state.caret = utf8::next_char_boundary(text, state.caret);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_word_left" => {
            let next = utf8::move_word_left(text, state.caret, boundary_mode);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_word_right" => {
            let next = utf8::move_word_right(text, state.caret, boundary_mode);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_word_left" => {
            state.caret = utf8::move_word_left(text, state.caret, boundary_mode);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_word_right" => {
            state.caret = utf8::move_word_right(text, state.caret, boundary_mode);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_home" => {
            let next = line_start(text, state.caret);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.move_end" => {
            let next = line_end(text, state.caret);
            state.selection_anchor = next;
            state.caret = next;
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_home" => {
            state.caret = line_start(text, state.caret);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        "text.select_end" => {
            state.caret = line_end(text, state.caret);
            state.affinity = fret_core::CaretAffinity::Downstream;
            needs_repaint = true;
        }
        _ => return SelectableTextCommandOutcome::NotHandled,
    }

    SelectableTextCommandOutcome::Handled {
        needs_repaint,
        copy_range,
    }
}
