use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle_group as snippets;

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let outline = snippets::outline::render(cx);
    let full_width_items = snippets::full_width_items::render(cx);
    let stretch = snippets::flex_1_items::render(cx);
    let size = snippets::size::render(cx);
    let spacing = snippets::spacing::render(cx);
    let vertical = snippets::vertical::render(cx);
    let disabled = snippets::disabled::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/toggle_group.rs` and `ecosystem/fret-ui-shadcn/src/toggle.rs`.",
            "Use Single mode for mutually-exclusive options (alignment, list/grid/cards).",
            "Use Multiple mode for formatting toggles where users may combine states.",
            "`spacing` is useful when each item needs stronger visual separation.",
            "For icon-only groups, keep explicit `a11y_label` for assistive technologies.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Toggle Group docs order: Demo, Outline, Size, Spacing, Vertical, Disabled, RTL (plus a flex-1 regression gate section).",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Multiple selection with icon-only items.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Text items with outline chrome.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("Full Width Items", full_width_items)
                .description("Stretched items (flex-1) to gate control-chrome fill invariants.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::full_width_items::SOURCE, "example"),
            DocSection::new("Flex-1 items", stretch)
                .description("Regression gate for hit/visual alignment under `flex-1` sizing.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example"),
            DocSection::new("Size", size)
                .description("Size presets for toolbar density.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Spacing", spacing)
                .description("Explicit spacing between items to reduce mis-clicks.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::spacing::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation for side panels / inspectors.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled groups keep layout but block interaction.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Item ordering and pressed visuals under RTL.")
                .max_w(Px(560.0))
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-toggle-group")]
}
