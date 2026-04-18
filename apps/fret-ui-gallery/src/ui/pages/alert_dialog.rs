use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert_dialog as snippets;

pub(super) fn preview_alert_dialog(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let small = snippets::small::render(cx);
    let media = snippets::media::render(cx);
    let small_with_media = snippets::small_with_media::render(cx);
    let destructive = snippets::destructive::render(cx);
    let parts = snippets::parts::render(cx);
    let detached_trigger = snippets::detached_trigger::render(cx);
    let rich_content = snippets::rich_content::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`AlertDialogContent::size(...)` accepts `AlertDialogContentSize::Default` and `AlertDialogContentSize::Sm`, matching the upstream `size=\"default\" | \"sm\"` surface.",
        "`AlertDialog::children([...])` is the default copyable root path for part-based composition, and `AlertDialogPart` is available on the curated `shadcn` facade so the default import lane stays copyable.",
        "`AlertDialogPart::content_with(...)` plus `AlertDialogContent::with_children(...)`, `AlertDialogHeader::with_children(...)`, and `AlertDialogFooter::with_children(...)` form the default copyable content lane when child parts need the current alert-dialog scope.",
        "`AlertDialog::children([...])` is already the warranted composable root API here because the component owns Trigger/Portal/Overlay/Content parts and the scope-sensitive `from_scope(...)` buttons must stay inside `AlertDialogContent`; no broader untyped JSX-style root children API is warranted beyond the typed `AlertDialogPart` lane.",
        "`AlertDialogContent::new([...])`, `AlertDialogHeader::new([...])`, and `AlertDialogFooter::new([...])` still work for already-materialized children, but they are not the default teaching path for `from_scope(...)` buttons.",
        "`AlertDialogContent::build(...)` remains the typed companion when staged or conditional section assembly is clearer than inline deferred children.",
        "`AlertDialogAction::from_scope(...)` and `AlertDialogCancel::from_scope(...)` keep footer composition close to shadcn docs without threading the same open model through every button.",
        "`AlertDialogCancel::variant(...)` keeps non-default destructive cancel styling available for upstream example surfaces without requiring a lower-level escape hatch.",
        "`AlertDialogTitle::new_children(...)`, `AlertDialogDescription::new_children(...)`, and `AlertDialogDescription::new_selectable_with(...)` cover the current composed rich-text / interactive-inline-content seams when string-only labels are not enough.",
    ]);

    let extras = doc_layout::notes_block([
        "`Parts` documents the advanced part adapters (`Trigger` / `Portal` / `Overlay`) used by explicit `compose()` call sites.",
        "`Detached Trigger` shows `AlertDialogHandle`, the supported path when the opener and the dialog content live in different subtrees and still need correct focus restore.",
        "`Rich Content` demonstrates the currently supported children-style extensions for title, description, and footer content.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`. Reference stack: shadcn Alert Dialog docs and examples, the default registry recipe, Radix Primitives alert-dialog semantics, and Base UI alert-dialog ownership.",
        "Preview mirrors the shadcn docs path after skipping `Installation`: `Demo`, `Usage`, `Basic`, `Small`, `Media`, `Small with Media`, `Destructive`, `RTL`, and `API Reference`.",
        "Example copy now follows the docs-page examples (`Show Dialog`, `Share Project`, dual-example RTL) rather than the registry-base example labels such as `Default (Media)`.",
        "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
        "Modal semantics follow Radix/Base UI outcomes: outside press does not dismiss, `role=alertdialog` is preserved, and initial focus prefers `AlertDialogCancel` when present.",
        "Radix Primitives and Base UI agree on the relevant semantics axis here: modal alert dialog, `role=alertdialog`, outside press does not dismiss, and initial focus prefers the least-destructive `Cancel` action. Those outcomes are already handled in `fret-ui-kit` / `fret-ui-shadcn`, so this page is now mainly a docs/public-surface alignment task rather than a `fret-ui` mechanism bug.",
        "`Usage` is the default copyable path; `Parts` remains an advanced adapter lane for explicit root-part ownership.",
        "`Usage` now teaches the root `children([...])` path plus deferred `content_with(...)` / `with_children(...)` composition because `from_scope(...)` parts must be authored inside the alert-dialog content scope.",
        "Current remaining difference is mostly that heterogeneous Rust child lists still land explicitly with `.into_element(cx)`; behavior, layout, dismissal policy, and the content-side authoring seam are already aligned in the recipe/mechanism split.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-alert-dialog-api-reference")
        .description("Public surface summary and current authoring-surface guidance.");
    let extras = DocSection::build(cx, "Fret Extras", extras)
        .no_shell()
        .test_id_prefix("ui-gallery-alert-dialog-extras")
        .description("Focused follow-ups that stay outside the upstream docs path.");
    let notes = DocSection::build(cx, "Notes", notes)
        .title_test_id("ui-gallery-section-notes-title")
        .test_id_prefix("ui-gallery-alert-dialog-notes")
        .description("Parity notes and implementation pointers.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Default-sized modal alert dialog.")
        .test_id_prefix("ui-gallery-alert-dialog-demo-docsec")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference using deferred content children inside the alert-dialog scope.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("A minimal alert dialog with default buttons.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let small = DocSection::build(cx, "Small", small)
        .description("Use the `size=\"sm\"` prop to make the alert dialog smaller.")
        .test_id_prefix("ui-gallery-alert-dialog-small-docsec")
        .code_rust_from_file_region(snippets::small::SOURCE, "example");
    let media = DocSection::build(cx, "Media", media)
        .description("Use the `AlertDialogMedia` component to add a media element such as an icon or image to the alert dialog.")
        .test_id_prefix("ui-gallery-alert-dialog-media-docsec")
        .code_rust_from_file_region(snippets::media::SOURCE, "example");
    let small_with_media = DocSection::build(cx, "Small with Media", small_with_media)
        .description("Use the `size=\"sm\"` prop to make the alert dialog smaller and the `AlertDialogMedia` component to add a media element such as an icon or image to the alert dialog.")
        .code_rust_from_file_region(snippets::small_with_media::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .description("Use the `AlertDialogAction` component to add a destructive action button to the alert dialog.")
        .test_id_prefix("ui-gallery-alert-dialog-destructive-docsec")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("To enable RTL support in shadcn/ui, see the RTL configuration guide.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description(
            "Advanced part surface adapters for explicit shadcn-style root-part ownership.",
        )
        .test_id_prefix("ui-gallery-alert-dialog-parts-docsec")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");
    let detached_trigger = DocSection::build(cx, "Detached Trigger", detached_trigger)
        .description("Use `AlertDialogHandle` when the opener and the dialog content are authored in different subtrees.")
        .test_id_prefix("ui-gallery-alert-dialog-detached-trigger-docsec")
        .code_rust_from_file_region(snippets::detached_trigger::SOURCE, "example");
    let rich_content = DocSection::build(cx, "Rich Content", rich_content)
        .description("Composable title/description content plus custom footer button content using `new_children(...)`, `new_selectable_with(...)`, and `children(...)`.")
        .test_id_prefix("ui-gallery-alert-dialog-rich-content-docsec")
        .code_rust_from_file_region(snippets::rich_content::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Alert Dialog docs path after `Installation`, then keeps Fret-only follow-ups explicit under `Fret Extras`.",
        ),
        vec![
            demo,
            usage,
            basic,
            small,
            media,
            small_with_media,
            destructive,
            rtl,
            api_reference,
            extras,
            parts,
            detached_trigger,
            rich_content,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-alert-dialog").into_element(cx)]
}
