use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct NativeSelectPageModels {
        basic_value: Option<Model<Option<Arc<str>>>>,
        basic_open: Option<Model<bool>>,
        groups_value: Option<Model<Option<Arc<str>>>>,
        groups_open: Option<Model<bool>>,
        disabled_value: Option<Model<Option<Arc<str>>>>,
        disabled_open: Option<Model<bool>>,
        error_value: Option<Model<Option<Arc<str>>>>,
        error_open: Option<Model<bool>>,
    }

    let (
        basic_value,
        basic_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        error_value,
        error_open,
    ) = cx.with_state(NativeSelectPageModels::default, |st| {
        (
            st.basic_value.clone(),
            st.basic_open.clone(),
            st.groups_value.clone(),
            st.groups_open.clone(),
            st.disabled_value.clone(),
            st.disabled_open.clone(),
            st.error_value.clone(),
            st.error_open.clone(),
        )
    });

    let (
        basic_value,
        basic_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        error_value,
        error_open,
    ) = match (
        basic_value,
        basic_open,
        groups_value,
        groups_open,
        disabled_value,
        disabled_open,
        error_value,
        error_open,
    ) {
        (
            Some(basic_value),
            Some(basic_open),
            Some(groups_value),
            Some(groups_open),
            Some(disabled_value),
            Some(disabled_open),
            Some(error_value),
            Some(error_open),
        ) => (
            basic_value,
            basic_open,
            groups_value,
            groups_open,
            disabled_value,
            disabled_open,
            error_value,
            error_open,
        ),
        _ => {
            let models = cx.app.models_mut();
            let basic_value = models.insert(None);
            let basic_open = models.insert(false);
            let groups_value = models.insert(None);
            let groups_open = models.insert(false);
            let disabled_value = models.insert(None);
            let disabled_open = models.insert(false);
            let error_value = models.insert(None);
            let error_open = models.insert(false);
            cx.with_state(NativeSelectPageModels::default, |st| {
                st.basic_value = Some(basic_value.clone());
                st.basic_open = Some(basic_open.clone());
                st.groups_value = Some(groups_value.clone());
                st.groups_open = Some(groups_open.clone());
                st.disabled_value = Some(disabled_value.clone());
                st.disabled_open = Some(disabled_open.clone());
                st.error_value = Some(error_value.clone());
                st.error_open = Some(error_open.clone());
            });
            (
                basic_value,
                basic_open,
                groups_value,
                groups_open,
                disabled_value,
                disabled_open,
                error_value,
                error_open,
            )
        }
    };

    let select_width = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let heading = |cx: &mut ElementContext<'_, App>, text: &'static str| {
        ui::text(cx, text)
            .text_sm()
            .font_medium()
            .text_color(ColorRef::Color(muted_fg))
            .into_element(cx)
    };

    let block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, children: Vec<AnyElement>| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                |cx| {
                    vec![
                        heading(cx, title),
                        stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N4)
                                .items_start()
                                .layout(LayoutRefinement::default().w_full().min_w_0()),
                            move |_cx| children,
                        ),
                    ]
                },
            )
        };

    let basic = {
        let native = shadcn::NativeSelect::new("Select a fruit")
            .a11y_label("Native select: fruit")
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-basic-native");
        let styled = shadcn::Select::new(basic_value, basic_open)
            .placeholder("Select a fruit")
            .trigger_test_id("ui-gallery-native-select-basic-styled-trigger")
            .items([
                shadcn::SelectItem::new("apple", "Apple"),
                shadcn::SelectItem::new("banana", "Banana"),
                shadcn::SelectItem::new("blueberry", "Blueberry"),
                shadcn::SelectItem::new("grapes", "Grapes").disabled(true),
                shadcn::SelectItem::new("pineapple", "Pineapple"),
            ])
            .refine_layout(select_width.clone())
            .into_element(cx);
        block(cx, "Basic Select", vec![native, styled]).test_id("ui-gallery-native-select-basic")
    };

    let with_groups = {
        let native = shadcn::NativeSelect::new("Select a food")
            .a11y_label("Native select: food")
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-groups-native");

        let styled = shadcn::Select::new(groups_value, groups_open)
            .placeholder("Select a food")
            .trigger_test_id("ui-gallery-native-select-groups-styled-trigger")
            .entries([
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Fruits").into(),
                    shadcn::SelectItem::new("apple", "Apple").into(),
                    shadcn::SelectItem::new("banana", "Banana").into(),
                    shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                ])
                .into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Vegetables").into(),
                    shadcn::SelectItem::new("carrot", "Carrot").into(),
                    shadcn::SelectItem::new("broccoli", "Broccoli").into(),
                    shadcn::SelectItem::new("spinach", "Spinach").into(),
                ])
                .into(),
            ])
            .refine_layout(select_width.clone())
            .into_element(cx);

        block(cx, "With Groups", vec![native, styled]).test_id("ui-gallery-native-select-groups")
    };

    let disabled_state = {
        let native = shadcn::NativeSelect::new("Disabled")
            .a11y_label("Native select: disabled")
            .disabled(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-disabled-native");
        let styled = shadcn::Select::new(disabled_value, disabled_open)
            .placeholder("Disabled")
            .trigger_test_id("ui-gallery-native-select-disabled-styled-trigger")
            .disabled(true)
            .items([
                shadcn::SelectItem::new("apple", "Apple"),
                shadcn::SelectItem::new("banana", "Banana"),
            ])
            .refine_layout(select_width.clone())
            .into_element(cx);
        block(cx, "Disabled State", vec![native, styled])
            .test_id("ui-gallery-native-select-disabled")
    };

    let error_state = {
        let native = shadcn::NativeSelect::new("Error state")
            .a11y_label("Native select: error")
            .aria_invalid(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-error-native");
        let styled = shadcn::Select::new(error_value, error_open)
            .placeholder("Error state")
            .trigger_test_id("ui-gallery-native-select-error-styled-trigger")
            .aria_invalid(true)
            .items([
                shadcn::SelectItem::new("apple", "Apple"),
                shadcn::SelectItem::new("banana", "Banana"),
            ])
            .refine_layout(select_width.clone())
            .into_element(cx);
        block(cx, "Error State", vec![native, styled]).test_id("ui-gallery-native-select-error")
    };

    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N8)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![basic, with_groups, disabled_state, error_state],
    )
    .test_id("ui-gallery-native-select-demo");

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::NativeSelect::new("Select language")
            .a11y_label("RTL native select")
            .refine_layout(select_width.clone())
            .into_element(cx)
    })
    .test_id("ui-gallery-native-select-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/native_select.rs` and `ecosystem/fret-ui-shadcn/src/select.rs`.",
            "Gallery alignment note: upstream NativeSelect is a real DOM `<select>` with Option/OptGroup nodes; Fret's `NativeSelect` is currently a styled pressable placeholder (no option model yet).",
            "Use `Select` for rich overlays and custom interactions; revisit `NativeSelect` when platform-native select widgets are implemented per backend.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Native Select demo: Basic Select, With Groups, Disabled State, Error State.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .no_shell()
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"// Basic Select
shadcn::NativeSelect::new("Select a fruit")
    .a11y_label("Native select: fruit")
    .into_element(cx);

shadcn::Select::new(value, open)
    .placeholder("Select a fruit")
    .items([
        shadcn::SelectItem::new("apple", "Apple"),
        shadcn::SelectItem::new("banana", "Banana"),
        shadcn::SelectItem::new("blueberry", "Blueberry"),
        shadcn::SelectItem::new("grapes", "Grapes").disabled(true),
        shadcn::SelectItem::new("pineapple", "Pineapple"),
    ])
    .into_element(cx);

// With Groups (styled Select supports groups)
shadcn::Select::new(value, open)
    .placeholder("Select a food")
    .entries([
        shadcn::SelectGroup::new([
            shadcn::SelectLabel::new("Fruits").into(),
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
        ])
        .into(),
    ])
    .into_element(cx);

// Disabled / Error use `disabled(true)` + `aria_invalid(true)`."#,
                ),
            DocSection::new("Extras", rtl)
                .description("RTL smoke check (not present in upstream demo).")
                .max_w(Px(980.0))
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
