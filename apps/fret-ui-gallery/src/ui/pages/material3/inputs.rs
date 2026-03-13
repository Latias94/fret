use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_date_picker(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::date_picker::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::date_picker::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_time_picker(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::time_picker::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::time_picker::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_select(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::select::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::select::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_autocomplete(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::autocomplete::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::autocomplete::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_text_field(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::text_field::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::text_field::SOURCE,
    )
}
