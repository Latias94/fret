use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sonner as snippets;
use fret::UiCx;

pub(super) fn preview_sonner(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let setup = snippets::setup::render(cx);
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx);
    let position = snippets::position::render(cx);
    let extras = snippets::extras::render(cx);
    let notes = snippets::notes::render(cx);
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Status + parity notes.")
        .test_id_prefix("ui-gallery-sonner-notes");
    let setup = DocSection::build(cx, "Setup", setup)
        .description("Mount a toaster layer in your window root.")
        .test_id_prefix("ui-gallery-sonner-setup")
        .code_rust_from_file_region(snippets::setup::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for firing a toast through the Sonner facade.")
        .test_id_prefix("ui-gallery-sonner-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Buttons that fire different toast styles and actions.")
        .test_id_prefix("ui-gallery-sonner-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let position = DocSection::build(cx, "Position", position)
        .description("Fret-specific: mutate the local toaster position for overlay testing.")
        .test_id_prefix("ui-gallery-sonner-position")
        .code_rust_from_file_region(snippets::position::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
        .description("Pinned + swipe-dismiss toasts.")
        .test_id_prefix("ui-gallery-sonner-extras")
        .code_rust_from_file_region(snippets::extras::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Sonner docs flow: Setup -> Usage -> Demo, with Fret-specific position and extras sections for overlay coverage.",
        ),
        vec![setup, usage, demo, position, extras, notes],
    );
    let toaster = snippets::local_toaster(cx).into_element(cx);

    vec![body.test_id("ui-gallery-sonner").into_element(cx), toaster]
}
