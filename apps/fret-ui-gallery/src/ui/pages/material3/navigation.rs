use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_top_app_bar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::top_app_bar::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::top_app_bar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut UiCx<'_>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::tabs::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::tabs::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_list(
    cx: &mut UiCx<'_>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::list::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::list::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut UiCx<'_>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_bar::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_bar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut UiCx<'_>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_rail::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_rail::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut UiCx<'_>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_drawer::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_drawer::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut UiCx<'_>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::modal_navigation_drawer::render(cx, open, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::modal_navigation_drawer::SOURCE,
    )
}
