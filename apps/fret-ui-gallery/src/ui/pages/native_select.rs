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
    let with_groups = snippets::with_groups::render(cx, groups_value, groups_open);
    let disabled_state = snippets::disabled::render(cx, disabled_value, disabled_open);
    let invalid_state = snippets::invalid::render(cx, invalid_value, invalid_open);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/native_select.rs`.",
            "Gallery alignment note: upstream NativeSelect is a DOM `<select>`; Fret's `NativeSelect` is a popover-backed fallback today (platform-native pickers TBD).",
            "Use `Select` for rich overlays and custom interactions; revisit `NativeSelect` when platform-native select widgets are implemented per backend.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Native Select docs: Demo, With Groups, Disabled State, Invalid State.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A styled native-select-like control (upstream is a DOM `<select>`).")
                .no_shell()
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("With Groups", with_groups)
                .description("Organize options using `NativeSelectOptGroup`.")
                .code_rust_from_file_region(snippets::with_groups::SOURCE, "example"),
            DocSection::new("Disabled State", disabled_state)
                .description("Disable the select with `disabled(true)`.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid State", invalid_state)
                .description("Show validation errors with `aria_invalid(true)`.")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("Extras", rtl)
                .description("RTL smoke check (not present in upstream docs).")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-native-select")]
}
