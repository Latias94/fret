use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::native_select as snippets;

pub(super) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct NativeSelectPageModels {
        demo_value: Option<Model<Option<Arc<str>>>>,
        demo_open: Option<Model<bool>>,
        groups_value: Option<Model<Option<Arc<str>>>>,
        groups_open: Option<Model<bool>>,
        disabled_value: Option<Model<Option<Arc<str>>>>,
        disabled_open: Option<Model<bool>>,
        invalid_value: Option<Model<Option<Arc<str>>>>,
        invalid_open: Option<Model<bool>>,
    }

    let (
        demo_value,
        demo_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        invalid_value,
        invalid_open,
    ) = cx.with_state(NativeSelectPageModels::default, |st| {
        (
            st.demo_value.clone(),
            st.demo_open.clone(),
            st.groups_value.clone(),
            st.groups_open.clone(),
            st.disabled_value.clone(),
            st.disabled_open.clone(),
            st.invalid_value.clone(),
            st.invalid_open.clone(),
        )
    });

    let (
        demo_value,
        demo_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        invalid_value,
        invalid_open,
    ) = match (
        demo_value,
        demo_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        invalid_value,
        invalid_open,
    ) {
        (
            Some(demo_value),
            Some(demo_open),
            Some(groups_value),
            Some(groups_open),
            Some(disabled_value),
            Some(disabled_open),
            Some(invalid_value),
            Some(invalid_open),
        ) => (
            demo_value,
            demo_open,
            groups_value,
            groups_open,
            disabled_value,
            disabled_open,
            invalid_value,
            invalid_open,
        ),
        _ => {
            let models = cx.app.models_mut();
            let demo_value = models.insert(None);
            let demo_open = models.insert(false);
            let groups_value = models.insert(None);
            let groups_open = models.insert(false);
            let disabled_value = models.insert(None);
            let disabled_open = models.insert(false);
            let invalid_value = models.insert(None);
            let invalid_open = models.insert(false);
            cx.with_state(NativeSelectPageModels::default, |st| {
                st.demo_value = Some(demo_value.clone());
                st.demo_open = Some(demo_open.clone());
                st.groups_value = Some(groups_value.clone());
                st.groups_open = Some(groups_open.clone());
                st.disabled_value = Some(disabled_value.clone());
                st.disabled_open = Some(disabled_open.clone());
                st.invalid_value = Some(invalid_value.clone());
                st.invalid_open = Some(invalid_open.clone());
            });
            (
                demo_value,
                demo_open,
                groups_value,
                groups_open,
                disabled_value,
                disabled_open,
                invalid_value,
                invalid_open,
            )
        }
    };

    let demo = snippets::demo::render(cx, demo_value, demo_open);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let groups = snippets::with_groups::render(cx, groups_value, groups_open);
    let disabled = snippets::disabled::render(cx, disabled_value, disabled_open);
    let invalid = snippets::invalid::render(cx, invalid_value, invalid_open);
    let rtl = snippets::rtl::render(cx);

    let native_select_vs_select = doc_layout::notes(
        cx,
        [
            "Use `NativeSelect` when you want the simpler native-form-control authoring shape and a path toward backend-native picker semantics later.",
            "Use `Select` when you need richer custom overlays, search/filtering, icons inside items, or broader menu-style composition today.",
            "In the current Fret implementation, `NativeSelect` is a popover-backed fallback that preserves the shadcn surface while platform-native pickers remain future work.",
        ],
    );

    let api_reference = doc_layout::notes(
        cx,
        [
            "`NativeSelect::new(model, open)` and `new_controllable(...)` cover the controlled and default-value/open authoring paths.",
            "`options(...)` and `optgroups(...)` are the source-aligned structured equivalent of upstream `NativeSelectOption` and `NativeSelectOptGroup` children, so no extra generic children API is needed here.",
            "`size(...)`, `disabled(...)`, `aria_invalid(...)`, `control_id(...)`, `placeholder(...)`, and `a11y_label(...)` cover the documented control surface.",
            "Trigger chrome, chevron icon, default heights (`default` / `sm`), and invalid/focus states remain recipe-owned; surrounding width caps and form/page layout remain caller-owned.",
            "True backend-native parity remains deferred until platform-native select widgets are in scope.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Native Select docs path first: Demo, Usage, Groups, Disabled, Invalid, Native Select vs Select, RTL, then keeps `Label Association` and `API Reference` as focused Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A styled native-select-like control following the upstream top-of-page example.")
                .no_shell()
                .test_id_prefix("ui-gallery-native-select-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `NativeSelect`.")
                .test_id_prefix("ui-gallery-native-select-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Groups", groups)
                .description("Organize options with `NativeSelectOptGroup`.")
                .test_id_prefix("ui-gallery-native-select-groups")
                .code_rust_from_file_region(snippets::with_groups::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disable the control with `disabled(true)`.")
                .test_id_prefix("ui-gallery-native-select-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Show validation state with `aria_invalid(true)`.")
                .test_id_prefix("ui-gallery-native-select-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("Native Select vs Select", native_select_vs_select)
                .description("Pick the simpler native-style surface only when you do not need the richer custom select recipe.")
                .no_shell()
                .test_id_prefix("ui-gallery-native-select-vs-select"),
            DocSection::new("RTL", rtl)
                .description("Direction provider + popup alignment under RTL.")
                .test_id_prefix("ui-gallery-native-select-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description(
                    "Use `FieldLabel::for_control` plus `NativeSelect::control_id` when you want an explicit label-click example outside the upstream docs path.",
                )
                .test_id_prefix("ui-gallery-native-select-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .description("Public surface summary, ownership notes, and defer rationale.")
                .no_shell()
                .test_id_prefix("ui-gallery-native-select-api-reference"),
        ],
    );

    vec![body.test_id("ui-gallery-native-select")]
}
