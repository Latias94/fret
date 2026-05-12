//! Input, editing, and command handling for the code editor surface.

use super::*;

mod clipboard;
mod edit;
mod keyboard;
mod navigation;
mod pointer;

pub(super) use clipboard::{copy_selection, cut_selection, request_paste};
pub(super) use edit::{
    apply_and_record_edit, apply_ime_delete_surrounding, delete_backward, delete_forward,
    delete_word_backward, delete_word_forward, insert_text, insert_text_with_kind, redo, undo,
};
pub(super) use keyboard::{command_availability, handle_key_down};
pub(super) use navigation::{
    clamp_selection_out_of_folds, move_caret_home_end, move_caret_left, move_caret_page,
    move_caret_right, move_caret_vertical, move_word, scroll_caret_into_view,
};
pub(super) use pointer::apply_pointer_down_selection;
