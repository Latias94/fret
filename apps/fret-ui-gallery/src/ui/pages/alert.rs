use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert as snippets;

pub(super) fn preview_alert(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let rich_title = snippets::rich_title::render(cx);
    let destructive = snippets::destructive::render(cx);
    let action = snippets::action::render(cx);
    let interactive_links = snippets::interactive_links::render(cx);
    let custom_colors = snippets::custom_colors::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Alert::new([...])` and `Alert::build(...)` cover the default root composition lane.",
        "`variant(...)` covers the documented `default` and `destructive` recipe surface.",
        "`AlertAction::build(...)` is the preferred typed slot surface for top-end actions; `AlertAction::new([...])` remains valid when the action subtree is already landed.",
        "`AlertTitle::new(...)` keeps the compact title lane, while `AlertTitle::new_children(...)` and `AlertTitle::build(...)` cover attributed or precomposed title content.",
        "`AlertDescription::new(...)` keeps the plain-text lane, while `AlertDescription::new_children(...)` and `AlertDescription::build(...)` cover multi-paragraph or composed description content.",
        "Recipe-owned defaults stop at intrinsic alert chrome and `w-full`; caller-owned width negotiation such as `max-w-*`, `min-w-0`, or page centering stays on the surrounding layout.",
    ]);

    let extras = doc_layout::notes_block([
        "`Rich Title` demonstrates the builder-first composed title lane for attributed text.",
        "`Interactive Links` demonstrates deterministic, diagnostics-safe composed link content inside `AlertDescription`.",
        "Those follow-ups stay after `API Reference` because they extend the copyable Fret teaching surface rather than the upstream docs path itself.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/alert.rs`.",
        "Reference stack: shadcn Alert docs on the Radix and Base UI lanes, the default registry recipe, and the action-slot example.",
        "This audit lands on the recipe/docs axis, not the runtime mechanism axis: `Alert` is a static callout and does not need new `fret-ui` substrate work.",
        "Current new-york-v4 chrome uses `line-clamp-1` for `AlertTitle`, so the base/radix docs' multiline-title examples remain a known chrome-axis difference rather than a `fret-ui` bug.",
        "Preview now mirrors the shadcn docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Destructive`, `Action`, `Custom Colors`, `RTL`, and `API Reference`.",
        "`Rich Title` and `Interactive Links` stay explicit under `Fret Extras` so composed title/description authoring remains copyable without pretending those sections are upstream docs examples.",
        "Current chrome baseline: the default shadcn registry `Alert` recipe.",
        "Action-slot reference: the radix registry alert recipe and example.",
        "Keep alert copy concise and action-oriented; reserve longer guidance for Dialog or Sheet.",
        "Prefer `AlertTitle::build(...)` or `AlertTitle::new_children(...)` when the title needs attributed text or a precomposed child subtree.",
        "Prefer `AlertDescription::build(...)` or `AlertDescription::new_children(...)` when the description needs multiple paragraphs, lists, or rich text.",
        "Gallery link examples open safe URLs in normal runs; scripted diag runs keep them deterministic by recording activation instead of launching the browser.",
        "Use `Destructive` only for high-risk or blocking failures to preserve visual hierarchy.",
        "Validate RTL + narrow layout so icon/title/description remain readable in editor sidebars.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and default-style ownership notes.")
        .test_id_prefix("ui-gallery-alert-api-reference");
    let extras = DocSection::build(cx, "Fret Extras", extras)
        .no_shell()
        .description(
            "Fret-specific composed-content follow-ups kept outside the upstream docs path.",
        )
        .test_id_prefix("ui-gallery-alert-extras");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("Audit conclusions, ownership notes, and upstream reference pointers.")
        .test_id_prefix("ui-gallery-alert-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Top-level docs preview for the current upstream `alert-demo` surface.")
        .test_id_prefix("ui-gallery-alert-demo-docsec")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable root + slot composition for `Alert`, `AlertTitle`, `AlertDescription`, and `AlertAction`.")
        .test_id_prefix("ui-gallery-alert-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description(
            "Docs-aligned basic patterns: title-only, title + description, and description-only.",
        )
        .test_id_prefix("ui-gallery-alert-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .description("Use `variant(\"destructive\")` to create a destructive alert.")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let action = DocSection::build(cx, "Action", action)
        .description("Use `AlertAction` to add button or badge content to the top-end action slot.")
        .code_rust_from_file_region(snippets::action::SOURCE, "example");
    let rich_title = DocSection::build(cx, "Rich Title", rich_title)
        .description(
            "Composable title content using the builder-first `AlertTitle::build(...)` lane.",
        )
        .test_id_prefix("ui-gallery-alert-rich-title")
        .code_rust_from_file_region(snippets::rich_title::SOURCE, "example");
    let interactive_links = DocSection::build(cx, "Interactive Links", interactive_links)
        .description("A Fret-specific composed-description pattern: normal runs open safe URLs, while diagnostics still keep deterministic activation evidence.")
        .code_rust_from_file_region(snippets::interactive_links::SOURCE, "example");
    let custom_colors = DocSection::build(cx, "Custom Colors", custom_colors)
        .description("Customize alert colors with caller-owned chrome refinements.")
        .code_rust_from_file_region(snippets::custom_colors::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("To enable RTL support in shadcn/ui, see the RTL configuration guide.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Alert docs path after `Installation`, while the default chrome stays aligned to the current new-york-v4 alert source and the later `Fret Extras` keep composed-content lanes explicit.",
        ),
        vec![
            demo,
            usage,
            basic,
            destructive,
            action,
            custom_colors,
            rtl,
            api_reference,
            extras,
            rich_title,
            interactive_links,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-alert-component").into_element(cx)]
}
