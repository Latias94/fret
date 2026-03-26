use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn speech_input_builder_surface_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "SpeechInput",
                "new",
                "builder",
                "-",
                "Leaf voice-input button aligned with the official AI Elements `SpeechInput` chrome while keeping capture and transcription outside the widget.",
            ],
            [
                "SpeechInput",
                "listening_model / default_listening",
                "Model<bool> / bool",
                "None / false",
                "Controlled and uncontrolled listening state. Drives microphone vs stop icon plus the pulse-ring animation.",
            ],
            [
                "SpeechInput",
                "processing_model / default_processing",
                "Model<bool> / bool",
                "None / false",
                "Controlled and uncontrolled processing state. Shows the spinner and blocks repeated activation while transcription is pending.",
            ],
            [
                "SpeechInput",
                "on_listening_change",
                "Arc<Fn(&mut UiActionHost, ActionCx, bool)>",
                "None",
                "Intent seam fired when the button toggles. App code starts/stops capture and kicks off transcription work here.",
            ],
            [
                "SpeechInput",
                "variant / size / style / refine_style / refine_layout",
                "builder",
                "default / icon / no extra overrides",
                "Button chrome and layout overrides. This is the Rust/Fret equivalent of the upstream `Button`-prop customization lane.",
            ],
            [
                "SpeechInput",
                "disabled / focusable / a11y_label / test_id",
                "builder",
                "false / true / dynamic start-stop label / None",
                "Accessibility and diagnostics hooks for app-specific policy and deterministic UI Gallery automation.",
            ],
            [
                "SpeechInput",
                "children API",
                "not exposed",
                "-",
                "Intentionally remains a leaf button surface. Compose transcript, clear action, capability copy, and backend status outside the component just like the upstream docs do.",
            ],
        ],
        true,
    )
}

fn speech_input_backend_mapping_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Concern", "Official AI Elements", "Fret mapping"],
        [
            [
                "Backend selection",
                "Web Speech API vs MediaRecorder is detected inside the component.",
                "App code decides which capture/transcription backend exists, then drives `disabled`, `listening_model`, and `processing_model` explicitly.",
            ],
            [
                "Transcription callback",
                "`onTranscriptionChange(text)` receives final transcript chunks.",
                "The app updates its own transcript model after backend work completes; the demo uses a deterministic timer to simulate that handoff.",
            ],
            [
                "Recorded-audio fallback",
                "`onAudioRecorded(audioBlob)` handles Firefox/Safari fallback.",
                "Stopping capture and enqueueing transcription remains app-owned; `SpeechInput` only emits the start/stop intent.",
            ],
            [
                "Language / backend config",
                "`lang` is a component prop because the browser speech backend lives inside the widget.",
                "Language and backend options belong to the app-owned capture/transcription service, not the UI chrome component.",
            ],
            [
                "Unsupported environments",
                "The component disables itself when no supported browser API is available.",
                "Apps decide capability and can disable the widget or show fallback copy without teaching browser-specific policy inside `fret-ui-ai`.",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_speech_input_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::speech_input_demo::render(cx);
    let features = doc_layout::notes_block([
        "The preview follows the same user-visible flow as the official AI Elements docs example: idle hint -> listening pulse -> processing spinner -> transcript result.",
        "The button chrome still tracks upstream expectations: rounded icon button, microphone icon at rest, stop icon while listening, and disabled processing state.",
        "Capture, browser capability checks, and transcription remain app-owned in Fret, so this page teaches the adaptation boundary instead of pretending the browser runtime lives inside the component.",
        "The promoted diag script already covers the key interaction path for this page and passed in the current audit run.",
    ]);
    let builder_surface = speech_input_builder_surface_table(cx);
    let backend_mapping = speech_input_backend_mapping_table(cx);
    let accessibility = doc_layout::notes_block([
        "The control stays a semantic button with keyboard activation inherited from the shadcn button surface.",
        "The default accessible label switches between `Start recording` and `Stop recording`, so icon-only usage still announces the current action.",
        "Stable `test_id` anchors exist for the button, processing hint, transcript surface, and clear action, which keeps diag coverage deterministic.",
    ]);
    let notes = doc_layout::notes_block([
        "Audit conclusion: no `crates/fret-ui` mechanism bug surfaced here. The current component and demo behavior are already in the right layer.",
        "The bigger parity gap was the docs surface itself: the old detail page did not explain how official browser-owned props map onto Fret's app-owned backend boundary.",
        "We intentionally keep `SpeechInput` non-compound for now. Upstream treats it as a leaf button surface, so adding a children-first API would create extra policy without improving parity.",
        "If future parity work is needed, the next likely target is richer first-party backend teaching snippets or prompt-composer integration, not a runtime contract change.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Speech Input coverage for AI Elements: the voice-input chrome lives in `fret-ui-ai`, while browser capability decisions, microphone capture, and transcription remain explicit app responsibilities in Fret.",
        ),
        vec![
            DocSection::build(cx, "Usage", demo)
                .description(
                    "Rust/Fret analogue of the official AI Elements `SpeechInput` example with transcript feedback and a deterministic processing phase.",
                )
                .description(
                    "The preview stays copyable by simulating app-owned capture/transcription instead of embedding browser-only APIs into the gallery surface.",
                )
                .test_id_prefix("ui-gallery-ai-speech-input-demo")
                .code_rust_from_file_region(snippets::speech_input_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description(
                    "High-signal behavior and composition outcomes preserved while aligning with the official Speech Input docs surface.",
                )
                .no_shell(),
            DocSection::build(cx, "Builder Surface", builder_surface)
                .description(
                    "Current Fret API surface for `SpeechInput`, including the explicit ownership boundary versus the upstream browser-backed component.",
                )
                .no_shell(),
            DocSection::build(cx, "Backend Mapping", backend_mapping)
                .description(
                    "How the official browser/runtime responsibilities translate into Fret's app-owned capture and transcription boundary.",
                )
                .no_shell(),
            DocSection::build(cx, "Accessibility", accessibility)
                .description(
                    "Semantics, keyboard activation, and automation hooks that remain available even though backends stay outside the widget.",
                )
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Parity conclusion, diagnostics evidence, and the current stance on compound children.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
