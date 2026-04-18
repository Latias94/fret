use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "ModelSelector",
                "new / open_model / default_open",
                "builder",
                "closed dialog",
                "Thin wrapper over shadcn `Dialog`. Owns open state via `open_model(...)` / `default_open(...)`.",
            ],
            [
                "ModelSelector",
                "children([...]) / trigger(...) / content(...)",
                "ModelSelectorChild",
                "-",
                "Docs-shaped compound root aligned with upstream `<ModelSelector>...</ModelSelector>` composition.",
            ],
            [
                "ModelSelector",
                "into_element_with_children(cx, ...)",
                "slot closure",
                "-",
                "Lower-level escape hatch when trigger/content need to be built under a live scope instead of an eager child list.",
            ],
            [
                "ModelSelectorTrigger",
                "new(child)",
                "AnyElement",
                "caller-owned child",
                "Pressable wrapper that flips `open=true` on activate while letting apps fully own trigger visuals.",
            ],
            [
                "ModelSelectorContent",
                "new(children) / input(...) / list(...)",
                "IntoIterator<Item = AnyElement> / typed builder",
                "DialogContent + Command shell",
                "Dialog content shell with an accessible title (`sr-only`) plus typed `ModelSelectorInput` / `ModelSelectorList` convenience lanes.",
            ],
            [
                "ModelSelectorInput",
                "new(model) / placeholder / input_test_id",
                "builder",
                "query-bound input",
                "Thin wrapper over shadcn `CommandInput` with the official `h-auto py-3.5` defaults.",
            ],
            [
                "ModelSelectorList",
                "new / new_entries / entries / query_model",
                "typed builder",
                "\"No results.\"",
                "Typed list wrapper over shadcn `CommandList` that accepts selector-owned groups/items while preserving cmdk query and empty-state behavior.",
            ],
            [
                "ModelSelectorGroup",
                "new / heading / items",
                "typed builder",
                "no heading",
                "Selector-owned group wrapper that keeps docs-shaped grouping without forcing `CommandGroup` at the call site.",
            ],
            [
                "ModelSelectorItem",
                "new / value / child / children / on_select_action",
                "typed builder",
                "label-derived value",
                "Selector-owned row builder with typed metadata children and app-owned selection action.",
            ],
            [
                "ModelSelectorEmpty / Shortcut / Separator / Dialog",
                "shared shadcn aliases",
                "type aliases",
                "-",
                "These stay aligned with shared `Command*` behavior so selector parity does not fork overlay or cmdk semantics.",
            ],
            [
                "ModelSelectorLogo",
                "new(provider)",
                "builder",
                "local placeholder badge",
                "Provider identity helper. Upstream loads SVGs from `models.dev`; the Fret port renders a local placeholder badge by default.",
            ],
            [
                "ModelSelectorLogoGroup",
                "new(children)",
                "typed builder",
                "overlapping logo strip",
                "Provider logo strip helper. Now accepts typed `ModelSelectorLogo` children directly instead of forcing prebuilt `AnyElement`s.",
            ],
            [
                "ModelSelectorName",
                "new(text)",
                "builder",
                "flex-1 truncate label",
                "Small name label helper (e.g. \"GPT-4o\").",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_model_selector_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::model_selector_demo::render(cx);
    let features = doc_layout::notes_block([
        "Behavior baseline remains healthy: searchable list filtering, grouped provider organization, open/close flow, and empty-state text all stay on shared shadcn `Dialog` + `Command*` surfaces.",
        "The root now exposes a docs-shaped `children([...])` lane, so the first-party example no longer has to teach slot-based composition as the only path.",
        "List/group/item are now selector-owned typed builders, which removes the old `CommandItem` alias cliff and lets the example stay closer to the official docs structure.",
        "Provider logo strips now accept typed `ModelSelectorLogo` children directly, so the example does not have to prebuild a row of `AnyElement`s just to express upstream composition.",
        "Selection state, model inventory, and provider grouping remain app-owned by design; this pass only corrected the public authoring surface.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "This is not a `crates/fret-ui` mechanism bug. The drift was still in `ecosystem/fret-ui-ai` public surface and gallery teaching surface.",
        "The current `ModelSelectorLogo` intentionally avoids remote network fetches and renders a local placeholder badge instead.",
        "If you need the fully cmdk-aligned input semantics surface, prefer `ModelSelectorDialog` (shadcn `CommandDialog`) / `CommandPalette` rather than expanding selector policy in `crates/fret-ui`.",
        "The slot-based `into_element_with_children(...)` escape hatch remains available for host-generic call sites, but it is no longer the recommended first teaching surface.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("A searchable command palette for selecting AI models in your chat interface."),
        vec![
            DocSection::build(cx, "Compound API", demo)
                .descriptions([
                    "Uses the same dialog + command decomposition as the official AI Elements docs, but now teaches the docs-shaped compound root first.",
                    "The example keeps selection state and model inventory in app code while the selector surface itself stays typed and copyable.",
                ])
                .test_id_prefix("ui-gallery-ai-model-selector-demo")
                .code_rust_from_file_region(snippets::model_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Key features, aligned with the official AI Elements docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Current Fret API surface for `ModelSelector`, including the docs-shaped root and typed list/group/item wrappers.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Intentional divergences and layering guidance.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
