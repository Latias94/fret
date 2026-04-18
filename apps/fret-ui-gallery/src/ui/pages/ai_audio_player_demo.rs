use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn builder_surface_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Description"],
        [
            [
                "AudioPlayer",
                "element / control_bar / children",
                "Docs-shaped compound root aligned with upstream `<AudioPlayer>...</AudioPlayer>` composition.",
            ],
            [
                "AudioPlayer",
                "playing_model / muted_model / time_model / duration_secs_model / volume_model",
                "App-owned playback state models; Fret keeps media ownership outside the component chrome layer.",
            ],
            [
                "AudioPlayer",
                "into_element_with_children",
                "Low-level escape hatch when lazy controller-aware composition is still the best fit.",
            ],
            [
                "AudioPlayerElement",
                "src / speech_data / test_id",
                "Docs-aligned source builders for remote URLs or AI SDK-style speech data without embedding a playback backend.",
            ],
            [
                "AudioPlayerControlBar",
                "empty / children / play_button / seek_* / time_* / duration_display / mute_button / volume_range",
                "Typed control-bar composition surface that keeps all official slots inside one bar.",
            ],
            [
                "AudioPlayerTimeRange / VolumeRange",
                "refine_layout / test_id",
                "Sizing and diagnostics hooks for app-specific layout and scripted gates.",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_audio_player_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::audio_player_demo::render_speech_result(cx);
    let remote_audio = snippets::audio_player_remote_demo::render_remote_audio(cx);
    let features = doc_layout::notes_block([
        "Layering conclusion: no new `crates/fret-ui` mechanism gap surfaced here; the meaningful drift was public authoring surface plus the UI Gallery docs page shape.",
        "The root and control bar now expose docs-shaped compound builders, so the first-party example no longer has to split the time row out of the control bar.",
        "Time and range slots now live inside the same grouped chrome lane, which is closer to the official AI Elements `ButtonGroup` composition outcome.",
        "Playback remains app-owned in Fret today. `AudioPlayerElement` now keeps docs-aligned `src` and `speech_data` builders so the teaching surface can stay close to upstream.",
    ]);
    let builder_surface = builder_surface_table(cx);
    let notes = doc_layout::notes_block([
        "Keep `ui-ai-audio-player-demo-play`, `ui-ai-audio-player-demo-mute`, and `ui-ai-audio-player-demo-time` stable; the promoted diag gate depends on them.",
        "This detail page is gated behind `gallery-dev`, which is also required for the wider `fret-ui-ai` UI Gallery surface.",
        "Because Fret is GPU-first and move-only, React `children` becomes typed builder methods and explicit child enums instead of raw DOM prop passthrough.",
        "The preview still uses app-driven models for time, mute, volume, and playing state; that boundary matches the current Fret architecture rather than media-chrome's browser-owned runtime.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "The `AudioPlayer` component is a composable audio chrome surface aligned with the official AI Elements docs. In Fret, apps still own actual playback and feed the player models.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .test_id_prefix("ui-gallery-ai-audio-player-demo")
                .description(
                    "Docs-aligned AI SDK speech-result example using the compound root + control-bar builders.",
                )
                .code_rust_from_file_region(snippets::audio_player_demo::SOURCE, "example"),
            DocSection::build(cx, "Remote Audio", remote_audio)
                .test_id_prefix("ui-gallery-ai-audio-player-remote")
                .description(
                    "Remote URL variant matching the official AI Elements docs structure while keeping playback app-owned.",
                )
                .code_rust_from_file_region(
                    snippets::audio_player_remote_demo::SOURCE,
                    "example",
                ),
            DocSection::build(cx, "Features", features)
                .test_id_prefix("ui-gallery-ai-audio-player-features")
                .description("High-signal parity notes against the official AI Elements docs surface.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", builder_surface)
                .test_id_prefix("ui-gallery-ai-audio-player-builder-surface")
                .description("Current Fret API surface for the `AudioPlayer*` family.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .test_id_prefix("ui-gallery-ai-audio-player-notes")
                .description("Parity findings, diagnostics anchors, and the current app-owned playback boundary.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
