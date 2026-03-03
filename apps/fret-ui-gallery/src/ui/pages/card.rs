use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::card as snippets;

pub(super) fn preview_card(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let login = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let size = snippets::size::render(cx);
    let card_content_inline_button = snippets::card_content::render(cx);
    let meeting_notes = snippets::meeting_notes::render(cx);
    let image = snippets::image::render(cx, event_cover_image);
    let rtl = snippets::rtl::render(cx);
    let compositions = snippets::compositions::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Card provides structure (header/content/footer) but leaves layout decisions to composition.",
            "Prefer consistent max widths for card-based forms to avoid layout jumps across pages.",
            "MediaImage demos use `ImageSourceElementContextExt` to resolve local/URL image sources into `ImageId`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Card docs core flow (Demo + Usage), plus extra regression-focused sections (Size, Image, RTL).",
        ),
        vec![
            DocSection::new("Demo", login)
                .no_shell()
                .max_w(Px(980.0))
                .description(
                    "Login card layout (CardHeader + CardAction + CardContent + CardFooter).",
                )
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .no_shell()
                .max_w(Px(980.0))
                .description(
                    "Basic structure (header/content/footer) with an optional action slot.",
                )
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Size", size)
                .no_shell()
                .max_w(Px(980.0))
                .description("Use `CardSize::Sm` for a more compact spacing preset.")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("CardContent", card_content_inline_button)
                .no_shell()
                .max_w(Px(980.0))
                .description("CardContent should preserve intrinsic sizes for inline children.")
                .code_rust_from_file_region(snippets::card_content::SOURCE, "example"),
            DocSection::new("Meeting Notes", meeting_notes)
                .no_shell()
                .max_w(Px(980.0))
                .description("Card with text content and a footer stack.")
                .code_rust_from_file_region(snippets::meeting_notes::SOURCE, "example"),
            DocSection::new("Image", image)
                .no_shell()
                .max_w(Px(980.0))
                .description("Card with a media cover and a richer footer row.")
                .code_rust_from_file_region(snippets::image::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .no_shell()
                .max_w(Px(980.0))
                .description("Card should respect right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Compositions", compositions)
                .no_shell()
                .max_w(Px(980.0))
                .description("Spot-check slot combinations: header/content/footer permutations.")
                .code_rust_from_file_region(snippets::compositions::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Implementation notes and pointers."),
        ],
    );

    vec![body.test_id("ui-gallery-card")]
}
