use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::button as snippets;

pub(super) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let variants = snippets::variants::render(cx);
    let link_render = snippets::link_render::render(cx);
    let size = snippets::size::render(cx);
    let icon_only = snippets::icon::render(cx);
    let with_icon = snippets::with_icon::render(cx);
    let loading = snippets::loading::render(cx);
    let rounded = snippets::rounded::render(cx);
    let button_group = snippets::button_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview aims to match shadcn Button docs order so you can visually compare variants quickly.",
            "Prefer icon-only buttons to use explicit `ButtonSize::Icon*` to keep square chrome.",
            "For long-running actions, combine a disabled button with a spinner + label.",
            "Use `ButtonRender::Link` when you want link semantics (`role=link`, Enter-only activation) on the pressable root.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Button docs (plus a compact variants row and a deterministic link render example).",
        ),
        vec![
            DocSection::new("Variants", variants)
                .description("Default shadcn button variants.")
                .code_rust_from_file_region(
                    include_str!("../snippets/button/variants.rs"),
                    "example",
                ),
            DocSection::new("Link (render)", link_render)
                .description(
                    "Render the button with link semantics (shadcn `asChild`-style composition).",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/button/link_render.rs"),
                    "example",
                ),
            DocSection::new("Size", size)
                .description("Text and icon-only sizes.")
                .code_rust_from_file_region(include_str!("../snippets/button/size.rs"), "example"),
            DocSection::new("Icon", icon_only)
                .description("Icon-only buttons.")
                .code_rust_from_file_region(include_str!("../snippets/button/icon.rs"), "example"),
            DocSection::new("With Icon", with_icon)
                .description("Compose an icon + text label.")
                .code_rust_from_file_region(
                    include_str!("../snippets/button/with_icon.rs"),
                    "example",
                ),
            DocSection::new("Loading", loading)
                .description("Spinner + label for in-flight actions.")
                .code_rust_from_file_region(
                    include_str!("../snippets/button/loading.rs"),
                    "example",
                ),
            DocSection::new("Rounded", rounded)
                .description("Use a fully-rounded chrome for pill-shaped buttons.")
                .code_rust_from_file_region(
                    include_str!("../snippets/button/rounded.rs"),
                    "example",
                ),
            DocSection::new("Button Group", button_group)
                .description("A grouped set of buttons with shared borders and radii.")
                .code_rust_from_file_region(
                    include_str!("../snippets/button/button_group.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Button layout should work under an RTL direction provider.")
                .code_rust_from_file_region(include_str!("../snippets/button/rtl.rs"), "example"),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-button")]
}
