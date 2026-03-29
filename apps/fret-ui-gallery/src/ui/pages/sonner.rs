use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sonner as snippets;
use fret::UiCx;

pub(super) fn preview_sonner(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx);
    let about = doc_layout::notes_block([
        "Sonner is an opinionated toast component by Emil Kowalski, and shadcn/ui now points toast users here instead of the deprecated Toast page.",
        "Fret keeps Sonner as the shadcn-facing toast recipe surface, while the actual overlay/store/rendering pipeline lives in `fret-ui-kit`.",
        "The current chrome baseline follows `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`; interaction outcomes are cross-checked against Radix Toast and Base UI Toast references.",
    ]);
    let types = snippets::types::render(cx);
    let description = snippets::description::render(cx);
    let position = snippets::position::render(cx);
    let examples = doc_layout::notes_block([
        "The upstream `Examples` group maps to the docs-backed preview sections below: `Types`, `Description`, and `Position`.",
        "`Demo` stays separate because the shadcn page leads with a top-of-page preview before the prose sections.",
    ]);
    let api_reference = doc_layout::notes_block([
        "`Toaster::new()` mirrors the shadcn wrapper defaults: top-center placement, Lucide icons, `Notifications` container label, no close button by default, and Sonner-style spacing/width defaults.",
        "`Sonner::global(app)` plus `toast_message(...)`, the typed variant helpers, and `ToastMessageOptions` cover the main docs-facing message lane (`description`, `action`, `cancel`, `duration`, `pinned`, `dismissible`, `icon`).",
        "`toast_promise(...)`, `toast_promise_with(...)`, and the async promise helpers keep the current loading-to-success/error story on the same facade instead of forcing app code down into raw store plumbing.",
        "A generic composable `children([...])` API is not warranted on the shadcn Sonner surface today: upstream Sonner is still message-oriented, while Radix/Base UI-style compound toast parts belong on a lower-level toast primitive if Fret needs fully custom toast bodies later.",
        "This pass did not identify a missing `fret-ui` mechanism bug; the main remaining work is docs/teaching alignment plus any future lower-level custom-content seam.",
    ]);
    let setup = snippets::setup::render(cx);
    let extras = snippets::extras::render(cx);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/sonner.rs`. Upstream refs: `repo-ref/ui/apps/v4/content/docs/components/base/sonner.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-types.tsx`, `repo-ref/primitives/packages/react/toast/src/toast.tsx`, and `repo-ref/base-ui/packages/react/src/toast/`.",
        "Preview mirrors the shadcn Sonner docs path after collapsing the top `ComponentPreview` into `Demo` and skipping package-install steps: `Demo`, `About`, `Usage`, `Examples`, `Types`, `Description`, `Position`, and `API Reference`.",
        "`Mounting (Fret)` and `Extras (Fret)` stay after `API Reference` because they are Fret-specific follow-ups rather than part of the upstream docs path.",
        "The code tabs now point at standalone docs sources instead of page-local gallery helpers, so copied snippets keep the required `Toaster` mount and action wiring in one place.",
        "Existing `web_vs_fret` tests plus `tools/diag-scripts/ui-gallery/sonner/*` still cover overlay geometry, stacking, and swipe-dismiss outcomes; this pass focused on shadcn docs parity and authoring clarity.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .max_w(Px(980.0))
        .no_shell()
        .description("Status + parity notes.")
        .test_id_prefix("ui-gallery-sonner-notes");
    let about = DocSection::build(cx, "About", about)
        .max_w(Px(980.0))
        .no_shell()
        .description("What Sonner is and which source axes Fret is aligning to.")
        .test_id_prefix("ui-gallery-sonner-about");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Self-contained minimal usage with a mounted `Toaster` and one message toast.")
        .test_id_prefix("ui-gallery-sonner-usage")
        .code_rust_from_file_region(snippets::usage::DOCS_SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Docs-aligned top preview for the primary `Show Toast` example.")
        .test_id_prefix("ui-gallery-sonner-demo")
        .code_rust_from_file_region(snippets::demo::DOCS_SOURCE, "example");
    let examples = DocSection::build(cx, "Examples", examples)
        .max_w(Px(980.0))
        .no_shell()
        .description("How the upstream `Examples` group maps onto the preview sections below.")
        .test_id_prefix("ui-gallery-sonner-examples");
    let types = DocSection::build(cx, "Types", types)
        .description("Default, status, and promise toast variants.")
        .test_id_prefix("ui-gallery-sonner-types")
        .code_rust_from_file_region(snippets::types::DOCS_SOURCE, "example");
    let description = DocSection::build(cx, "Description", description)
        .description("Toast with supporting copy, matching the docs example.")
        .test_id_prefix("ui-gallery-sonner-description")
        .code_rust_from_file_region(snippets::description::DOCS_SOURCE, "example");
    let position = DocSection::build(cx, "Position", position)
        .description(
            "Use `position` to move the toast placement; the gallery keeps the toaster local so placements stay deterministic.",
        )
        .test_id_prefix("ui-gallery-sonner-position")
        .code_rust_from_file_region(snippets::position::DOCS_SOURCE, "example");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .max_w(Px(980.0))
        .no_shell()
        .description(
            "Public surface summary, source axes, and the current children-API conclusion.",
        )
        .test_id_prefix("ui-gallery-sonner-api-reference");
    let setup = DocSection::build(cx, "Mounting (Fret)", setup)
        .description("Mount a `Toaster` once per window. The usage snippets below inline this too, but this is the smallest focused install surface.")
        .test_id_prefix("ui-gallery-sonner-mounting")
        .code_rust_from_file_region(snippets::setup::DOCS_SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
        .description(
            "Fret-specific extras after docs parity examples: action/cancel + swipe-dismiss.",
        )
        .test_id_prefix("ui-gallery-sonner-extras")
        .code_rust_from_file_region(snippets::extras::DOCS_SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Sonner docs path after collapsing the top `ComponentPreview` into `Demo` and skipping package-install steps, then keeps Fret-specific mounting and extras explicit after the docs path.",
        ),
        vec![
            demo,
            about,
            usage,
            examples,
            types,
            description,
            position,
            api_reference,
            setup,
            extras,
            notes,
        ],
    );
    let toaster = snippets::local_toaster(cx).into_element(cx);

    vec![body.test_id("ui-gallery-sonner").into_element(cx), toaster]
}
