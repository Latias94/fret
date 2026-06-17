//! Editor-owned text assist field recipe.
//!
//! This sits above:
//! - `fret-ui-headless::text_assist` query/filter/navigation math,
//! - `fret-ui-kit::headless::text_assist` input-owned semantics + key policy glue,
//! - and below any app-local completion/history domain logic.
//!
//! Current scope:
//! - one owning `TextField`,
//! - shared listbox rendering for inline and anchored overlay surfaces,
//! - input-owned focus with `active_descendant`,
//! - default accept wiring that commits the chosen label back into the bound query model.

mod accept;
mod element;
mod empty;
mod model;
mod overlay;
mod panel;
#[cfg(test)]
mod tests;

use fret_core::Px;
use fret_ui_kit::headless::text_assist::input_owned_text_assist_expanded;

use crate::primitives::popup_list::editor_popup_list_default_max_content_height;

pub use element::TextAssistField;
pub use model::{OnTextAssistFieldAccept, TextAssistFieldOptions, TextAssistFieldSurface};

fn should_render_inline_empty_label(
    surface: TextAssistFieldSurface,
    query: &str,
    visible_count: usize,
) -> bool {
    matches!(surface, TextAssistFieldSurface::Inline)
        && !query.trim().is_empty()
        && visible_count == 0
}

fn text_assist_max_content_height(
    surface: TextAssistFieldSurface,
    max_list_height: Option<Px>,
    row_height: Px,
) -> Option<Px> {
    max_list_height.or_else(|| {
        matches!(surface, TextAssistFieldSurface::AnchoredOverlay)
            .then(|| editor_popup_list_default_max_content_height(row_height))
    })
}

fn text_assist_field_expanded(
    surface: TextAssistFieldSurface,
    query: &str,
    dismissed_query: &str,
    visible_count: usize,
    input_focused: bool,
) -> bool {
    let query_expanded = input_owned_text_assist_expanded(query, dismissed_query, visible_count);
    match surface {
        TextAssistFieldSurface::Inline => query_expanded,
        TextAssistFieldSurface::AnchoredOverlay => input_focused && query_expanded,
    }
}

fn should_clear_text_assist_dismissal_on_focus_gain(
    query: &str,
    dismissed_query: &str,
    visible_count: usize,
    was_focused: bool,
    is_focused: bool,
) -> bool {
    is_focused
        && !was_focused
        && !query.trim().is_empty()
        && query == dismissed_query
        && visible_count > 0
}
