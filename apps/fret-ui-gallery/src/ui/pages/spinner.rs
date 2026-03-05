use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::spinner as snippets;

pub(super) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let customization = snippets::customization::render(cx);
    let sizes = snippets::sizes::render(cx);
    let buttons = snippets::buttons::render(cx);
    let badges = snippets::badges::render(cx);
    let input_group = snippets::input_group::render(cx);
    let empty = snippets::empty::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Spinner docs/examples (new-york-v4): Demo, Customization, Size, Button, Badge, Input Group, Empty, RTL.",
            "Extras are Fret-specific demos and regression gates.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("An indicator that can be used to show a loading state."),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-spinner-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Customization", customization)
                .test_id_prefix("ui-gallery-spinner-customization")
                .code_rust_from_file_region(snippets::customization::SOURCE, "example"),
            DocSection::new("Sizes", sizes)
                .test_id_prefix("ui-gallery-spinner-sizes")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("Buttons", buttons)
                .test_id_prefix("ui-gallery-spinner-buttons")
                .code_rust_from_file_region(snippets::buttons::SOURCE, "example"),
            DocSection::new("Badges", badges)
                .test_id_prefix("ui-gallery-spinner-badges")
                .code_rust_from_file_region(snippets::badges::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .test_id_prefix("ui-gallery-spinner-input-group")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("Empty", empty)
                .test_id_prefix("ui-gallery-spinner-empty")
                .code_rust_from_file_region(snippets::empty::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-spinner-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-spinner")]
}
