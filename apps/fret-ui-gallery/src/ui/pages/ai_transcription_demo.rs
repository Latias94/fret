use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn transcription_builder_surface_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Transcription",
                "new / from_arc",
                "builder",
                "empty / explicit segments",
                "Root provider for AI SDK-style transcript segments.",
            ],
            [
                "Transcription",
                "current_time_model / default_current_time",
                "Model<f32> / f32",
                "None / 0.0",
                "Controlled and uncontrolled playback position.",
            ],
            [
                "Transcription",
                "on_seek",
                "Arc<Fn(&mut UiActionHost, ActionCx, f32)>",
                "None",
                "Seek intent seam fired by segment activation. Playback transport stays app-owned.",
            ],
            [
                "Transcription",
                "test_id_root / refine_layout",
                "builder",
                "None / min-w-0",
                "Diagnostics hooks plus caller-owned layout negotiation.",
            ],
            [
                "Transcription",
                "into_element_with_children",
                "closure lane",
                "default `TranscriptionSegment` renderer",
                "Fret equivalent of the official render-props `children(segment, index)` API.",
            ],
            [
                "TranscriptionSegment",
                "new / test_id / text_style / on_activate / refine_layout",
                "builder",
                "shared small segment chrome",
                "Leaf segment styling plus extra activation behavior after the default seek callback.",
            ],
        ],
        true,
    )
}

fn transcription_docs_mapping_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Official docs concept", "Fret surface", "Notes"],
        [
            [
                "`segments`",
                "`TranscriptionSegmentData` via `new` / `from_arc`",
                "Same data contract; whitespace-only segments are filtered before rendering.",
            ],
            [
                "`currentTime`",
                "`current_time_model(...)` or `default_current_time(...)`",
                "Controlled or uncontrolled playback position, matching the upstream split.",
            ],
            [
                "`onSeek`",
                "`on_seek(...)`",
                "Seek intent only. Audio/video transport and actual media state stay app-owned.",
            ],
            [
                "render-props `children(segment, index)`",
                "`into_element_with_children(cx, |cx, segment, index| ...)`",
                "Closure stays context-safe and now numbers indexes in filtered rendered order, like upstream.",
            ],
            [
                "DOM `className` / `data-slot`",
                "`text_style(...)`, `refine_layout(...)`, `test_id_root(...)`, `test_id(...)`",
                "GPU UI keeps typed builder hooks and stable `test_id` anchors instead of raw DOM attrs.",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_transcription_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::transcription_demo::render(cx);
    let features = doc_layout::notes_block([
        "The usage snippet keeps the official AI Elements flow: app-owned playback time drives an interactive transcript, and clicking a segment seeks the transport.",
        "Whitespace-only segments are filtered before rendering, and custom-children indexes now follow the filtered rendered order so render-props logic stays aligned with upstream expectations.",
        "The default `TranscriptionSegment` still owns active / past / future coloring plus keyboard/button semantics, while `text_style(...)` remains an explicit leaf override for app-specific typography.",
        "This surface sits in `fret-ui-ai`, not `crates/fret-ui`: no runtime mechanism gap surfaced in this audit.",
    ]);
    let builder_surface = transcription_builder_surface_table(cx);
    let docs_mapping = transcription_docs_mapping_table(cx);
    let accessibility = doc_layout::notes_block([
        "Rendered segments stay semantic buttons, so keyboard activation and screen-reader button semantics remain available through the default `TranscriptionSegment` path.",
        "`TranscriptionSegment::on_activate(...)` composes after `on_seek(...)`, which lets apps add analytics or selection side effects without replacing the seek intent seam.",
        "Stable `test_id_root(...)` and per-segment `test_id(...)` hooks replace the upstream DOM `data-slot` teaching surface for diagnostics and scripted repros.",
        "When playback is unavailable, app code still owns disable policy and alternate copy; the transcript widget only represents the interactive transcript chrome.",
    ]);
    let notes = doc_layout::notes_block([
        "Audit conclusion: the meaningful component mismatch was render-props index numbering, not a `fret-ui` mechanism bug.",
        "The larger parity gap was the UI Gallery detail page itself: the old page showed a preview but did not explain the render-props lane, controlled/uncontrolled mapping, or the app-owned playback boundary.",
        "We intentionally keep this root on `into_element_with_children(...)` instead of adding an eager `.children(...)` builder because the provider context must exist before segment children resolve.",
        "If more parity work is needed later, the next likely targets are token polish or a richer custom-segment example, not new runtime contracts.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "The `Transcription` component is a docs-aligned interactive transcript surface for AI SDK-style segments. In Fret, playback timing and transport stay app-owned, while `fret-ui-ai` owns the transcript chrome, render-props lane, and seek intent seam.",
        ),
        vec![
            DocSection::build(cx, "Usage", demo)
                .test_id_prefix("ui-gallery-ai-transcription-demo")
                .description(
                    "Docs-aligned interactive transcript example: audio player state stays app-owned, and the transcript consumes `current_time` plus `on_seek` just like the official AI Elements example.",
                )
                .code_rust_from_file_region(snippets::transcription_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .test_id_prefix("ui-gallery-ai-transcription-features")
                .description("High-signal parity outcomes preserved in the current Fret port.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", builder_surface)
                .test_id_prefix("ui-gallery-ai-transcription-builder-surface")
                .description("Current Fret API surface for `Transcription` and `TranscriptionSegment`.")
                .no_shell(),
            DocSection::build(cx, "Docs Mapping", docs_mapping)
                .test_id_prefix("ui-gallery-ai-transcription-docs-mapping")
                .description("How the official AI Elements props and DOM-oriented teaching surface map onto Fret's typed GPU UI API.")
                .no_shell(),
            DocSection::build(cx, "Accessibility", accessibility)
                .test_id_prefix("ui-gallery-ai-transcription-accessibility")
                .description("Semantics, keyboard behavior, and diagnostics hooks that remain stable under the Fret port.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .test_id_prefix("ui-gallery-ai-transcription-notes")
                .description("Problem classification, parity conclusion, and the current stance on the composable children API.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
