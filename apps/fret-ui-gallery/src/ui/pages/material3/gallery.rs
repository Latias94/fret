use super::*;
use fret::AppComponentCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut AppComponentCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let _ = last_action;

    let demo = snippets::material3::gallery::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::gallery::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_state_matrix(
    cx: &mut AppComponentCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::state_matrix::render(cx, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::state_matrix::SOURCE,
    )
}
