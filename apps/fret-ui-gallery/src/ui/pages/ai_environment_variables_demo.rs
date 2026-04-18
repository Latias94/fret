use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_props_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Key inputs", "Notes"],
        [
            [
                "EnvironmentVariables",
                "show_values_model(...) | default_show_values(...) | on_show_values_change(...) | into_element_with_children(...)",
                "Root provider for visibility state. The closure-based composition stays intentional in Fret because provider context must exist before parts resolve inherited state.",
            ],
            [
                "EnvironmentVariablesHeader / Content",
                "new(children)",
                "Header keeps the border/shell chrome; content owns the row dividers.",
            ],
            [
                "EnvironmentVariablesTitle",
                "new() | text(...) | new_children(...)",
                "Defaults to the upstream label and now exposes a heading-level children override lane.",
            ],
            [
                "EnvironmentVariablesToggle",
                "new() | a11y_label(...) | test_id_*",
                "Switch stays bound to the shared visibility model; icon + switch test ids remain stable for diagnostics.",
            ],
            [
                "EnvironmentVariable",
                "new(name, value) | into_element_with_children(...)",
                "Row provider for name/value context. Default content still matches the official preview composition.",
            ],
            [
                "EnvironmentVariableName / Value / Required",
                "new() | text(...) | children(...)",
                "Leaf overrides now support composable children. Custom value children intentionally bypass the built-in mask/unmask display, matching upstream `children ?? displayValue` semantics.",
            ],
            [
                "EnvironmentVariableCopyButton",
                "new() | children(...) | copy_format(...) | on_copy(...) | on_error(...) | timeout(...)",
                "Custom button content is now supported, and copy callbacks are driven by real clipboard success/failure outcomes.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_environment_variables_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::environment_variables_demo::render(cx);
    let custom_children = snippets::environment_variables_custom_children::render(cx);
    let features = doc_layout::notes_block([
        "Value masking by default",
        "Toggle visibility switch",
        "Copy individual values",
        "Export format support (export KEY=\"value\")",
        "Required badge indicator",
    ]);
    let findings = doc_layout::notes_block([
        "Mechanism health looks good here: show/hide state, copy feedback, and selectable text toggling all remain in `fret-ui-ai`, with no evidence that the behavior should move into `crates/fret-ui`.",
        "The remaining work was public-surface and teaching alignment: beyond `EnvironmentVariablesTitle`, `EnvironmentVariableName` / `Value` / `Required` / `CopyButton` now expose upstream-style children overrides.",
        "The root and row surfaces intentionally keep `into_element_with_children(...)` because provider/context state must exist before descendant parts resolve inherited values.",
        "Custom `EnvironmentVariableValue` children take over the visible content instead of the default mask/unmask path, matching the official `children ?? displayValue` semantics.",
        "`EnvironmentVariableCopyButton` now exposes `on_error`, so copy feedback can remain completion-driven on failure as well as success.",
        "This page still requires the `gallery-dev` feature, which also gates the broader AI Elements surface in UI Gallery.",
    ]);
    let props = parts_props_table(cx);

    let demo_section = DocSection::build(cx, "Example", demo)
        .description(
            "Rust/Fret analogue of the official AI Elements Environment Variables example.",
        )
        .test_id_prefix("ui-gallery-ai-environment-variables-demo")
        .code_rust_from_file_region(snippets::environment_variables_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features).no_shell();
    let custom_children_section = DocSection::build(cx, "Custom Children", custom_children)
        .description(
            "Shows the upstream-style leaf override lane, including title/name/value/required/copy custom children.",
        )
        .test_id_prefix("ui-gallery-ai-environment-variables-custom-children")
        .code_rust_from_file_region(
            snippets::environment_variables_custom_children::SOURCE,
            "example",
        );
    let props_section = DocSection::build(cx, "Props", props).no_shell();
    let notes_section = DocSection::build(cx, "Notes", findings)
        .description("Layering + parity findings for Environment Variables.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Display environment variables with default masking, a visibility toggle, and per-row copy actions.",
        ),
        vec![
            demo_section,
            features_section,
            custom_children_section,
            props_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
