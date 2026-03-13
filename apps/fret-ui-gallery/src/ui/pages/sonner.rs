use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sonner as snippets;
use fret::UiCx;

pub(super) fn preview_sonner(
    cx: &mut UiCx<'_>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> Vec<AnyElement> {
    let setup = snippets::setup::render(cx);
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx, last_action.clone());
    let position = snippets::position::render(cx, last_action.clone(), sonner_position);
    let extras = snippets::extras::render(cx, last_action.clone());
    let notes = snippets::notes::render(cx, last_action);
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Status + parity notes.")
        .test_id_prefix("ui-gallery-sonner-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Sonner docs flow: Setup -> Usage -> Demo, with Fret-specific position and extras sections for overlay coverage.",
        ),
        vec![
            DocSection::new("Setup", setup)
                .description("Mount a toaster layer in your window root.")
                .test_id_prefix("ui-gallery-sonner-setup")
                .code_rust_from_file_region(snippets::setup::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description(
                    "Copyable minimal usage for firing a toast through the global Sonner facade.",
                )
                .test_id_prefix("ui-gallery-sonner-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Demo", demo)
                .description("Buttons that fire different toast styles and actions.")
                .test_id_prefix("ui-gallery-sonner-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Position", position)
                .description("Fret-specific: mutate global toaster position for overlay testing.")
                .test_id_prefix("ui-gallery-sonner-position")
                .code_rust_from_file_region(snippets::position::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .description("Pinned + swipe-dismiss toasts.")
                .test_id_prefix("ui-gallery-sonner-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-sonner")]
}
