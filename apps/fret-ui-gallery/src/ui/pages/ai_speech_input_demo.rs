use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_speech_input_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::speech_input_demo::render(cx);
    let notes = doc_layout::notes(
        cx,
        [
            "Current parity gap is component-level polish rather than a fret-ui runtime bug: the existing diag script already proves the page loads and toggles reliably.",
            "The demo now mirrors the official AI Elements docs flow (idle hint → listening pulse → processing spinner → transcript result) while keeping capture/transcription backends app-owned.",
            "Upstream `SpeechInput` is still a leaf button surface, so we intentionally keep this API non-compound for now instead of adding a children-first composition layer.",
        ],
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Preview stays close to the official AI Elements docs example while preserving Fret's UI-only seam: apps own microphone capture, transcription, and browser capability decisions.",
        ),
        vec![
            DocSection::new("Speech Input", demo)
                .description("Docs-aligned speech input flow with transcript feedback and a deterministic processing phase.")
                .test_id_prefix("ui-gallery-ai-speech-input-demo")
                .code_rust_from_file_region(snippets::speech_input_demo::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Layering and parity notes for SpeechInput."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-speech-input-demo")]
}
