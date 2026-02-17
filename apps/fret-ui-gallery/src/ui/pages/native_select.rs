use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct NativeSelectPageModels {
        styled_select_value: Option<Model<Option<Arc<str>>>>,
        styled_select_open: Option<Model<bool>>,
    }

    let (styled_select_value, styled_select_open) =
        cx.with_state(NativeSelectPageModels::default, |st| {
            (
                st.styled_select_value.clone(),
                st.styled_select_open.clone(),
            )
        });

    let (styled_select_value, styled_select_open) = match (styled_select_value, styled_select_open)
    {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let value = cx.app.models_mut().insert(Some(Arc::<str>::from("apple")));
            let open = cx.app.models_mut().insert(false);
            cx.with_state(NativeSelectPageModels::default, |st| {
                st.styled_select_value = Some(value.clone());
                st.styled_select_open = Some(open.clone());
            });
            (value, open)
        }
    };

    let select_width = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let content = shadcn::NativeSelect::new("Select a fruit")
            .a11y_label("Fruit")
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-demo");
        content
    };

    let groups = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(select_width.clone()),
            |cx| {
                vec![
                    shadcn::NativeSelect::new("Fruits").a11y_label("Fruits group").into_element(cx),
                    shadcn::NativeSelect::new("Vegetables")
                        .a11y_label("Vegetables group")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "NativeSelect currently exposes a single-label API; optgroup-like grouping is approximated here with multiple selects.",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-native-select-groups");
        content
    };

    let disabled = {
        let content = shadcn::NativeSelect::new("Disabled")
            .a11y_label("Disabled select")
            .disabled(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-disabled");
        content
    };

    let invalid = {
        let content = shadcn::NativeSelect::new("Select a country")
            .a11y_label("Invalid select")
            .aria_invalid(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-invalid");
        content
    };

    let native_vs_select = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                vec![
                    shadcn::NativeSelect::new("Native select")
                        .a11y_label("Native select")
                        .into_element(cx),
                    shadcn::Select::new(styled_select_value.clone(), styled_select_open.clone())
                        .placeholder("Styled select")
                        .items([
                            shadcn::SelectItem::new("apple", "Apple"),
                            shadcn::SelectItem::new("banana", "Banana"),
                            shadcn::SelectItem::new("blueberry", "Blueberry"),
                        ])
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "Use NativeSelect for native browser behavior/mobile ergonomics; use Select for richer overlays and custom interactions.",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-native-select-vs-select");
        content
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::NativeSelect::new("Select language")
                    .a11y_label("RTL native select")
                    .refine_layout(select_width.clone())
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-native-select-rtl");

        rtl_content
    };

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/native_select.rs` and `ecosystem/fret-ui-shadcn/src/select.rs`.",
            "Current NativeSelect API is label-based; explicit option/optgroup nodes are not exposed yet.",
            "Groups example is a practical approximation until optgroup-level API is added.",
            "Use NativeSelect for native browser behavior/mobile ergonomics; use Select for richer overlays and custom interactions.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Native Select docs order: Demo, Groups, Disabled, Invalid, Native Select vs Select, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic native select with a label.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new("Select a fruit")
    .a11y_label("Fruit")
    .into_element(cx);"#,
                ),
            DocSection::new("Groups", groups)
                .description("Optgroup-like grouping is approximated with multiple selects.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"stack::vstack(
    cx,
    stack::VStackProps::default().gap(Space::N2).items_start(),
    |cx| {
        vec![
            shadcn::NativeSelect::new("Fruits").a11y_label("Fruits group").into_element(cx),
            shadcn::NativeSelect::new("Vegetables").a11y_label("Vegetables group").into_element(cx),
        ]
    },
)
.into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled)
                .description("Disabled native select.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new("Disabled")
    .a11y_label("Disabled select")
    .disabled(true)
    .into_element(cx);"#,
                ),
            DocSection::new("Invalid", invalid)
                .description("Invalid state via `aria_invalid(true)`.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"shadcn::NativeSelect::new("Select a country")
    .a11y_label("Invalid select")
    .aria_invalid(true)
    .into_element(cx);"#,
                ),
            DocSection::new("Native Select vs Select", native_vs_select)
                .description("Compare native and styled select side-by-side.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"stack::vstack(cx, stack::VStackProps::default().gap(Space::N3).items_start(), |cx| {
    vec![
        shadcn::NativeSelect::new("Native select").a11y_label("Native select").into_element(cx),
        shadcn::Select::new(value, open)
            .placeholder("Styled select")
            .items([
                shadcn::SelectItem::new("apple", "Apple"),
                shadcn::SelectItem::new("banana", "Banana"),
            ])
            .into_element(cx),
    ]
})
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Native select under an RTL direction provider.")
                .max_w(Px(820.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::NativeSelect::new("Select language").into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and caveats.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-native-select")]
}
