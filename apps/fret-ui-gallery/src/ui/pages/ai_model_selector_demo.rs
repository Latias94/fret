use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;

fn parts_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        2,
        [
            shadcn::TableHead::new("Part").into_element(cx),
            shadcn::TableHead::new("Fret surface").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("ModelSelector")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Thin dialog root that owns open state only. Selected model and search query stay app-owned in the current Fret surface."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("ModelSelectorTrigger")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Recipe-level trigger surface that opens the dialog while letting apps compose provider logos and names explicitly."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("ModelSelectorContent + Dialog")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Dialog shell with accessible title plus command container. `ModelSelectorDialog` remains the alternate full command-dialog surface."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Input / List / Empty / Group / Item / Shortcut / Separator")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("These parts intentionally lean on shared `Command*` surfaces, which matches the upstream component's thin-wrapper nature and keeps behavior centralized."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Logo / LogoGroup / Name")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Presentation helpers for provider identity. The current Fret port uses local placeholder badges instead of the official remote `models.dev` SVG source."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

pub(super) fn preview_ai_model_selector_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::model_selector_demo::render(cx);
    let features = doc_layout::notes(
        cx,
        [
            "This selector is intentionally a thin wrapper over dialog + command surfaces, which already matches the spirit of the official AI Elements component.",
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
            "`ModelSelector` should stay lighter than `VoiceSelector`; the main parity work here belongs in shared command composition and docs examples, not in extra selector-owned contracts.",
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
                    "Shows the same dialog / content / input / list decomposition as the official `model-selector` docs, expressed with Rust builders instead of JSX nesting.",
                    "Highlights that most of the surface is a thin alias layer over shared command parts, with only a few selector-specific presentation helpers on top.",
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
