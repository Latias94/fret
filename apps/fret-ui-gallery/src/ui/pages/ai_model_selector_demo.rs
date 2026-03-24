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
                "Thin wrapper over shadcn `Dialog`. Owns open state only via `open_model(...)` / `default_open(...)`.",
            ],
            [
                "ModelSelectorTrigger",
                "Pressable wrapper that flips `open=true` on activate while letting apps fully own trigger visuals.",
            ],
            [
                "ModelSelectorContent",
                "Dialog content shell with an accessible title (rendered as `sr-only`) and a `Command` container. Default title is \"Model Selector\".",
            ],
            [
                "ModelSelectorDialog",
                "Alias for shadcn `CommandDialog` (full dialog + cmdk-aligned palette). Use when you want the canonical command-dialog assembly.",
            ],
            [
                "ModelSelectorInput",
                "Alias for shadcn `CommandInput`. In Rust it is bound to an explicit query `Model<String>`.",
            ],
            [
                "ModelSelectorList",
                "Alias for shadcn `CommandList`. Supports cmdk-style fuzzy scoring + grouping with `query_model(...)`, plus `empty_text(...)` for empty-state text.",
            ],
            [
                "ModelSelectorEmpty",
                "Alias for shadcn `CommandEmpty` (used internally by `CommandList` via `empty_text(...)`).",
            ],
            ["ModelSelectorGroup", "Alias for shadcn `CommandGroup`."],
            ["ModelSelectorItem", "Alias for shadcn `CommandItem`."],
            [
                "ModelSelectorShortcut",
                "Alias for shadcn `CommandShortcut` (right-aligned shortcut hint).",
            ],
            [
                "ModelSelectorSeparator",
                "Alias for shadcn `CommandSeparator` (cmdk `always_render(...)` supported).",
            ],
            [
                "ModelSelectorLogo",
                "Provider identity helper. Upstream loads SVGs from `models.dev`; the Fret port renders a local placeholder badge by default.",
            ],
            [
                "ModelSelectorLogoGroup",
                "Provider logo strip helper. Matches upstream's overlapping layout (Tailwind `-space-x-1`) via a small negative margin shim.",
            ],
            [
                "ModelSelectorName",
                "Small name label helper (e.g. \"GPT-4o\").",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_model_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::model_selector_demo::render(cx);
    let features = doc_layout::notes_block([
        "Searchable interface with keyboard navigation.",
        "Fuzzy search filtering across model names (cmdk-style scoring).",
        "Grouped model organization by provider.",
        "Keyboard shortcuts support via `ModelSelectorShortcut`.",
        "Empty state handling via `ModelSelectorList::empty_text(...)`.",
        "Customizable styling via `Theme` tokens plus `refine_style(...)` / `refine_layout(...)` surfaces.",
        "Built from the same shadcn `Dialog` + `Command*` composition as the official AI Elements component.",
        "Rust-native API with typed builders plus explicit `Model<T>` state ownership in app code.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "Selection state, model inventory, and provider grouping are intentionally app-owned; `ModelSelector` stays a thin UI wrapper.",
        "The current `ModelSelectorLogo` avoids remote network fetches and renders a local placeholder badge instead.",
        "If you need the fully cmdk-aligned input semantics surface, prefer `ModelSelectorDialog` (shadcn `CommandDialog`) / `CommandPalette` rather than expanding selector policy in `crates/fret-ui`.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("A searchable command palette for selecting AI models in your chat interface."),
        vec![
            DocSection::build(cx, "Compound API", demo)
                .descriptions([
                    "Uses the same dialog + command decomposition as the official AI Elements docs.",
                    "Shows a Rust-friendly compound entrypoint via `ModelSelector::into_element_with_children(...)` and slot-based composition.",
                ])
                .test_id_prefix("ui-gallery-ai-model-selector-demo")
                .code_rust_from_file_region(snippets::model_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Key features, aligned with the official AI Elements docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Mapping from AI Elements parts to Fret surfaces.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Intentional divergences and layering guidance.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
