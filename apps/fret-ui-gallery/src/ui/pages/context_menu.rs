use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::context_menu as snippets;

pub(super) fn preview_context_menu(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let submenu = snippets::submenu::render(cx);
    let shortcuts = snippets::shortcuts::render(cx);
    let groups = snippets::groups::render(cx);
    let icons = snippets::icons::render(cx);
    let checkboxes = snippets::checkboxes::render(cx);
    let radio = snippets::radio::render(cx);
    let destructive = snippets::destructive::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let examples = doc_layout::notes_block([
        "Gallery collapses the upstream top-of-page `ComponentPreview` into `Demo` and skips `Installation`, because the UI Gallery teaches live Rust surfaces rather than package-install steps.",
        "The upstream `Examples` group stays explicit here so `Basic`, `Submenu`, `Shortcuts`, `Groups`, `Icons`, `Checkboxes`, `Radio`, `Destructive`, and `Sides` remain easy to compare one-to-one with the docs page.",
        "`RTL` remains a separate top-level docs section after `Examples`, matching the upstream page structure instead of being folded into the example group.",
    ]);
    let api_reference = doc_layout::notes_block([
        "Reference baseline: shadcn base Context Menu docs.",
        "`ContextMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` is the default copyable root path; `build(...)`, `build_parts(...)`, and `into_element_parts(...)` stay as narrower adapter seams for already-landed or closure-owned roots.",
        "`ContextMenuTrigger`, `ContextMenuPortal`, `ContextMenuContent`, and `ContextMenuSub*` keep the shadcn/Base UI part names available without moving menu policy into `fret-ui`.",
        "Trigger chrome stays caller-owned, while explicit panel sizing stays on `ContextMenuContent::{min_width,submenu_min_width}(...)`.",
        "Logical-side placement now matches the upstream Base UI docs path: `DropdownMenuSide::{InlineStart, InlineEnd}` is accepted by `ContextMenuContent::side(...)` and `DropdownMenuContent::side(...)`.",
        "The docs-backed preview examples now use a caller-owned dashed context region closer to the upstream docs surface, including pointer-aware trigger copy (`Right click here` vs `Long press here`); `Usage` intentionally stays simpler so the typed root lane remains easy to copy.",
        "No extra generic heterogeneous children API is currently warranted: the explicit `ContextMenuEntry` tree is the Fret-equivalent structured surface for upstream nested menu children, and a generic children lane would add hidden scope/collection contracts without unlocking new behavior.",
    ]);
    let notes = doc_layout::notes_block([
        "Preview now mirrors the upstream shadcn/Base UI Context Menu docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Examples`, `RTL`, and `API Reference`.",
        "Mechanism parity already looks healthy here: existing web-vs-fret placement/chrome gates plus context-menu diag scripts cover right-click open, keyboard open, dismissal, focus routing, safe-corridor submenu behavior, and panel geometry.",
        "This pass mainly fixes teaching-surface drift: docs-aligned snippets now prefer the same typed `compose()` root lane instead of mixing older `build(...)` roots into the default examples.",
        "Those lower-level adapter seams are still advanced API, not the default copyable teaching lane.",
        "Docs-backed trigger copy now adapts to the committed primary pointer capability, so touch-first windows read `Long press here` / `Long press (...)` without needing any new context-menu mechanism work.",
        "The explicit `Examples` section now keeps the upstream grouping visible before the page returns to the top-level `RTL` and `API Reference` sections.",
        "The Sides preview now mirrors the upstream Base UI docs set more closely by covering `inline-start`, `left`, `top`, `bottom`, `right`, and `inline-end` in one section-level placement sweep.",
        "The RTL example now exercises logical-side placement directly: `ContextMenuContent::side(shadcn::DropdownMenuSide::InlineEnd)` matches the upstream Base UI docs while submenu chevrons still follow direction-provider parity.",
        "The RTL preview now stays closer to the upstream Base UI example shape too: dual submenus, checkbox toggles, and a radio group all render under `LayoutDirection::Rtl` while keeping the explicit `inline-end` teaching point.",
        "The explicit entry tree remains intentional, so the page records why we are not widening this family into a generic heterogeneous children API.",
        "Examples are snippet-backed: preview and code stay in sync.",
        "Keep `ui-gallery-context-menu-*` test IDs stable; multiple diag scripts depend on them.",
    ]);
    let examples = DocSection::build(cx, "Examples", examples)
        .no_shell()
        .test_id_prefix("ui-gallery-context-menu-examples")
        .description("How the upstream `Examples` group maps onto the preview sections below.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-context-menu-api-reference")
        .description("Public-surface ownership, part-surface mapping, and children API guidance.");
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-context-menu-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official-style combined example with submenu, toggles, and radio items.")
        .test_id_prefix("ui-gallery-context-menu-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable minimal usage aligned with the upstream four-item `Profile / Billing / Team / Subscription` block.")
        .test_id_prefix("ui-gallery-context-menu-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("A simple context menu with a few actions.")
        .test_id_prefix("ui-gallery-context-menu-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let submenu = DocSection::build(cx, "Submenu", submenu)
        .description("Use `ContextMenuSub*` helpers to nest secondary actions.")
        .test_id_prefix("ui-gallery-context-menu-submenu")
        .code_rust_from_file_region(snippets::submenu::SOURCE, "example");
    let shortcuts = DocSection::build(cx, "Shortcuts", shortcuts)
        .description("Use `ContextMenuShortcut` to show keyboard hints.")
        .test_id_prefix("ui-gallery-context-menu-shortcuts")
        .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Group related actions and separate them with dividers.")
        .test_id_prefix("ui-gallery-context-menu-groups")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example");
    let icons = DocSection::build(cx, "Icons", icons)
        .description("Combine icons with labels for quick scanning.")
        .test_id_prefix("ui-gallery-context-menu-icons")
        .code_rust_from_file_region(snippets::icons::SOURCE, "example");
    let checkboxes = DocSection::build(cx, "Checkboxes", checkboxes)
        .description("Use checkbox items for toggles.")
        .test_id_prefix("ui-gallery-context-menu-checkboxes")
        .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example");
    let radio = DocSection::build(cx, "Radio", radio)
        .description("Use a radio group for exclusive choices.")
        .test_id_prefix("ui-gallery-context-menu-radio")
        .code_rust_from_file_region(snippets::radio::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .description("Use `variant=Destructive` for irreversible actions.")
        .test_id_prefix("ui-gallery-context-menu-destructive")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let sides = DocSection::build(cx, "Sides", sides)
        .description("Control content placement with physical and logical `side` props.")
        .test_id_prefix("ui-gallery-context-menu-sides")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL layout keeps logical `inline-end` placement while mirroring the richer upstream preview structure.")
        .test_id_prefix("ui-gallery-context-menu-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the upstream shadcn/Base UI Context Menu docs path after collapsing `ComponentPreview` into `Demo` and skipping `Installation`, then keeps Fret-specific API guidance and notes explicit.",
        ),
        vec![
            demo,
            usage,
            examples,
            basic,
            submenu,
            shortcuts,
            groups,
            icons,
            checkboxes,
            radio,
            destructive,
            sides,
            rtl,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-context-menu").into_element(cx)]
}
