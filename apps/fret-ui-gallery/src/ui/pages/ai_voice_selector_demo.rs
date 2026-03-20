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
                "VoiceSelector",
                "UI root with controlled/uncontrolled `value_model` / `open_model` support plus a Rust-friendly `into_element_with_children(...)` compound entrypoint.",
            ],
            [
                "VoiceSelectorTrigger + Value/Button",
                "Trigger chrome, selected summary, and button composition stay split so apps can mirror the official docs layout without hiding policy in the runtime.",
            ],
            [
                "VoiceSelectorContent + Dialog",
                "Dialog shell with accessible title; `VoiceSelectorDialog` remains available for a command-palette style surface.",
            ],
            [
                "VoiceSelectorInput + List",
                "Shared query model, filtering, highlight, and close-on-select behavior are already encapsulated at the selector / Command ecosystem layer.",
            ],
            [
                "Empty / Group / Item / Separator / Shortcut",
                "These surfaces intentionally reuse `Command*` semantics so selector parity work stays aligned with shared command behavior instead of forking it.",
            ],
            [
                "Name / Description / Gender / Accent / Age / Attributes / Bullet / Preview",
                "Policy-level presentation parts for AI voice metadata. These are good selector-level affordances and should not move down into `crates/fret-ui`.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_voice_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::voice_selector_demo::render(cx);
    let features = doc_layout::notes_block([
        "This is the most complete AI selector compound surface in Fret today, and it is the right baseline for selector naming parity.",
        "The demo already covers searchable selection, grouped entries, metadata rows, and preview actions without leaking policy into the runtime layer.",
        "`VoiceSelectorEmpty`, `Group`, `Item`, `Separator`, and `Shortcut` deliberately stay aligned with shared `Command*` semantics.",
        "Voice inventory and preview playback remain app-owned so the example stays copy-paste friendly and backend-agnostic.",
        "Rust now uses the same root-level `into_element_with_children(...)` compound entrypoint as `MicSelector`, while still keeping voice-specific metadata parts explicit.",
    ]);
    let parts = parts_table(cx);
    let notes = doc_layout::notes_block([
        "Compared with the React docs, Rust exposes `use_voice_selector_controller(...)` rather than a `useVoiceSelector()` hook name, but the intent is the same: read selector context from descendants.",
        "If we later tighten parity further, the next place to invest is shared `Command` composition and docs examples, not `crates/fret-ui` mechanisms.",
        "This component is intentionally richer than `ModelSelector`; its metadata and preview parts are selector policy, not a universal contract every AI selector must copy.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned preview keeps the official compound taxonomy while intentionally leaving voice inventory and preview transport in app code.",
        ),
        vec![
            DocSection::build(cx, "Compound API", demo)
                .descriptions([
                    "Uses the same high-level trigger / content / input / list / item taxonomy as the official AI Elements `voice-selector` docs, now expressed with the same root-level `into_element_with_children(...)` pattern used by `MicSelector`.",
                    "Selector-specific metadata parts remain visible so this page demonstrates where policy ends, shared selector composition begins, and shared command semantics take over.",
                ])
                .test_id_prefix("ui-gallery-ai-voice-selector-demo")
                .code_rust_from_file_region(snippets::voice_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official AI Elements docs.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which surfaces are selector-owned versus shared with the underlying command layer.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Layering and next-step parity guidance.")
                .no_shell(),
        ],
    );

    vec![body.into_element(cx)]
}
