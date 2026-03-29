use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::button as snippets;

pub(super) fn preview_button(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let size = snippets::size::render(cx);
    let default = snippets::default::render(cx);
    let outline = snippets::outline::render(cx);
    let secondary = snippets::secondary::render(cx);
    let ghost = snippets::ghost::render(cx);
    let destructive = snippets::destructive::render(cx);
    let link = snippets::link::render(cx);
    let icon_only = snippets::icon::render(cx);
    let with_icon = snippets::with_icon::render(cx);
    let rounded = snippets::rounded::render(cx);
    let spinner = snippets::loading::render(cx);
    let button_group = snippets::button_group::render(cx);
    let link_render = snippets::link_render::render(cx);
    let rtl = snippets::rtl::render(cx);
    let children = snippets::children::render(cx);
    let variants = snippets::variants::render(cx);

    let cursor = doc_layout::notes_block([
        "The upstream cursor note is Tailwind CSS-specific (`cursor: default` vs `cursor: pointer`).",
        "Fret is self-drawn, so this exact CSS footgun does not apply one-to-one to the button recipe.",
        "If host-specific pointer cursor behavior needs adjustment, treat it as runtime / pressable policy, not a `Button` default-style change.",
    ]);

    let api_reference = doc_layout::notes_block([
        "`Button::new(label)` plus `variant(...)` covers the documented `default`, `outline`, `secondary`, `ghost`, `destructive`, and `link` recipe surface.",
        "`size(...)` covers `default`, `xs`, `sm`, `lg`, `icon`, `icon-xs`, `icon-sm`, and `icon-lg`.",
        "`leading_child(...)` / `trailing_child(...)` are the ergonomic single-node variants for the upstream `data-icon=\"inline-start|inline-end\"` child-composition path.",
        "`leading_children(...)` / `trailing_children(...)` remain available when you need multiple landed nodes on one side.",
        "`child(...)` / `children(...)` are the explicit full content override when you want to replace the entire inner row on purpose.",
        "`ButtonRender::Link` is the shared Fret mapping for the Base UI docs' `As Link` section and the Radix docs' `As Child` link example, so semantic link rendering stays button-owned instead of widening the public surface with a generic root `asChild` / `compose()` API.",
        "No extra generic root `asChild` / composable children API is currently warranted: `leading_child(...)` / `trailing_child(...)` already cover the documented inline icon/spinner lane, `child(...)` / `children(...)` keep the full-row override explicit, and `ButtonRender::Link` covers the semantics-sensitive link escape hatch.",
        "Intrinsic chrome stays recipe-owned; page-level width, wrapping, `flex-1`, `min-w-0`, and fully-rounded one-off examples stay caller-owned refinements.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/button.rs`.",
        "Visual chrome stays aligned to the current `new-york-v4` button recipe, while the docs section order follows the published Base / Radix Button pages.",
        "Gallery sections now mirror shadcn Button docs first: Demo, Usage, Cursor, Size, Default, Outline, Secondary, Ghost, Destructive, Link, Icon, With Icon, Rounded, Spinner, Button Group, As Link / As Child (Semantic), RTL, API Reference.",
        "`Children (Fret)` stays after the upstream path to document the landed-element equivalent of JSX child composition without widening `Button` into a generic root `asChild` surface.",
        "`Variants Overview (Fret)` stays after the upstream path so existing variant chrome diagnostics remain easy to compare without displacing the docs order.",
        "The `RTL` preview keeps the translated upstream row shape and makes the logical slot contract explicit: `trailing_icon(...)` still means inline-end and `leading_child(...)` still means inline-start, so the visual order flips automatically under `DirectionProvider(Rtl)`.",
        "Icon glyph direction remains caller-owned rather than recipe-owned. The RTL submit example uses `lucide.arrow-left` to match the upstream web example's `ArrowRightIcon` plus `rtl:rotate-180` outcome without introducing a button-specific auto-mirror rule.",
        "The main parity fix here is recipe/public-surface work: logical inline child slots now cover spinner/icon compositions, the semantic-link lane now bridges both the Base `As Link` and Radix `As Child` docs surfaces, and the page now stamps stable button-scoped section ids so docs smoke gates can target the real page structure.",
    ]);

    let cursor = DocSection::build(cx, "Cursor", cursor)
        .no_shell()
        .test_id_prefix("ui-gallery-button-cursor")
        .description("Translate the upstream Tailwind cursor note into Fret ownership terms.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-button-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-button-notes")
        .description("Parity notes and implementation pointers.");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-button-demo")
        .description("Docs-aligned outline button + icon button preview.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .test_id_prefix("ui-gallery-button-usage")
        .description("Copyable minimal usage for `Button`.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let size = DocSection::build(cx, "Size", size)
        .test_id_prefix("ui-gallery-button-size")
        .description("Use `size(...)` to change text and icon button sizes.")
        .code_rust_from_file_region(snippets::size::SOURCE, "example");
    let default = DocSection::build(cx, "Default", default)
        .test_id_prefix("ui-gallery-button-default")
        .code_rust_from_file_region(snippets::default::SOURCE, "example");
    let outline = DocSection::build(cx, "Outline", outline)
        .test_id_prefix("ui-gallery-button-outline")
        .code_rust_from_file_region(snippets::outline::SOURCE, "example");
    let secondary = DocSection::build(cx, "Secondary", secondary)
        .test_id_prefix("ui-gallery-button-secondary")
        .code_rust_from_file_region(snippets::secondary::SOURCE, "example");
    let ghost = DocSection::build(cx, "Ghost", ghost)
        .test_id_prefix("ui-gallery-button-ghost")
        .code_rust_from_file_region(snippets::ghost::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .test_id_prefix("ui-gallery-button-destructive")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let link = DocSection::build(cx, "Link", link)
        .test_id_prefix("ui-gallery-button-link")
        .description("The documented link variant remains a button-styled text action.")
        .code_rust_from_file_region(snippets::link::SOURCE, "example");
    let icon_only = DocSection::build(cx, "Icon", icon_only)
        .test_id_prefix("ui-gallery-button-icon")
        .description("Icon-only outline button.")
        .code_rust_from_file_region(snippets::icon::SOURCE, "example");
    let with_icon = DocSection::build(cx, "With Icon", with_icon)
        .test_id_prefix("ui-gallery-button-with-icon")
        .description("Leading and trailing icon patterns.")
        .code_rust_from_file_region(snippets::with_icon::SOURCE, "example");
    let rounded = DocSection::build(cx, "Rounded", rounded)
        .test_id_prefix("ui-gallery-button-rounded")
        .description("Keep `rounded-full` as an explicit call-site refinement.")
        .code_rust_from_file_region(snippets::rounded::SOURCE, "example");
    let spinner = DocSection::build(cx, "Spinner", spinner)
        .test_id_prefix("ui-gallery-button-spinner")
        .description("Disabled button compositions with inline spinners.")
        .code_rust_from_file_region(snippets::loading::SOURCE, "example");
    let button_group = DocSection::build(cx, "Button Group", button_group)
        .test_id_prefix("ui-gallery-button-button-group")
        .description("See `ButtonGroup` for grouped actions with shared chrome.")
        .code_rust_from_file_region(snippets::button_group::SOURCE, "example");
    let link_render = DocSection::build(cx, "As Link / As Child (Semantic)", link_render)
        .test_id_prefix("ui-gallery-button-link-semantic")
        .description(
            "Fret equivalent of the upstream semantic-link lane across both the Base UI `As Link` docs section and the Radix `asChild` link example.",
        )
        .code_rust_from_file_region(snippets::link_render::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-button-rtl")
        .description(
            "Translated upstream RTL row showing logical inline-start/inline-end slot flipping; icon glyph mirroring stays caller-owned.",
        )
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let children = DocSection::build(cx, "Children (Fret)", children)
        .test_id_prefix("ui-gallery-button-children")
        .description("Single-node slot helpers and explicit full-row child composition.")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");
    let variants = DocSection::build(cx, "Variants Overview (Fret)", variants)
        .test_id_prefix("ui-gallery-button-variants-overview")
        .description("Compact variant comparison kept for diagnostics and quick visual diffing.")
        .code_rust_from_file_region(snippets::variants::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Button docs order first while keeping the current `new-york-v4` button chrome baseline and a shared semantic-link lane for the Base `As Link` / Radix `As Child` follow-up.",
        ),
        vec![
            demo,
            usage,
            cursor,
            size,
            default,
            outline,
            secondary,
            ghost,
            destructive,
            link,
            icon_only,
            with_icon,
            rounded,
            spinner,
            button_group,
            link_render,
            rtl,
            api_reference,
            children,
            variants,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-button").into_element(cx)]
}
