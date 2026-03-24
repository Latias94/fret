use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                "new() | children(...) | copy_format(...) | on_copy(...) | timeout(...)",
                "Custom button content is now supported. `onError` is still intentionally absent because clipboard writes remain fire-and-forget effects in Fret.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_environment_variables_demo(
    cx: &mut UiCx<'_>,
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
        "机制层看起来是健康的：显示/隐藏状态、复制反馈和可选文本切换都还停留在 `fret-ui-ai` 组件层，没有发现需要下沉到 `crates/fret-ui` 的问题。",
        "当前主要问题在公共 surface 和教学面：`EnvironmentVariablesTitle` 之外，`EnvironmentVariableName` / `Value` / `Required` / `CopyButton` 现在也补上了 upstream-style children 覆盖能力。",
        "根和行仍然保持 `into_element_with_children(...)` 的闭包式 authoring，这不是机制缺陷，而是为了先安装 provider/context，再让子部件读取继承状态。",
        "自定义 `EnvironmentVariableValue` children 会接管可见内容，不再走默认的 mask/unmask 文本路径，这与官方实现的 `children ?? displayValue` 语义一致。",
        "这页仍然要求 `gallery-dev` feature，AI Elements 的 wider surface 也依赖这个开关。",
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
