use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_reasoning_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::reasoning_demo::render(cx);
    let hooks = snippets::reasoning_hooks::render(cx);

    let compare = doc_layout::notes_block([
        "Use `Reasoning` when the model returns a single block/stream of thinking content.",
        "If your model yields discrete, labeled steps (search queries, tool calls, phases), prefer `ChainOfThought` for a more structured trace.",
    ])
    .test_id("ui-gallery-ai-reasoning-demo-compare");

    let features = doc_layout::notes_block([
        "Auto-opens when streaming starts, then auto-closes once streaming ends (with a small delay).",
        "Default Trigger mirrors the AI Elements affordance: Brain icon, thinking copy, and a rotating Chevron.",
        "Reasoning stays manually toggleable; custom triggers can read `use_reasoning_controller(cx)` and drive the derived open model directly.",
        "Content renders markdown via `fret_markdown` and defaults to inert links (reasoning is usually non-interactive).",
        "Docs-shaped compound composition is supported via `Reasoning::trigger(...).content(...)` (or `children([ReasoningChild::...])`).",
    ])
    .test_id("ui-gallery-ai-reasoning-demo-features");

    let props = reasoning_props_table(cx).test_id("ui-gallery-ai-reasoning-demo-props");

    let notes = doc_layout::notes_block([
        "This is a policy-level composition in `ecosystem/fret-ui-ai`, built on shadcn `Collapsible` primitives (not a `crates/fret-ui` runtime contract surface).",
        "Diagnostics gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-reasoning-demo-auto-open-close.json` exercises the auto-open/close behavior.",
        "Gallery docs-smoke gate should keep `ui-gallery-ai-reasoning-demo-*` section IDs and the `ui-ai-reasoning-hooks-*` preview IDs stable.",
        "The Gallery preview simulates streaming with a local model toggle; in an app, wire `is_streaming` to your chat status and pass consolidated reasoning text to `ReasoningContent`.",
        "Upstream `useReasoning()` maps to `use_reasoning_controller(cx)` in Fret. Because Fret's typed child lists are eager, the hook is most useful in the lower-level `Reasoning::into_element(cx, trigger, content)` lane for state-aware custom triggers.",
    ])
    .test_id("ui-gallery-ai-reasoning-demo-notes");
    let usage_section = DocSection::build(cx, "Usage", demo)
        .test_id_prefix("ui-gallery-ai-reasoning-demo")
        .description(
            "Minimal streaming-driven Reasoning preview with docs-shaped children composition.",
        )
        .code_rust_from_file_region(snippets::reasoning_demo::SOURCE, "example");
    let compare_section = DocSection::build(cx, "Reasoning vs Chain of Thought", compare)
        .description("Quick guidance for picking the right disclosure surface.")
        .no_shell();
    let features_section = DocSection::build(cx, "Features", features)
        .description("Behavior and composition notes mapped from the official docs.")
        .no_shell();
    let props_section = DocSection::build(cx, "Props", props)
        .description("Fret API surface for `fret_ui_ai::Reasoning*` builders.")
        .no_shell();
    let hooks_section = DocSection::build(cx, "Hooks", hooks)
        .description(
            "Fret equivalent of upstream `useReasoning()` for custom trigger/content authoring.",
        )
        .test_id_prefix("ui-gallery-ai-reasoning-demo-hooks")
        .code_rust_from_file_region(snippets::reasoning_hooks::SOURCE, "example");
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Layering, diagnostics, and how to wire the demo to real streaming state.")
        .no_shell();

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Reasoning disclosure: streaming-driven auto-open/close plus a compound Trigger/Content composition surface.",
        ),
        vec![
            usage_section,
            compare_section,
            features_section,
            props_section,
            hooks_section,
            notes_section,
        ],
    );

    vec![body.into_element(cx)]
}

fn reasoning_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Reasoning",
                "new(is_streaming)",
                "bool",
                "false",
                "Root disclosure container (auto-open/close is driven by `is_streaming`).",
            ],
            [
                "Reasoning",
                "open(open_model)",
                "Model<bool>",
                "None",
                "Controlled open state (Radix `open`).",
            ],
            [
                "Reasoning",
                "default_open(value)",
                "Option<bool>",
                "None",
                "Uncontrolled initial open. `None` defaults to `is_streaming`; `Some(false)` also suppresses auto-open when streaming begins.",
            ],
            [
                "Reasoning",
                "duration_secs(value)",
                "Option<u32>",
                "None",
                "Override the duration displayed by the Trigger (AI Elements `duration`).",
            ],
            [
                "Reasoning",
                "trigger / content",
                "builder methods",
                "-",
                "Docs-shaped compound composition for Trigger + Content (preferred teaching surface).",
            ],
            [
                "Reasoning",
                "children([...])",
                "ReasoningChild",
                "-",
                "Lower-level typed child list (use `ReasoningChild::Trigger(..)` / `ReasoningChild::Content(..)`).",
            ],
            [
                "Reasoning",
                "into_element(cx, trigger, content)",
                "FnOnce + FnOnce",
                "-",
                "Lower-level escape hatch when parts must be built under a live scope.",
            ],
            [
                "ReasoningTrigger",
                "children([...])",
                "AnyElement",
                "None",
                "Overrides the full Trigger body (JSX children-style).",
            ],
            [
                "ReasoningTrigger",
                "thinking_children([...])",
                "AnyElement",
                "None",
                "Fret convenience: overrides only the thinking message slot while keeping default icons.",
            ],
            [
                "ReasoningContent",
                "new(markdown)",
                "impl Into<Arc<str>>",
                "-",
                "Markdown content body (rendered via `fret_markdown`).",
            ],
            [
                "Hooks",
                "use_reasoning_controller(cx)",
                "Option<ReasoningController>",
                "None outside `Reasoning`",
                "Reads the derived open model plus `is_open`, `is_streaming`, and `duration_secs` from custom descendants.",
            ],
        ],
        true,
    )
}
