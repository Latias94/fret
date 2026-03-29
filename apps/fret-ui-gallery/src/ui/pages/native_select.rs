use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::native_select as snippets;

pub(super) fn preview_native_select(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let groups = snippets::with_groups::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let rtl = snippets::rtl::render(cx);

    let native_select_vs_select = doc_layout::notes_block([
        "Use `NativeSelect` when you want the simpler native-form-control authoring shape and a path toward backend-native picker semantics later.",
        "Use `Select` when you need richer custom overlays, search/filtering, icons inside items, or broader menu-style composition today.",
        "In the current Fret implementation, `NativeSelect` is a popover-backed fallback that preserves the shadcn surface while platform-native pickers remain future work.",
    ]);

    let api_reference = doc_layout::notes_block([
        "`native_select(model, open)` is the default controlled authoring helper, while `new_controllable(...)` stays available for controlled/uncontrolled bridging and default-value/open setup.",
        "`options(...)` and `optgroups(...)` are the source-aligned structured equivalent of upstream `NativeSelectOption` and `NativeSelectOptGroup` children, so no extra generic children API is needed here.",
        "`size(...)`, `disabled(...)`, `required(...)`, `aria_invalid(...)`, `control_id(...)`, `placeholder(...)`, and `a11y_label(...)` cover the documented control surface.",
        "Trigger chrome, chevron icon, default heights (`default` / `sm`), and invalid/focus states remain recipe-owned; surrounding width caps and form/page layout remain caller-owned.",
        "True backend-native parity remains deferred until platform-native select widgets are in scope.",
    ]);
    let native_select_vs_select = DocSection::build(
        cx,
        "Native Select vs Select",
        native_select_vs_select,
    )
    .description(
        "Pick the simpler native-style surface only when you do not need the richer custom select recipe.",
    )
    .no_shell()
    .test_id_prefix("ui-gallery-native-select-vs-select");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .description("Public surface summary, ownership notes, and defer rationale.")
        .no_shell()
        .test_id_prefix("ui-gallery-native-select-api-reference");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "A styled native-select-like control following the upstream top-of-page example.",
        )
        .no_shell()
        .test_id_prefix("ui-gallery-native-select-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `native_select(model, open)`.")
        .test_id_prefix("ui-gallery-native-select-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Organize options with `NativeSelectOptGroup`.")
        .test_id_prefix("ui-gallery-native-select-groups")
        .code_rust_from_file_region(snippets::with_groups::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disable the control with `disabled(true)`.")
        .test_id_prefix("ui-gallery-native-select-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description("Show validation state with `aria_invalid(true)`.")
        .test_id_prefix("ui-gallery-native-select-invalid")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider + popup alignment under RTL.")
        .test_id_prefix("ui-gallery-native-select-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description(
            "Use `FieldLabel::for_control` plus `NativeSelect::control_id` when you want an explicit label-click example outside the upstream docs path.",
        )
        .test_id_prefix("ui-gallery-native-select-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Native Select docs path first: Demo, Usage, Groups, Disabled, Invalid, Native Select vs Select, RTL, then keeps `Label Association` and `API Reference` as focused Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            groups,
            disabled,
            invalid,
            native_select_vs_select,
            rtl,
            label,
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-native-select").into_element(cx)]
}
