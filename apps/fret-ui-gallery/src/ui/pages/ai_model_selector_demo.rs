use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;
use fret_ui_kit::ui::UiElementSinkExt as _;

fn parts_table(cx: &mut UiCx<'_>) -> AnyElement {
    let row = |part: &'static str, surface: &'static str| {
        shadcn::TableRow::build(2, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(surface)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Fret surface").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("ModelSelector", "Thin dialog root that owns open state only, now with a Rust-friendly `into_element_with_children(...)` compound entrypoint. Selected model and search query stay app-owned."));
                out.push_ui(cx, row("ModelSelectorTrigger", "Recipe-level trigger surface that opens the dialog while letting apps compose provider logos and names explicitly."));
                out.push_ui(cx, row("ModelSelectorContent + Dialog", "Dialog shell with accessible title plus command container. `ModelSelectorDialog` remains the alternate full command-dialog surface."));
                out.push_ui(cx, row("Input / List / Empty / Group / Item / Shortcut / Separator", "These parts intentionally lean on shared `Command*` surfaces, which matches the upstream component's thin-wrapper nature and keeps behavior centralized."));
                out.push_ui(cx, row("Logo / LogoGroup / Name", "Presentation helpers for provider identity. The current Fret port uses local placeholder badges instead of the official remote `models.dev` SVG source."));
            }),
        );
    })
    .into_element(cx)
}

pub(super) fn preview_ai_model_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::model_selector_demo::render(cx);
    let features = doc_layout::notes(
        cx,
        [
            "This selector is intentionally a thin wrapper over dialog + command surfaces, but it now shares the same root-level `into_element_with_children(...)` composition pattern as the other AI selectors.",
            "The demo keeps selected model state, query state, and provider grouping local so the snippet remains easy to lift into an app.",
            "Keyboard navigation, filtering, empty-state handling, and grouped entries all come from the shared command surface rather than selector-specific runtime code.",
            "Provider badges and label layout are recipe-level helpers; they improve docs parity without turning `ModelSelector` into a policy-heavy root.",
            "This page now follows the same doc layout as `MicSelector`, making selector ecosystem differences easier to compare side by side.",
        ],
    );
    let parts = parts_table(cx);
    let notes = doc_layout::notes(
        cx,
        [
            "`ModelSelector` should stay lighter than `VoiceSelector`; sharing the root compound entrypoint is enough, and the main remaining parity work still belongs in shared command composition and docs examples.",
            "The current `ModelSelectorLogo` intentionally avoids remote network fetches and renders a local placeholder badge instead, which is a reasonable Fret-specific divergence for now.",
            "If we later want 1:1 docs parity, the next step is refining `Command` composition and optional asset hooks rather than pushing model-specific semantics into `crates/fret-ui`.",
        ],
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned preview keeps the AI Elements dialog + command split while intentionally leaving model inventory, query ownership, and provider policy in app code.",
        ),
        vec![
            DocSection::new("Compound API", demo)
                .descriptions([
                    "Shows the same dialog / content / input / list decomposition as the official `model-selector` docs, expressed with the same root-level `into_element_with_children(...)` pattern used by the other selectors.",
                    "Highlights that most of the surface is still a thin alias layer over shared command parts, with only a few selector-specific presentation helpers on top.",
                ])
                .test_id_prefix("ui-gallery-ai-model-selector-demo")
                .code_rust_from_file_region(snippets::model_selector_demo::SOURCE, "example"),
            DocSection::new("Features", features)
                .description("High-signal parity notes against the official AI Elements docs.")
                .no_shell(),
            DocSection::new("Parts & Props", parts)
                .description("Which parts are thin wrappers over shared command surfaces and which remain selector-specific helpers.")
                .no_shell(),
            DocSection::new("Notes", notes)
                .description("Layering and next-step parity guidance.")
                .no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-model-selector-demo")]
}
