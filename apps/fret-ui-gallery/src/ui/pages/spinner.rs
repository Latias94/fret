use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::spinner as snippets;

pub(super) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let customization = snippets::customization::render(cx);
    let sizes = snippets::sizes::render(cx);
    let buttons = snippets::buttons::render(cx);
    let badges = snippets::badges::render(cx);
    let input_group = snippets::input_group::render(cx);
    let empty = snippets::empty::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Spinner::new()` mirrors the upstream leaf spinner with the default loader icon, intrinsic 16px box, and continuous spin.",
            "The default icon, current-color inheritance, size-4 box, and spin animation remain recipe-owned because the upstream component source defines those defaults on the spinner itself.",
            "Custom icon choice (`icon(...)`), explicit size (`refine_layout(...)`), and optional color (`color(...)`) remain caller-owned refinements; `speed(...)` stays a focused Fret follow-up and is documented under `Extras`, not the upstream docs path.",
            "Button, badge, and input-group spacing stay owned by those host recipes rather than the spinner itself.",
            "Spinner is a visual leaf primitive, so no generic `compose()` / children API is needed here.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Spinner docs path first: Demo, Usage, Customization, Size, Button, Badge, Input Group, Empty, RTL, then keeps `Extras` and `API Reference` as focused Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Default top-of-page preview matching the upstream spinner item example.",
                )
                .test_id_prefix("ui-gallery-spinner-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Spinner`.")
                .test_id_prefix("ui-gallery-spinner-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Customization", customization)
                .description(
                    "Swap the default loader icon while keeping the same leaf component contract.",
                )
                .test_id_prefix("ui-gallery-spinner-customization")
                .code_rust_from_file_region(snippets::customization::SOURCE, "example"),
            DocSection::new("Size", sizes)
                .description("Use explicit layout refinements for size-3 / 4 / 6 / 8 variants.")
                .test_id_prefix("ui-gallery-spinner-size")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("Button", buttons)
                .description("Disabled buttons with inline-start spinner content.")
                .test_id_prefix("ui-gallery-spinner-button")
                .code_rust_from_file_region(snippets::buttons::SOURCE, "example"),
            DocSection::new("Badge", badges)
                .description(
                    "Badge compositions where the surrounding recipe owns inline icon spacing.",
                )
                .test_id_prefix("ui-gallery-spinner-badge")
                .code_rust_from_file_region(snippets::badges::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .description("Spinner inside trailing and block-end input-group regions.")
                .test_id_prefix("ui-gallery-spinner-input-group")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("Empty", empty)
                .description("Empty-state surface with a spinner media slot.")
                .test_id_prefix("ui-gallery-spinner-empty")
                .code_rust_from_file_region(snippets::empty::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Spinner item layout under an RTL direction provider.")
                .test_id_prefix("ui-gallery-spinner-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .description(
                    "Fret-specific follow-ups such as speed control and extra icon variants.",
                )
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-api-reference")
                .description("Public surface summary and ownership notes."),
        ],
    );

    vec![body.test_id("ui-gallery-spinner")]
}
