use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut UiCx<'_>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let _ = last_action;

    let demo = snippets::material3::gallery::render(
        cx,
        material3_text_field_disabled,
        material3_text_field_error,
    );

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::gallery::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_state_matrix(
    cx: &mut UiCx<'_>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::state_matrix::render(
        cx,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        last_action,
    );

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::state_matrix::SOURCE,
    )
}
