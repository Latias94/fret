use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

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

    // shadcn NativeSelect is `w-fit` at the wrapper level; keep the gallery close to that default
    // (content-driven width), while still clamping to a reasonable max.
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let demo = {
        let select = shadcn::NativeSelect::new(demo_value, demo_open)
            .a11y_label("Native select: status")
            .placeholder("Select status")
            .trigger_test_id("ui-gallery-native-select-basic-native-trigger")
            .test_id_prefix("ui-gallery-native-select-basic-native")
            .options([
                shadcn::NativeSelectOption::placeholder("Select status"),
                shadcn::NativeSelectOption::new("todo", "Todo"),
                shadcn::NativeSelectOption::new("in-progress", "In Progress"),
                shadcn::NativeSelectOption::new("done", "Done"),
                shadcn::NativeSelectOption::new("cancelled", "Cancelled"),
            ])
            .refine_layout(select_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-basic-native");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![select],
        )
        .test_id("ui-gallery-native-select-demo")
    };

    let with_groups = {
        let select = shadcn::NativeSelect::new(groups_value, groups_open)
            .a11y_label("Native select: department")
            .placeholder("Select department")
            .trigger_test_id("ui-gallery-native-select-groups-native-trigger")
            .test_id_prefix("ui-gallery-native-select-groups-native")
            .options([shadcn::NativeSelectOption::placeholder("Select department")])
            .optgroups([
                shadcn::NativeSelectOptGroup::new(
                    "Engineering",
                    [
                        shadcn::NativeSelectOption::new("frontend", "Frontend"),
                        shadcn::NativeSelectOption::new("backend", "Backend"),
                        shadcn::NativeSelectOption::new("devops", "DevOps"),
                    ],
                ),
                shadcn::NativeSelectOptGroup::new(
                    "Sales",
                    [
                        shadcn::NativeSelectOption::new("sales-rep", "Sales Rep"),
                        shadcn::NativeSelectOption::new("account-manager", "Account Manager"),
                        shadcn::NativeSelectOption::new("sales-director", "Sales Director"),
                    ],
                ),
                shadcn::NativeSelectOptGroup::new(
                    "Operations",
                    [
                        shadcn::NativeSelectOption::new("support", "Customer Support"),
                        shadcn::NativeSelectOption::new("product-manager", "Product Manager"),
                        shadcn::NativeSelectOption::new("ops-manager", "Operations Manager"),
                    ],
                ),
            ])
            .refine_layout(select_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-groups-native");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![select],
        )
        .test_id("ui-gallery-native-select-groups")
    };

    let disabled_state = {
        let select = shadcn::NativeSelect::new(disabled_value, disabled_open)
            .a11y_label("Native select: priority (disabled)")
            .placeholder("Select priority")
            .disabled(true)
            .trigger_test_id("ui-gallery-native-select-disabled-native-trigger")
            .test_id_prefix("ui-gallery-native-select-disabled-native")
            .options([
                shadcn::NativeSelectOption::placeholder("Select priority"),
                shadcn::NativeSelectOption::new("low", "Low"),
                shadcn::NativeSelectOption::new("medium", "Medium"),
                shadcn::NativeSelectOption::new("high", "High"),
                shadcn::NativeSelectOption::new("critical", "Critical"),
            ])
            .refine_layout(select_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-disabled-native");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![select],
        )
        .test_id("ui-gallery-native-select-disabled")
    };

    let invalid_state = {
        let select = shadcn::NativeSelect::new(invalid_value, invalid_open)
            .a11y_label("Native select: role (invalid)")
            .placeholder("Select role")
            .aria_invalid(true)
            .trigger_test_id("ui-gallery-native-select-error-native-trigger")
            .test_id_prefix("ui-gallery-native-select-error-native")
            .options([
                shadcn::NativeSelectOption::placeholder("Select role"),
                shadcn::NativeSelectOption::new("admin", "Admin"),
                shadcn::NativeSelectOption::new("editor", "Editor"),
                shadcn::NativeSelectOption::new("viewer", "Viewer"),
                shadcn::NativeSelectOption::new("guest", "Guest"),
            ])
            .refine_layout(select_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-error-native");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![select],
        )
        .test_id("ui-gallery-native-select-error")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
            .placeholder("Choose language")
            .a11y_label("RTL native select")
            .refine_layout(select_layout.clone())
            .into_element(cx)
    })
    .test_id("ui-gallery-native-select-rtl");

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
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new(value, open)
    .a11y_label("Native select: status")
    .placeholder("Select status")
    .options([
        shadcn::NativeSelectOption::placeholder("Select status"),
        shadcn::NativeSelectOption::new("todo", "Todo"),
        shadcn::NativeSelectOption::new("in-progress", "In Progress"),
        shadcn::NativeSelectOption::new("done", "Done"),
        shadcn::NativeSelectOption::new("cancelled", "Cancelled"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("With Groups", with_groups)
                .description("Organize options using `NativeSelectOptGroup`.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new(value, open)
    .a11y_label("Native select: department")
    .placeholder("Select department")
    .options([shadcn::NativeSelectOption::placeholder("Select department")])
    .optgroups([
        shadcn::NativeSelectOptGroup::new(
            "Engineering",
            [
                shadcn::NativeSelectOption::new("frontend", "Frontend"),
                shadcn::NativeSelectOption::new("backend", "Backend"),
            ],
        ),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Disabled State", disabled_state)
                .description("Disable the select with `disabled(true)`.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new(value, open)
    .placeholder("Select priority")
    .disabled(true)
    .options([
        shadcn::NativeSelectOption::placeholder("Select priority"),
        shadcn::NativeSelectOption::new("low", "Low"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Invalid State", invalid_state)
                .description("Show validation errors with `aria_invalid(true)`.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new(value, open)
    .placeholder("Select role")
    .aria_invalid(true)
    .options([
        shadcn::NativeSelectOption::placeholder("Select role"),
        shadcn::NativeSelectOption::new("admin", "Admin"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Extras", rtl)
                .description("RTL smoke check (not present in upstream docs).")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
        .placeholder("Choose language")
        .into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and caveats.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-native-select")]
}
