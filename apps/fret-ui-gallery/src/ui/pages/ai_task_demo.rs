use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_task_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::task_demo::render(cx);

    let features = doc_layout::notes_block([
        "Default Trigger matches the upstream source shape: search icon, muted small text, full-width row, and a Chevron that rotates when the collapsible opens.",
        "TaskContent keeps the docs-aligned left rule, top spacing, and stacked item rhythm instead of widening the mechanism layer.",
        "TaskItemFile maps the inline file chip from the official AI Elements source into a retained-layout pill with border, background, and smaller typography.",
        "Docs-shaped compound composition is now supported through `Task::root().children([TaskChild::Trigger(..), TaskChild::Content(..)])`, while `Task::new(trigger, content)` remains available as the explicit assembly seam.",
    ])
    .test_id("ui-gallery-ai-task-demo-features");

    let notes = doc_layout::notes_block([
        "Mechanism health looks good: the existing diag gate `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-task-demo-toggle.json` passes on the current branch, so the main drift was not in `crates/fret-ui`.",
        "The real mismatch was public-surface and teaching-surface parity. Upstream teaches nested JSX children, while Fret previously only exposed `Task::new(trigger, content)` in first-party examples.",
        "This Gallery page keeps the official docs shape but swaps the networked AI SDK example for local preset buttons so the preview stays deterministic, copyable, and backend-free.",
        "The AI Task page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` surfaces in UI Gallery.",
    ])
    .test_id("ui-gallery-ai-task-demo-notes");

    let props = task_props_table(cx).test_id("ui-gallery-ai-task-demo-props");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The `Task` family is a policy-level collapsible task-list surface in `fret-ui-ai`: semantics stay on shadcn/Radix `Collapsible`, while the docs-facing recipe owns trigger chrome, item typography, and file-pill presentation.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .description("Rust/Fret analogue of the official AI Elements Task example with docs-shaped compound composition.")
                .test_id_prefix("ui-gallery-ai-task-demo")
                .code_rust_from_file_region(snippets::task_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Behavior and authoring outcomes preserved while aligning against the official source and docs surface.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", props)
                .description("Current Fret API surface for `Task`, including the new compound children lane and the legacy explicit assembly lane.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Parity findings, layering decision, and why this alignment stays out of the runtime mechanism layer.")
                .no_shell(),
        ],
    );

    vec![body.into_element(cx)]
}

fn task_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Task",
                "new(trigger, content)",
                "TaskTrigger + TaskContent",
                "-",
                "Legacy explicit assembly lane. Still valid when you already have both parts in hand.",
            ],
            [
                "Task",
                "root()",
                "builder root",
                "-",
                "Docs-shaped compound root aligned with upstream `<Task>...</Task>` composition.",
            ],
            [
                "Task",
                "children([...])",
                "TaskChild",
                "-",
                "Typed child list for `TaskTrigger` + `TaskContent`; closest Fret analogue of nested JSX children.",
            ],
            [
                "Task",
                "trigger / content",
                "builder methods",
                "-",
                "Part-specific sugar on top of the compound root when you prefer a chained builder over an explicit child list.",
            ],
            [
                "Task",
                "default_open / open_model",
                "bool / Model<bool>",
                "true / None",
                "Uncontrolled or controlled open state mapped to the underlying collapsible policy surface.",
            ],
            [
                "TaskTrigger",
                "new(title)",
                "impl Into<Arc<str>>",
                "-",
                "Default trigger title used by the built-in search-row chrome.",
            ],
            [
                "TaskTrigger",
                "children([...])",
                "AnyElement",
                "None",
                "Overrides the visible trigger body while preserving the Task toggle behavior.",
            ],
            [
                "TaskContent",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Collapsible body container with the docs-aligned left border and stacked spacing.",
            ],
            [
                "TaskItem / TaskItemFile",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Task row content and optional inline file chip.",
            ],
            [
                "Hooks",
                "use_task_controller(cx)",
                "Option<TaskController>",
                "None outside `Task`",
                "Exposes the derived open model and `is_open` flag for lower-level custom descendants that need Task state.",
            ],
        ],
        true,
    )
}
