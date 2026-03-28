use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::item as snippets;

pub(super) fn preview_item(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let variants = snippets::variants::render(cx);
    let size = snippets::size::render(cx);
    let icon = snippets::icon::render(cx);
    let avatar = snippets::avatar::render(cx);
    let image = snippets::image::render(cx);
    let group = snippets::group::render(cx);
    let header = snippets::header::render(cx);
    let link = snippets::link::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let rtl = snippets::extras_rtl::render(cx);
    let gallery = snippets::gallery::render(cx);
    let link_render = snippets::link_render::render(cx);

    let item_vs_field = doc_layout::muted_full_width(
        cx,
        "Use `Field` when the row owns a form control such as a checkbox, input, radio, or select. Use `Item` when the row only presents media, title, description, and actions.",
    );
    let examples = doc_layout::notes_block([
        "Gallery collapses the upstream top-of-page `ComponentPreview` into `Demo` and skips `Installation`, because the UI Gallery teaches live Rust surfaces rather than package-install steps.",
        "The upstream `Examples` group stays explicit here so `Icon`, `Avatar`, `Image`, `Group`, `Header`, `Link`, and `Dropdown` remain easy to compare one-to-one with the docs page.",
        "`RTL` remains a separate top-level docs section after `Examples`, matching the upstream page structure instead of being folded into the example group.",
    ]);

    let api_reference = doc_layout::notes_block([
        "`Item::new([...])` plus `ItemMedia`, `ItemContent`, `ItemTitle`, `ItemDescription`, `ItemActions`, `ItemGroup`, and `ItemHeader` matches the upstream slot model directly, while `item_sized(...)` and `item_group(...)` are the preferred thin helpers on first-party teaching surfaces.",
        "`ItemRender::Link` is the Fret equivalent of the upstream `render={<a ... />}` pattern and keeps link semantics on the pressable root rather than burying them in a nested child.",
        "`Item::new([...])` already is the composable root-children lane for heterogeneous slot parts, while `ItemTitle::new_children([...])` and `ItemDescription::new_children([...])` keep slot-local rich child composition available for styled text or mixed inline content without widening the root API further.",
        "`ItemMediaVariant` intentionally stays `default | icon | image`; avatar rows are composed by placing `Avatar` inside `ItemMedia` rather than adding an extra recipe variant that the upstream source does not define.",
        "Intrinsic item chrome, slot spacing, and size presets remain recipe-owned because the upstream component source defines those defaults on the item itself.",
        "Caller-owned layout stays explicit for `max-w-*`, grid placement, page columns, and broader list composition. The recipe should not absorb those negotiation rules.",
        "`ItemGroup` owns the list container semantics (`role=list`); per-row `listitem` semantics stay caller-owned when a row also needs a stronger interactive role such as `link`, which matches the current upstream source shape better than forcing a recipe default.",
        "No extra generic `asChild` / `compose()` surface is needed here: the existing slot parts and `ItemRender::Link` already cover the documented composition model.",
        "`ItemSize::Xs` is already supported in Fret and is now shown explicitly in the gallery size example.",
    ]);
    let examples = DocSection::build(cx, "Examples", examples)
        .no_shell()
        .test_id_prefix("ui-gallery-item-examples")
        .description("How the upstream `Examples` group maps onto the preview sections below.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-item-api-reference")
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .no_shell()
        .test_id_prefix("ui-gallery-item-demo-section")
        .description("Top-of-page item preview matching the upstream docs intent.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .no_shell()
        .title_test_id("ui-gallery-item-section-title-usage")
        .test_id_prefix("ui-gallery-item-usage-section")
        .description("Copyable minimal usage for `Item` and its slot parts.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let item_vs_field = DocSection::build(cx, "Item vs Field", item_vs_field)
        .no_shell()
        .test_id_prefix("ui-gallery-item-vs-field")
        .description("Choose `Item` for presentation rows and `Field` for input-bearing rows.");
    let variants = DocSection::build(cx, "Variant", variants)
        .description("Default, outline, and muted variants.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-variant-section")
        .code_rust_from_file_region(snippets::variants::SOURCE, "example");
    let size = DocSection::build(cx, "Size", size)
        .description("`default`, `sm`, and `xs` item sizing; the size-scoped gallery lane uses `item_sized(...)`.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-size-section")
        .code_rust_from_file_region(snippets::size::SOURCE, "example");
    let icon = DocSection::build(cx, "Icon", icon)
        .no_shell()
        .test_id_prefix("ui-gallery-item-icon-section")
        .code_rust_from_file_region(snippets::icon::SOURCE, "example");
    let avatar = DocSection::build(cx, "Avatar", avatar)
        .no_shell()
        .test_id_prefix("ui-gallery-item-avatar-section")
        .description("Compose `Avatar` inside `ItemMedia`; the upstream docs prose names an avatar lane, but the registry source keeps avatar rows as slot composition rather than a dedicated `variant`.")
        .code_rust_from_file_region(snippets::avatar::SOURCE, "example");
    let image = DocSection::build(cx, "Image", image)
        .no_shell()
        .test_id_prefix("ui-gallery-item-image-section")
        .code_rust_from_file_region(snippets::image::SOURCE, "example");
    let group = DocSection::build(cx, "Group", group)
        .description("Grouped item rows using the thin `item_group(...)` helper.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-group-section")
        .code_rust_from_file_region(snippets::group::SOURCE, "example");
    let header = DocSection::build(cx, "Header", header)
        .no_shell()
        .test_id_prefix("ui-gallery-item-header-section")
        .code_rust_from_file_region(snippets::header::SOURCE, "example");
    let link = DocSection::build(cx, "Link", link)
        .description("Links are modeled via `ItemRender::Link` so the root carries link semantics.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-link-section")
        .code_rust_from_file_region(snippets::link::SOURCE, "example");
    let dropdown = DocSection::build(cx, "Dropdown", dropdown)
        .description("Item composed inside a dropdown menu row.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-dropdown-section")
        .code_rust_from_file_region(snippets::dropdown::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL smoke check for the item recipe and slot ordering.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-rtl-section")
        .code_rust_from_file_region(snippets::extras_rtl::SOURCE, "example");
    let gallery = DocSection::build(cx, "Gallery", gallery)
        .description("Extended regression coverage snapshot: columns plus mixed compositions.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-gallery-section")
        .max_w(Px(1100.0));
    let link_render = DocSection::build(cx, "Link (render)", link_render)
        .description("A gallery-focused deterministic link row example.")
        .no_shell()
        .test_id_prefix("ui-gallery-item-link-render-section")
        .code_rust_from_file_region(snippets::link_render::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Item docs path first after collapsing `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Item vs Field`, `Variant`, `Size`, `Examples`, the example set through `RTL`, and `API Reference`. After that, `Gallery` and `Link (render)` remain explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            item_vs_field,
            variants,
            size,
            examples,
            icon,
            avatar,
            image,
            group,
            header,
            link,
            dropdown,
            rtl,
            api_reference,
            gallery,
            link_render,
        ],
    );

    vec![body.test_id("ui-gallery-item").into_element(cx)]
}
