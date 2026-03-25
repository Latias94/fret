use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "VoiceSelector",
                "new / from_arc",
                "IntoIterator<Item = VoiceSelectorVoice> / Arc<[VoiceSelectorVoice]>",
                "-",
                "Root selector shell. Voice inventory stays app-owned in Fret instead of being fetched or previewed inside the widget.",
            ],
            [
                "VoiceSelector",
                "value_model / default_value",
                "Model<Option<Arc<str>>> / impl Into<Arc<str>>",
                "None",
                "Controlled and uncontrolled selected-voice value, aligned with upstream `value` / `defaultValue`.",
            ],
            [
                "VoiceSelector",
                "open_model / default_open",
                "Model<bool> / bool",
                "None / false",
                "Controlled and uncontrolled dialog open state, aligned with upstream `open` / `defaultOpen`.",
            ],
            [
                "VoiceSelector",
                "children([...]) / trigger(...) / content(...)",
                "VoiceSelectorChild",
                "-",
                "Docs-shaped compound root aligned with upstream `<VoiceSelector>...</VoiceSelector>` composition.",
            ],
            [
                "VoiceSelector",
                "into_element_with_children(cx, ...)",
                "slot closure",
                "-",
                "Lower-level escape hatch when trigger/content need to be built under a live scope instead of an eager child list.",
            ],
            [
                "VoiceSelectorTrigger",
                "new(child)",
                "AnyElement",
                "caller-owned child",
                "Dialog trigger wrapper. The child stays app-owned so the trigger can mirror the official `Button` composition instead of hard-coding selector chrome.",
            ],
            [
                "VoiceSelectorContent",
                "new(children) / input(...) / list(...)",
                "IntoIterator<Item = AnyElement> / typed builder",
                "DialogContent + Command shell",
                "Dialog body wrapper with typed `VoiceSelectorInput` and `VoiceSelectorList` lanes for docs-style composition.",
            ],
            [
                "VoiceSelectorInput",
                "new / placeholder",
                "builder",
                "\"Search voices...\"",
                "Search input bound to the shared query model.",
            ],
            [
                "VoiceSelectorList",
                "new / children(...) / new_entries(...)",
                "builder / render-prop closure / explicit entries",
                "auto rows",
                "Supports automatic rows, explicit `CommandEntry` lists, or a Rust render-prop equivalent of upstream `children(voices)` composition.",
            ],
            [
                "Group / Item / Separator / Shortcut / Dialog",
                "shared `Command*` / `Dialog` surfaces",
                "type aliases",
                "-",
                "These stay intentionally aligned with shared command/dialog behavior so selector parity does not fork overlay semantics.",
            ],
            [
                "Name / Description / Gender / Accent / Age / Attributes / Bullet / Preview",
                "new / value / children / refine_*",
                "builder",
                "selector metadata affordances",
                "Policy-level presentation parts for AI voice metadata, with children overrides on the upstream `children ?? default` surfaces.",
            ],
        ],
        true,
    )
}

fn hooks_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Surface", "Return", "Description"],
        [[
            "use_voice_selector_controller(cx)",
            "Option<VoiceSelectorController>",
            "Reads `voices`, `value`, `open`, and `query` from descendants under `VoiceSelector`. This is the Fret equivalent of upstream `useVoiceSelector()`, while keeping voice inventory and preview transport app-owned.",
        ]],
        true,
    )
}

pub(super) fn preview_ai_voice_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::voice_selector_demo::render(cx);
    let features = doc_layout::notes_block([
        "Behavior baseline is healthy: searchable selection, close-on-select, shared query highlight, and preview action embedding all live in `fret-ui-ai` / shared `Command` surfaces rather than `crates/fret-ui` mechanisms.",
        "The root now exposes a docs-shaped `children([...])` lane plus typed `input(...)` / `list(...)` content lanes, so the Rust example maps much more directly onto the official AI Elements composition.",
        "Leaf parts that upstream exposes as `children ?? default` now keep that escape hatch in Fret as well, which matters for icon/text overrides on `Gender`, `Accent`, `Bullet`, and `Preview`.",
        "Voice inventory and preview playback remain app-owned so the example stays backend-agnostic and copy-paste friendly.",
        "This detail page is feature-gated behind `gallery-dev`, which is also required for the wider `fret-ui-ai` surface in UI Gallery.",
    ])
    .test_id("ui-gallery-ai-voice-selector-demo-features");
    let parts = parts_table(cx);
    let hooks = hooks_table(cx).test_id("ui-gallery-ai-voice-selector-demo-hooks");
    let behavior = doc_layout::notes_block([
        "This is not a `crates/fret-ui` mechanism bug. The remaining drift was public-surface/docs-surface parity in `ecosystem/fret-ui-ai` + UI Gallery.",
        "Shared `Command*` and `Dialog` parts still own filtering, list semantics, and overlay behavior; `VoiceSelector` only layers voice-specific policy such as metadata affordances and preview chrome.",
        "Compared with the React docs, Fret keeps `use_voice_selector_controller(cx)` instead of the exact hook name `useVoiceSelector()`, but the descendant-state intent is the same.",
        "The only deliberately retained product seam is transport: audio preview playback and voice inventory fetching stay in app code instead of being hidden inside the component.",
    ])
    .test_id("ui-gallery-ai-voice-selector-demo-behavior");
    let notes = doc_layout::notes_block([
        "Diagnostics gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-voice-selector-demo-select.json` should keep passing with the same stable `test_id` anchors.",
        "The gallery snippet now acts as the first-party teaching surface for official-docs-style `VoiceSelector` composition under `gallery-dev`.",
        "If future parity work touches dismissal, focus restore, or overlay routing, re-check the runtime contract layer; this pass did not find evidence that those mechanisms are currently wrong for `VoiceSelector`.",
    ])
    .test_id("ui-gallery-ai-voice-selector-demo-notes");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Voice Selector coverage for AI Elements: shadcn `Dialog` + `Command` behavior stays intact, while voice inventory and preview playback remain explicit app responsibilities in Fret.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .descriptions([
                    "Rust/Fret analogue of the official AI Elements Voice Selector example, now using a docs-shaped compound root plus typed `VoiceSelectorContent::input(...)` / `list(...)` convenience lanes.",
                    "The example stays deterministic and copyable by keeping voice inventory and preview transport in app code instead of hiding them behind browser-only or provider-owned seams.",
                ])
                .test_id_prefix("ui-gallery-ai-voice-selector-demo")
                .code_rust_from_file_region(snippets::voice_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Behavior and composition outcomes preserved while aligning with the official Voice Selector docs surface.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Current Fret API surface for `VoiceSelector`, including the docs-shaped root children lane and the render-prop list lane.")
                .no_shell(),
            DocSection::build(cx, "Hooks", hooks)
                .description("Fret hook surface corresponding to descendant state access under `VoiceSelector`.")
                .no_shell(),
            DocSection::build(cx, "Behavior", behavior)
                .description("Layering and behavior notes that explain why this alignment stayed out of the runtime mechanism layer.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Diagnostics and remaining parity boundaries.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
