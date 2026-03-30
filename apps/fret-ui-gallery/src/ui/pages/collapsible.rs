use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::collapsible as snippets;

pub(super) fn preview_collapsible(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let controlled_state = snippets::controlled_state::render(cx);
    let basic = snippets::basic::render(cx);
    let settings = snippets::settings_panel::render(cx);
    let file_tree = snippets::file_tree::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Collapsible::new(Model<bool>)` and `Collapsible::uncontrolled(default_open)` cover the documented controlled and uncontrolled authoring paths.",
        "`shadcn::CollapsibleRoot`, `shadcn::CollapsibleTriggerPart`, and `shadcn::CollapsibleContentPart` provide the curated composable children lane; `shadcn::raw::collapsible::primitives::*` remains the explicit source-alignment escape hatch.",
        "The top-level `shadcn::Collapsible` wrapper stays a compact Fret-first builder for dense editor UIs, while the parts aliases keep the official shadcn children composition copyable on the curated facade.",
        "`CollapsibleTriggerPart::as_child(true)` keeps the official trigger-as-child lane copyable on the curated facade, while the compact wrapper still exposes the dense editor-friendly trigger/content builder.",
        "The `Demo` section now mirrors the official shadcn `@peduarte starred 3 repositories` example instead of the previous base-style order-details card.",
        "Disclosure state, trigger semantics, and measured open/close motion remain recipe/primitive-owned; surrounding width, gap, and card/layout constraints remain caller-owned.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/collapsible.rs`. Reference stack: shadcn Collapsible docs and examples, the default registry recipe, Radix Primitives collapsible semantics, and Base UI collapsible ownership.",
        "Preview mirrors the shadcn/base Collapsible docs path first after skipping `Installation`: `Demo`, `Usage`, `Controlled State`, `Basic`, `Settings Panel`, `File Tree`, `RTL`, and `API Reference`.",
        "`Demo` follows the current shadcn demo's repository-list surface, while `Basic`, `Settings Panel`, `File Tree`, and `RTL` track the base docs examples.",
        "`shadcn::Collapsible` remains the compact Fret-first builder for the common trigger/content lane, while `shadcn::CollapsibleRoot`, `shadcn::CollapsibleTriggerPart`, and `shadcn::CollapsibleContentPart` cover the copyable composable children lane on the curated facade.",
        "A broader generic `Collapsible::children([...])` / `compose()` root API is not currently warranted here: Collapsible only needs trigger/content, and the typed parts lane plus `shadcn::raw::collapsible::primitives::*` already cover the upstream nested composition / `asChild` story without widening the default recipe surface.",
        "Radix Primitives and Base UI agree on the relevant semantics axis here: controllable/uncontrolled open state, trigger-expanded/controls relationships, and measured panel lifecycle. Those outcomes are already handled in `fret-ui-kit` / `fret-ui-shadcn`, so the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "Keep width, gap, and card layout caller-owned when the upstream example does; the recipe owns disclosure semantics, trigger/content wiring, and the measured open/close motion substrate.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-collapsible-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .title_test_id("ui-gallery-section-notes-title")
        .test_id_prefix("ui-gallery-collapsible-notes")
        .description("Parity notes, source axes, and the current children-API decision.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official shadcn repository-list demo with `Trigger(asChild)` and a composable content section.")
        .test_id_prefix("ui-gallery-collapsible-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable composable usage on the curated facade via `CollapsibleRoot` and the parts aliases.")
        .test_id_prefix("ui-gallery-collapsible-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let controlled_state = DocSection::build(cx, "Controlled State", controlled_state)
        .description("Controlled via `Model<bool>` when state must be driven externally.")
        .test_id_prefix("ui-gallery-collapsible-controlled")
        .code_rust_from_file_region(snippets::controlled_state::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Basic collapsible card outcome, authored through the compact Fret wrapper.")
        .test_id_prefix("ui-gallery-collapsible-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let settings = DocSection::build(cx, "Settings Panel", settings)
        .description("Use a trigger button to reveal additional settings.")
        .test_id_prefix("ui-gallery-collapsible-settings")
        .code_rust_from_file_region(snippets::settings_panel::SOURCE, "example");
    let file_tree = DocSection::build(cx, "File Tree", file_tree)
        .description("Use nested collapsibles to build a file tree.")
        .test_id_prefix("ui-gallery-collapsible-file-tree")
        .code_rust_from_file_region(snippets::file_tree::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider keeps trigger and content alignment stable under RTL.")
        .test_id_prefix("ui-gallery-collapsible-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Collapsible docs path first: Demo, Usage, Controlled State, Basic, Settings Panel, File Tree, RTL, and API Reference. The lead demo follows the current repository-list example, the usage section keeps the composable children API on the curated facade, and `Notes` records the source axes plus the current children-API decision.",
        ),
        vec![
            demo,
            usage,
            controlled_state,
            basic,
            settings,
            file_tree,
            rtl,
            api_reference,
            notes,
        ],
    );

    let body = body.test_id("ui-gallery-collapsible-component");
    vec![body.into_element(cx)]
}
