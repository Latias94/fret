use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_suggestions_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::suggestions_demo::render(cx);
    let custom = snippets::suggestions_demo::render_custom_children(cx);
    let features = doc_layout::notes_block([
        "Horizontal scroll row aligned with AI Elements `Suggestions`: full-width viewport, non-wrapping pills, and stable spacing between chips.",
        "Default `Suggestion` chrome stays on the shadcn-aligned `outline` + `sm` button recipe with the expected full-radius pill shape.",
        "Suggestion activation emits the original suggestion string, so prompt insertion and send behavior remain app-owned rather than baked into the component.",
        "The gallery example leaves stable `test_id` hooks on the root, viewport, pills, and prompt input so the existing diag script stays deterministic.",
    ]);
    let notes = doc_layout::notes_block([
        "Mechanism health looks good: the existing UI Gallery diag already covers the click-to-prompt path, so this alignment work stays in `fret-ui-ai` / UI Gallery instead of `crates/fret-ui`.",
        "The main drift was public-surface parity. Upstream `Suggestion` accepts custom `children`, while Fret previously only exposed `label(...)` as a text-only fallback. `children(...)` is now supported for docs-aligned custom chip content.",
        "The official AI Elements prose mentions wrapping on smaller screens, but the upstream source of truth is a non-wrapping horizontal `ScrollArea`. Fret follows the source-level behavior and keeps horizontal scrolling as the responsive path.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` demo surfaces in UI Gallery.",
    ]);
    let props = suggestions_props_table(cx).test_id("ui-gallery-ai-suggestions-props");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The `Suggestion` family displays a horizontal row of clickable suggestion pills. In Fret, the component stays policy-level: the chip emits intent, and the app still owns prompt state, submission, and side effects.",
        ),
        vec![
            DocSection::build(cx, "Usage with PromptInput", usage)
                .description("Rust/Fret analogue of the official AI Elements Suggestion usage example.")
                .description("The preview intentionally keeps prompt filling app-owned: clicking a suggestion writes into the prompt model instead of hard-wiring an LLM transport into the component.")
                .test_id_prefix("ui-gallery-ai-suggestions-demo")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::suggestions_demo::SOURCE, "example"),
            DocSection::build(cx, "Custom Content", custom)
                .description("Docs-aligned composable children surface for icon + label chips while preserving the original suggestion payload.")
                .test_id_prefix("ui-gallery-ai-suggestions-custom")
                .max_w(Px(980.0))
                .code_rust_from_file_region(
                    snippets::suggestions_demo::SOURCE,
                    "custom_children_example",
                ),
            DocSection::build(cx, "Features", features)
                .description("Behavior and default-value outcomes worth preserving while aligning against the official docs surface.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Builder Surface", props)
                .description("Current Fret builder surface for `Suggestions` / `Suggestion`, including the new composable children lane.")
                .max_w(Px(980.0)),
            DocSection::build(cx, "Notes", notes)
                .description("Layering, parity, and responsive-behavior notes for Suggestion.")
                .max_w(Px(980.0))
                .no_shell(),
        ],
    );

    vec![body.into_element(cx)]
}

fn suggestions_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Suggestions",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Scroll viewport root that hosts a single non-wrapping row of suggestion pills.",
            ],
            [
                "Suggestions",
                "refine_layout / viewport_test_id / test_id_root",
                "builder methods",
                "w_full + min_w_0 / None / None",
                "Adjust viewport sizing and diagnostics selectors without changing the scroll recipe.",
            ],
            [
                "Suggestion",
                "new(suggestion)",
                "impl Into<Arc<str>>",
                "-",
                "Required suggestion payload and default visible text.",
            ],
            [
                "Suggestion",
                "on_click",
                "Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>)>",
                "None",
                "Fret analogue of upstream `onClick`: emits the selected suggestion string while effects remain app-owned.",
            ],
            [
                "Suggestion",
                "children",
                "IntoIterator<Item = AnyElement>",
                "None",
                "Docs-aligned custom content override. Keeps the button accessible while replacing the default visible text row.",
            ],
            [
                "Suggestion",
                "label",
                "impl Into<Arc<str>>",
                "None",
                "Fret-only text convenience when you want to swap the visible copy without building custom child elements.",
            ],
            [
                "Suggestion",
                "variant / size / disabled / test_id / refine_style / refine_layout",
                "builder methods",
                "outline / sm / false / None / default chrome / default layout",
                "Recipe styling, diagnostics hooks, and layout refinements that map the upstream passthrough surface onto explicit Fret builders.",
            ],
        ],
        true,
    )
}
