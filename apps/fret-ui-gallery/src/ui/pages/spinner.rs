use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::spinner as snippets;

pub(super) fn preview_spinner(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
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

    let api_reference = doc_layout::notes_block([
        "`Spinner::new()` mirrors the upstream leaf spinner with the default loader icon, intrinsic 16px box, and continuous spin.",
        "The default icon, current-color inheritance, size-4 box, and spin animation remain recipe-owned because the upstream component source defines those defaults on the spinner itself.",
        "Spinner now exposes status-style loading semantics (`role=status` equivalent with polite live-region behavior) instead of pretending to be a numeric progress bar.",
        "Upstream docs/source axes: `repo-ref/ui/apps/v4/content/docs/components/radix/spinner.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/spinner.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/spinner-{demo,custom,size,button,badge,input-group,empty}.tsx`, and `repo-ref/ui/apps/v4/examples/base/spinner-rtl.tsx`.",
        "Secondary shadcn headless references: `repo-ref/ui/apps/v4/registry/bases/radix/ui/spinner.tsx`, `repo-ref/ui/apps/v4/registry/bases/base/ui/spinner.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/examples/spinner-example.tsx`, and `repo-ref/ui/apps/v4/registry/bases/base/examples/spinner-example.tsx` all keep Spinner as a leaf `svg` recipe plus host-owned slot composition.",
        "Neither `repo-ref/primitives` nor `repo-ref/base-ui` defines a dedicated Spinner primitive, so this pass did not identify a missing `fret-ui` mechanism bug.",
        "`Button::leading_children(...)` / `trailing_children(...)` are the preferred Fret equivalent of the upstream `Spinner data-icon=\"inline-start|inline-end\"` composition story.",
        "Custom icon choice (`icon(...)`), explicit size (`refine_layout(...)`), and optional color (`color(...)`) remain caller-owned refinements; if your app wants a different default glyph, wrap `Spinner::new().icon(...)` in a local helper instead of widening the leaf API.",
        "`speed(...)` stays a focused Fret follow-up and is documented under `Extras`, not the upstream docs path.",
        "Button, badge, and input-group spacing stay owned by those host recipes rather than the spinner itself.",
        "Spinner is a visual leaf primitive; Button/Badge/InputGroup already cover composition through host-owned slots, so no extra generic `compose()` or composable children API is needed here.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-spinner-api-reference")
        .description("Public surface summary, source axes, and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default top-of-page preview matching the upstream spinner item example.")
        .test_id_prefix("ui-gallery-spinner-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `Spinner`.")
        .test_id_prefix("ui-gallery-spinner-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let customization = DocSection::build(cx, "Customization", customization)
        .description(
            "Build a small project-local wrapper when you want a different default spinner glyph.",
        )
        .test_id_prefix("ui-gallery-spinner-customization")
        .code_rust_from_file_region(snippets::customization::SOURCE, "example");
    let sizes = DocSection::build(cx, "Size", sizes)
        .description("Use explicit layout refinements for size-3 / 4 / 6 / 8 variants.")
        .test_id_prefix("ui-gallery-spinner-size")
        .code_rust_from_file_region(snippets::sizes::SOURCE, "example");
    let buttons = DocSection::build(cx, "Button", buttons)
        .description("Disabled buttons with inline-start spinner content.")
        .test_id_prefix("ui-gallery-spinner-button")
        .code_rust_from_file_region(snippets::buttons::SOURCE, "example");
    let badges = DocSection::build(cx, "Badge", badges)
        .description("Badge compositions where the surrounding recipe owns inline icon spacing through leading/trailing slots.")
        .test_id_prefix("ui-gallery-spinner-badge")
        .code_rust_from_file_region(snippets::badges::SOURCE, "example");
    let input_group = DocSection::build(cx, "Input Group", input_group)
        .description("Spinner inside trailing and block-end input-group regions.")
        .test_id_prefix("ui-gallery-spinner-input-group")
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let empty = DocSection::build(cx, "Empty", empty)
        .description("Empty-state surface with a spinner media slot.")
        .test_id_prefix("ui-gallery-spinner-empty")
        .code_rust_from_file_region(snippets::empty::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Spinner item layout under an RTL direction provider.")
        .test_id_prefix("ui-gallery-spinner-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
        .description("Fret-specific follow-ups such as speed control and extra icon variants.")
        .no_shell()
        .test_id_prefix("ui-gallery-spinner-extras")
        .code_rust_from_file_region(snippets::extras::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Spinner docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Customization`, `Size`, `Button`, `Badge`, `Input Group`, `Empty`, `RTL`, then keeps Fret-only `Extras` and `API Reference` as focused follow-ups.",
        ),
        vec![
            demo,
            usage,
            customization,
            sizes,
            buttons,
            badges,
            input_group,
            empty,
            rtl,
            extras,
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-spinner").into_element(cx)]
}
