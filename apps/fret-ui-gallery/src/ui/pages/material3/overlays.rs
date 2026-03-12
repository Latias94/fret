use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_bottom_sheet(
    cx: &mut UiCx<'_>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::bottom_sheet::render(cx, open);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::bottom_sheet::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut UiCx<'_>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::dialog::render(cx, open, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::dialog::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut UiCx<'_>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::menu::render(cx, open, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::menu::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::snackbar::render(cx, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::snackbar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_tooltip(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::tooltip::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::tooltip::SOURCE,
    )
}
