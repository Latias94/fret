use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;

fn parts_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |part: &'static str, surface: &'static str| {
        shadcn::TableRow::build(2, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(surface)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Fret surface").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("MicSelector", "UI-only root. Controlled state uses `value_model` / `open_model`; uncontrolled flows use `default_value` / `default_open`."));
                out.push_ui(cx, row("MicSelectorTrigger", "Outline button trigger. Accepts arbitrary children, appends the chevrons icon, and anchors content width."));
                out.push_ui(cx, row("MicSelectorValue", "Shows the selected device or placeholder text. Trailing `(XXXX:XXXX)` IDs are split and muted like upstream."));
                out.push_ui(cx, row("MicSelectorContent", "Popover content + Command shell. Exposes separate popover and command refinement surfaces."));
                out.push_ui(cx, row("MicSelectorInput", "Search field bound to the shared query model."));
                out.push_ui(cx, row("MicSelectorList", "Supports auto rows, explicit `new_entries(...)`, and a Rust closure-based `into_element_with_children(...)` equivalent for upstream `children(data)` composition."));
                out.push_ui(cx, row("MicSelectorItem + MicSelectorEmpty", "Thin selector-level wrappers over list row / empty-state outcomes. They add explicit AI Elements-style parts without moving behavior into `crates/fret-ui`."));
            }),
        );
    })
    .into_element(cx)
}

pub(super) fn preview_ai_mic_selector_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::mic_selector_demo::render(cx);
    let features = doc_layout::notes(
        cx,
        [
            "Controlled and uncontrolled selection / open state are already covered at the component layer.",
            "Trigger width is mirrored into the popover content, matching the official docs outcome.",
            "Search, filtering, close-on-select, explicit item composition, and `(XXXX:XXXX)` device label parsing are now all covered at the selector surface.",
            "The gallery now uses a Rust-native compound entrypoint plus a `MicSelectorList::into_element_with_children(...)` closure, so the example reads closer to the official docs composition.",
            "A stable diag script already covers select + close behavior for this page.",
        ],
    );
    let parts = parts_table(cx);
    let notes = doc_layout::notes(
        cx,
        [
            "This is not a `crates/fret-ui` mechanism bug. The remaining work is ecosystem API / docs parity.",
            "By design, Fret keeps device enumeration, permission prompts, and `devicechange` handling app-owned; `MicSelector` only renders UI chrome and emits selection intent.",
            "The main composition gap is now closed at the selector surface with a Rust closure-based `MicSelectorList::into_element_with_children(...)`, without pushing new policy into runtime contracts.",
        ],
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned preview keeps the AI Elements compound shape while intentionally leaving device enumeration and microphone permissions in app code.",
        ),
        vec![
            DocSection::new("Compound API", demo)
                .descriptions([
                    "Uses the same trigger / value / content / input / list decomposition as the official AI Elements docs, now with explicit empty/item parts at the selector surface.",
                    "Rust expresses the compound example with `into_element_with_children(...)` on both the root and list surfaces, giving a close equivalent to JSX nesting plus `children(data)` render props.",
                ])
                .test_id_prefix("ui-gallery-ai-mic-selector-demo")
                .code_rust_from_file_region(snippets::mic_selector_demo::SOURCE, "example"),
            DocSection::new("Features", features)
                .description("High-signal parity notes against the official AI Elements docs.")
                .no_shell(),
            DocSection::new("Parts & Props", parts)
                .description("Which layer owns what, and where the current Rust surface still differs from upstream.")
                .no_shell(),
            DocSection::new("Notes", notes)
                .description("Layering and next-step parity notes.")
                .no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-mic-selector-demo")]
}
