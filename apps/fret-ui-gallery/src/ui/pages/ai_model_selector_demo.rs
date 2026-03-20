use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Fret surface"],
        [
            [
                "ModelSelector",
                "Thin dialog root that owns open state only, now with a Rust-friendly `into_element_with_children(...)` compound entrypoint. Selected model and search query stay app-owned.",
            ],
            [
                "ModelSelectorTrigger",
                "Recipe-level trigger surface that opens the dialog while letting apps compose provider logos and names explicitly.",
            ],
            [
                "ModelSelectorContent + Dialog",
                "Dialog shell with accessible title plus command container. `ModelSelectorDialog` remains the alternate full command-dialog surface.",
            ],
            [
                "Input / List / Empty / Group / Item / Shortcut / Separator",
                "These parts intentionally lean on shared `Command*` surfaces, which matches the upstream component's thin-wrapper nature and keeps behavior centralized.",
            ],
            [
                "Logo / LogoGroup / Name",
                "Presentation helpers for provider identity. The current Fret port uses local placeholder badges instead of the official remote `models.dev` SVG source.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_model_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::model_selector_demo::render(cx);
    let features = doc_layout::notes_block([
        "This selector is intentionally a thin wrapper over dialog + command surfaces, but it now shares the same root-level `into_element_with_children(...)` composition pattern as the other AI selectors.",
        "The demo keeps selected model state, query state, and provider grouping local so the snippet remains easy to lift into an app.",
        "Keyboard navigation, filtering, empty-state handling, and grouped entries all come from the shared command surface rather than selector-specific runtime code.",
        "Provider badges and label layout are recipe-level helpers; they improve docs parity without turning `ModelSelector` into a policy-heavy root.",
        "This page now follows the same doc layout as `MicSelector`, making selector ecosystem differences easier to compare side by side.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "`ModelSelector` should stay lighter than `VoiceSelector`; sharing the root compound entrypoint is enough, and the main remaining parity work still belongs in shared command composition and docs examples.",
        "The current `ModelSelectorLogo` intentionally avoids remote network fetches and renders a local placeholder badge instead, which is a reasonable Fret-specific divergence for now.",
        "If we later want 1:1 docs parity, the next step is refining `Command` composition and optional asset hooks rather than pushing model-specific semantics into `crates/fret-ui`.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned preview keeps the AI Elements dialog + command split while intentionally leaving model inventory, query ownership, and provider policy in app code.",
        ),
        vec![
            DocSection::build(cx, "Compound API", demo)
                .descriptions([
                    "Shows the same dialog / content / input / list decomposition as the official `model-selector` docs, expressed with the same root-level `into_element_with_children(...)` pattern used by the other selectors.",
                    "Highlights that most of the surface is still a thin alias layer over shared command parts, with only a few selector-specific presentation helpers on top.",
                ])
                .test_id_prefix("ui-gallery-ai-model-selector-demo")
                .code_rust_from_file_region(snippets::model_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official AI Elements docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which parts are thin wrappers over shared command surfaces and which remain selector-specific helpers.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Layering and next-step parity guidance.")
                .no_shell(),
        ],
    );

    vec![body.into_element(cx)]
}
