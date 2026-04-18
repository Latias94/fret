use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "MicSelector",
                "new / from_arc",
                "IntoIterator<Item = MicSelectorDevice> / Arc<[MicSelectorDevice]>",
                "-",
                "Root selector shell. Device inventory stays app-owned in Fret instead of being fetched inside the widget.",
            ],
            [
                "MicSelector",
                "value_model / default_value",
                "Model<Option<Arc<str>>> / impl Into<Arc<str>>",
                "None",
                "Controlled and uncontrolled selected-device value, aligned with upstream `value` / `defaultValue`.",
            ],
            [
                "MicSelector",
                "open_model / default_open",
                "Model<bool> / bool",
                "None / false",
                "Controlled and uncontrolled popover open state, aligned with upstream `open` / `defaultOpen`.",
            ],
            [
                "MicSelector",
                "children([...]) / trigger(...) / content(...)",
                "MicSelectorChild",
                "-",
                "Docs-shaped compound root aligned with upstream `<MicSelector>...</MicSelector>` composition.",
            ],
            [
                "MicSelector",
                "into_element_with_children(cx, ...)",
                "slot closure",
                "-",
                "Lower-level escape hatch when trigger/content need to be built under a live scope instead of an eager child list.",
            ],
            [
                "MicSelectorTrigger",
                "new(children) / value(...)",
                "IntoIterator<Item = AnyElement> / MicSelectorValue",
                "outline button + chevrons icon",
                "Trigger button wrapper. Accepts arbitrary children, or a typed `MicSelectorValue`, then appends the chevrons icon and mirrors trigger width into content.",
            ],
            [
                "MicSelectorContent",
                "new(children) / input(...) / list(...)",
                "IntoIterator<Item = AnyElement> / typed builder",
                "PopoverContent + Command shell",
                "Popover body wrapper with separate popover/command refinement surfaces plus typed convenience lanes for docs-shaped `MicSelectorInput` + `MicSelectorList` composition.",
            ],
            [
                "MicSelectorInput",
                "new / placeholder",
                "builder",
                "\"Search microphones...\"",
                "Search input bound to the shared query model.",
            ],
            [
                "MicSelectorList",
                "new / children(...) / new_entries(...) / entries(...)",
                "typed builder / docs-shaped render-prop / explicit entries",
                "auto rows",
                "Supports automatic typed rows, explicit selector/shared command entries, or a docs-shaped render-prop builder that receives `Arc<[MicSelectorDevice]>` without requiring a live `cx`.",
            ],
            [
                "MicSelectorItem",
                "new / value / child / children / on_select_action",
                "builder",
                "label-derived value",
                "Selector-owned row builder. Typed `MicSelectorLabel` is the primary child lane, and selection clears query + closes the popover by default unless explicitly overridden.",
            ],
            [
                "MicSelectorEmpty + MicSelectorLabel",
                "new / text / new(device)",
                "builder",
                "\"No microphone found.\" / parsed device label",
                "Empty-state text plus the typed label renderer that splits `(XXXX:XXXX)` hardware IDs into muted trailing text.",
            ],
            [
                "Hook",
                "use_mic_selector_controller(cx)",
                "Option<MicSelectorController>",
                "None outside `MicSelector`",
                "Reads shared devices/value/open/query state inside custom descendants.",
            ],
        ],
        true,
    )
}

fn hooks_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Surface", "Return", "Description"],
        [[
            "use_mic_selector_controller(cx)",
            "Option<MicSelectorController>",
            "Reads `devices`, `value`, `open`, and `query` from descendants under `MicSelector`. Unlike upstream `useAudioDevices()`, enumeration and permission prompts remain app-owned in Fret.",
        ]],
        true,
    )
}

pub(super) fn preview_ai_mic_selector_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::mic_selector_demo::render(cx);

    let features = doc_layout::notes_block([
        "Behavior baseline is already healthy: controlled/uncontrolled value + open state, search/filtering, width sync, and close-on-select all match the intended AI Elements outcome.",
        "The root now exposes a docs-shaped `children([...])` lane, so the Rust example maps directly onto upstream `<MicSelector><MicSelectorTrigger /><MicSelectorContent /></MicSelector>` composition.",
        "The list surface now teaches selector-owned typed rows first: `children(...)` receives devices only, while `MicSelectorItem` + `MicSelectorLabel` handle row construction without leaking `cx` or prebuilt `AnyElement` rows into app code.",
        "Advanced escape hatches still exist through explicit entries and the root `into_element_with_children(cx, ...)` compound lane, but they are no longer the default teaching surface for ordinary examples.",
        "Device labels ending in `(XXXX:XXXX)` still split the hardware ID into muted trailing text, matching the official `MicSelectorLabel` outcome.",
        "The Gallery preview keeps stable `test_id` anchors, and the existing diag script already covers open -> select -> close behavior for this page.",
    ])
    .test_id("ui-gallery-ai-mic-selector-demo-features");

    let behavior = doc_layout::notes_block([
        "This is not a `crates/fret-ui` mechanism bug. The mismatch here was authoring-surface and docs-surface parity in `ecosystem/fret-ui-ai` + UI Gallery.",
        "Upstream owns `navigator.mediaDevices` enumeration and permission prompting inside the component. Fret intentionally keeps those concerns in app code and only renders selector chrome plus selection intent.",
        "The render-prop seam is now data-only: app code supplies device inventory, while the selector-owned typed row builders own actual row composition and default selection behavior.",
        "Trigger width is mirrored into the popover content, preserving the official docs outcome without baking page-level layout policy into `fret-ui`.",
        "The example keeps the selector visually centered with a `max-w-sm` style width cap on the trigger, matching the official docs composition more closely than the older full-width preview.",
    ])
    .test_id("ui-gallery-ai-mic-selector-demo-behavior");

    let parts = parts_table(cx);
    let parts = parts.test_id("ui-gallery-ai-mic-selector-demo-props");
    let hooks = hooks_table(cx).test_id("ui-gallery-ai-mic-selector-demo-hooks");

    let notes = doc_layout::notes_block([
        "Diagnostics gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-mic-selector-demo-select.json` should keep passing with the same stable test IDs.",
        "The docs page intentionally documents app-owned device inventory instead of pretending Fret ports the browser-only `useAudioDevices()` hook verbatim.",
        "The root `into_element_with_children(cx, ...)` escape hatch remains available for advanced trigger/content composition under a live provider scope, but day-to-day examples should stay on `children([...])` plus typed list/item builders.",
        "If future parity work touches dismissal, focus restore, or overlay routing, re-check the runtime contract layer; this pass did not find evidence that those mechanisms are currently wrong for `MicSelector`.",
    ])
    .test_id("ui-gallery-ai-mic-selector-demo-notes");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Mic Selector coverage for AI Elements: shadcn `Popover` + `Command` behavior stays intact, while device enumeration and microphone permissions remain explicit app responsibilities in Fret.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .description(
                    "Rust/Fret analogue of the official AI Elements Mic Selector example, now using a docs-shaped compound root plus typed trigger/content/list/item lanes around `MicSelectorValue`, `MicSelectorInput`, `MicSelectorItem`, and `MicSelectorLabel`.",
                )
                .description(
                    "The example stays deterministic and copyable by using app-owned mock devices instead of browser-only media APIs inside the preview.",
                )
                .test_id_prefix("ui-gallery-ai-mic-selector-demo")
                .code_rust_from_file_region(snippets::mic_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Behavior and composition outcomes preserved while aligning with the official Mic Selector docs surface.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Current Fret API surface for `MicSelector`, including the docs-shaped root and selector-owned typed list/item builders.")
                .no_shell(),
            DocSection::build(cx, "Hooks", hooks)
                .description("Fret hook surface corresponding to descendant state access under `MicSelector`.")
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
