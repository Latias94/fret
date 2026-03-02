use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::item as snippets;

pub(super) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let docs_demo = snippets::demo::render(cx);
    let docs_variants = snippets::variants::render(cx);
    let docs_size = snippets::size::render(cx);
    let docs_icon = snippets::icon::render(cx);
    let docs_avatar = snippets::avatar::render(cx);
    let docs_image = snippets::image::render(cx);
    let docs_group = snippets::group::render(cx);
    let docs_header = snippets::header::render(cx);
    let docs_link = snippets::link::render(cx);
    let docs_dropdown = snippets::dropdown::render(cx);
    let gallery_demo = snippets::gallery::render(cx);
    let link_render = snippets::link_render::render(cx);
    let rtl = snippets::extras_rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Docs sections align to shadcn Item examples (new-york-v4).",
            "The Gallery section is an extended snapshot used for internal regression coverage.",
            "Upstream uses `render={<a .../>}`; Fret uses `ItemRender::Link` to express link semantics on the pressable root.",
            "API reference: `ecosystem/fret-ui-shadcn/src/item.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Item docs (new-york-v4) with a few Fret-specific extras."),
        vec![
            DocSection::new("Demo", docs_demo)
                .no_shell()
                .max_w(Px(720.0))
                .code_rust_from_file_region(include_str!("../snippets/item/demo.rs"), "example"),
            DocSection::new("Variants", docs_variants)
                .description("Default, Outline, and Muted variants (new-york-v4).")
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/item/variants.rs"),
                    "example",
                ),
            DocSection::new("Size", docs_size)
                .description("Default vs `sm` item sizing (new-york-v4).")
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(include_str!("../snippets/item/size.rs"), "example"),
            DocSection::new("Icon", docs_icon)
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(include_str!("../snippets/item/icon.rs"), "example"),
            DocSection::new("Avatar", docs_avatar)
                .no_shell()
                .max_w(Px(720.0))
                .code_rust_from_file_region(include_str!("../snippets/item/avatar.rs"), "example"),
            DocSection::new("Image", docs_image)
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(include_str!("../snippets/item/image.rs"), "example"),
            DocSection::new("Group", docs_group)
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(include_str!("../snippets/item/group.rs"), "example"),
            DocSection::new("Header", docs_header)
                .no_shell()
                .max_w(Px(820.0))
                .code_rust_from_file_region(include_str!("../snippets/item/header.rs"), "example"),
            DocSection::new("Link", docs_link)
                .description(
                    "Links are modeled via `ItemRender::Link` so the root carries link semantics.",
                )
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(include_str!("../snippets/item/link.rs"), "example"),
            DocSection::new("Dropdown", docs_dropdown)
                .description("Item composed inside a DropdownMenu row (new-york-v4).")
                .no_shell()
                .max_w(Px(720.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/item/dropdown.rs"),
                    "example",
                ),
            DocSection::new("Gallery", gallery_demo)
                .description("Extended coverage snapshot: columns + mixed compositions.")
                .no_shell()
                .max_w(Px(1100.0)),
            DocSection::new("Link (render)", link_render)
                .description(
                    "Minimal link row with media + chevron (gallery-friendly, deterministic).",
                )
                .no_shell()
                .max_w(Px(640.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/item/link_render.rs"),
                    "example",
                ),
            DocSection::new("Extras", rtl)
                .description("RTL smoke check (not present in upstream demo).")
                .no_shell()
                .max_w(Px(980.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/item/extras_rtl.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-item")]
}
