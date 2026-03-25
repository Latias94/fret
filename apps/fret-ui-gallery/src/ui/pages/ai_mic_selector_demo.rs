use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn mic_selector_builder_surface_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                "new / children(...) / new_entries(...)",
                "builder / render-prop closure / explicit entries",
                "auto rows",
                "Supports automatic rows, explicit item entries, or a Rust render-prop equivalent of upstream `children(devices)`.",
            ],
            [
                "MicSelectorItem + MicSelectorEmpty",
                "new / value / children / empty",
                "builder",
                "label-as-value / \"No microphone found.\"",
                "Thin selector-level wrappers over shadcn command rows and empty-state text.",
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

fn mic_selector_hooks_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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

pub(super) fn preview_ai_mic_selector_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::mic_selector_demo::render(cx);

    let features = doc_layout::notes_block([
        "Behavior baseline is already healthy: controlled/uncontrolled value + open state, search/filtering, width sync, and close-on-select all match the intended AI Elements outcome.",
        "The root now exposes a docs-shaped `children([...])` lane, so the Rust example maps directly onto upstream `<MicSelector><MicSelectorTrigger /><MicSelectorContent /></MicSelector>` composition.",
        "The list surface now supports both a Rust render-prop builder (`children(...)`) and the older explicit `into_element_with_children(...)` escape hatch for host-generic call sites.",
        "Device labels ending in `(XXXX:XXXX)` still split the hardware ID into muted trailing text, matching the official `MicSelectorLabel` outcome.",
        "The Gallery preview keeps stable `test_id` anchors, and the existing diag script already covers open -> select -> close behavior for this page.",
    ])
    .test_id("ui-gallery-ai-mic-selector-demo-features");

    let behavior = doc_layout::notes_block([
        "This is not a `crates/fret-ui` mechanism bug. The mismatch here was authoring-surface and docs-surface parity in `ecosystem/fret-ui-ai` + UI Gallery.",
        "Upstream owns `navigator.mediaDevices` enumeration and permission prompting inside the component. Fret intentionally keeps those concerns in app code and only renders selector chrome plus selection intent.",
        "Trigger width is mirrored into the popover content, preserving the official docs outcome without baking page-level layout policy into `fret-ui`.",
        "The example keeps the selector visually centered with a `max-w-sm` style width cap on the trigger, matching the official docs composition more closely than the older full-width preview.",
    ])
    .test_id("ui-gallery-ai-mic-selector-demo-behavior");

    let props =
        mic_selector_builder_surface_table(cx).test_id("ui-gallery-ai-mic-selector-demo-props");
    let hooks = mic_selector_hooks_table(cx).test_id("ui-gallery-ai-mic-selector-demo-hooks");

    let notes = doc_layout::notes_block([
        "Diagnostics gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-mic-selector-demo-select.json` should keep passing with the same stable test IDs.",
        "The docs page intentionally documents app-owned device inventory instead of pretending Fret ports the browser-only `useAudioDevices()` hook verbatim.",
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
                    "Rust/Fret analogue of the official AI Elements Mic Selector example, now using a docs-shaped compound root plus typed trigger/content convenience lanes for `Value`, `Input`, and `List`.",
                )
                .description(
                    "The example stays deterministic and copyable by using app-owned mock devices instead of browser-only media APIs inside the preview.",
                )
                .test_id_prefix("ui-gallery-ai-mic-selector-demo")
                .code_rust_from_file_region(snippets::mic_selector_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("Behavior and composition outcomes preserved while aligning with the official Mic Selector docs surface.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", props)
                .description("Current Fret API surface for `MicSelector`, including the new docs-shaped root children lane and the render-prop list lane.")
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
