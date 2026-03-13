use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

const MATERIAL3_BOTTOM_SHEET_INTRO: &str = "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code). Default copyable bottom-sheet demos use `ModalBottomSheet::uncontrolled(cx)` plus `open_model()` for underlay triggers; keep `ModalBottomSheet::new(open)` when the app owns open state.";
const MATERIAL3_DIALOG_INTRO: &str = "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code). Default copyable dialog demos use `Dialog::uncontrolled(cx)` plus `open_model()` for underlay triggers; keep `Dialog::new(open)` when the app owns open state.";
const MATERIAL3_MENU_INTRO: &str = "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code). Default copyable menu demos use `DropdownMenu::uncontrolled(cx)` plus `open_model()` for trigger wiring; keep `DropdownMenu::new(open)` for explicit externally owned open state.";

pub(in crate::ui) fn preview_material3_bottom_sheet(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::bottom_sheet::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_BOTTOM_SHEET_INTRO),
        demo,
        snippets::material3::bottom_sheet::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::dialog::render(cx, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_DIALOG_INTRO),
        demo,
        snippets::material3::dialog::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::menu::render(cx, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_MENU_INTRO),
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
