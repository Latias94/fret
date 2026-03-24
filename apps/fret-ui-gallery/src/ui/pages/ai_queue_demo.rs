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
                "Queue",
                "Root container shell aligned with the AI Elements Queue chrome: rounded border + background + shadow. Layout negotiation (width/height/max-h) remains caller-owned via `refine_layout(...)`.",
            ],
            [
                "QueueSection",
                "Thin wrapper over shadcn `Collapsible`. Supports controlled (`Model<bool>`) and uncontrolled (`default_open`) lanes.",
            ],
            [
                "QueueSectionTrigger",
                "Trigger wrapper with hover chrome aligned to the upstream `bg-muted/40` header. Uses `a11y_label(...)` plus stable `test_id(...)` for diagnostics.",
            ],
            [
                "QueueSectionLabel",
                "Label row: chevron rotation, optional icon, and count+label text. Keeps muted foreground + medium weight (text-sm).",
            ],
            [
                "QueueSectionContent",
                "Thin wrapper over shadcn `CollapsibleContent` (force-mount supported via `QueueSection::force_mount_content(...)`).",
            ],
            [
                "QueueList",
                "Scrollable list container aligned with upstream `ScrollArea`: caller-owned max height (`max_height_px`) plus stable viewport `test_id` for deterministic wheel routing scripts.",
            ],
            [
                "QueueItem",
                "Hover region wrapper aligned with upstream `group` behavior (hover signal is inherited so descendants can read it).",
            ],
            [
                "QueueItemAction",
                "Icon action button aligned with upstream `opacity-0` + `group-hover:opacity-100`: by default it auto-follows the nearest `QueueItem` hovered state (no manual `.visible(st.hovered)` threading in snippets).",
            ],
            [
                "QueueItemIndicator / Content / Description",
                "Small primitives for status dots, clamped text, and optional description. Completed state applies muted opacity + strikethrough.",
            ],
            [
                "QueueItemAttachment / Image / File",
                "Attachment row + image/file chips aligned with upstream preview (32px image thumb, 100px truncation for file badges).",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_queue_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::queue_demo::render(cx);
    let prompt_input = snippets::queue_prompt_input_demo::render(cx);
    let features = doc_layout::notes_block([
        "Compound API: compose a Queue from small parts (sections, list, items, attachments).",
        "Hover-revealed per-row actions without changing layout (opacity-only).",
        "Completed/pending states: indicator + muted + strikethrough for content/description.",
        "Scroll routing: stable list viewport test ids for deterministic wheel scripts.",
        "Semantics: list/listitem grouping matches the upstream HTML intent.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "This alignment pass was public-surface drift, not a `crates/fret-ui` mechanism bug: the Queue demo no longer has to manually thread hover state into actions.",
        "The gallery page now mirrors the official Queue docs more closely: primary surface demo first, then the `With PromptInput` composition example from AI Elements.",
        "The current port matches upstream behavior outcomes, but does not yet animate opacity transitions (`transition-opacity`) for action reveal.",
        "Diagnostics gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-queue-demo-section-scroll-action.json` covers hover-reveal + remove action + scroll + collapse toggles.",
        "Diagnostics gate (PromptInput): `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-queue-demo-prompt-input.json` covers content-only section + todo remove + add attachments + submit + model selector close.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "A comprehensive queue component system for displaying message lists, todos, and collapsible task sections in AI applications.",
        ),
        vec![
            DocSection::build(cx, "Queue", demo)
                .test_id_prefix("ui-gallery-ai-queue-demo")
                .description(
                    "Docs-aligned Queue example with stable `test_id` anchors for diagnostics.",
                )
                .code_rust_from_file_region(snippets::queue_demo::SOURCE, "example"),
            DocSection::build(cx, "With PromptInput", prompt_input)
                .test_id_prefix("ui-gallery-ai-queue-prompt-input")
                .description(
                    "Rust/Fret analogue of the official AI Elements `queue-prompt-input` example: content-only section shell + PromptInput tools/model picker.",
                )
                .code_rust_from_file_region(snippets::queue_prompt_input_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Key behavior and API notes (mirrors the official AI Elements docs).")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Mapping from AI Elements parts to Fret surfaces.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Intentional divergences, layering guidance, and regression gates.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
