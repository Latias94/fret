use super::super::super::super::*;

pub(in crate::ui) fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default)]
    struct SelectPageModels {
        align_item_with_trigger: Option<Model<bool>>,
    }

    let align_item_with_trigger = cx.with_state(SelectPageModels::default, |st| {
        st.align_item_with_trigger.clone()
    });
    let align_item_with_trigger = match align_item_with_trigger {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(SelectPageModels::default, |st| {
                st.align_item_with_trigger = Some(model.clone());
            });
            model
        }
    };

    let demo = {
        // Keep the primary demo select stable for existing diag scripts.
        let entries: Vec<shadcn::SelectEntry> = std::iter::once(
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Fruits").into(),
                shadcn::SelectItem::new("apple", "Apple")
                    .test_id("ui-gallery-select-item-apple")
                    .into(),
                shadcn::SelectItem::new("banana", "Banana")
                    .test_id("ui-gallery-select-item-banana")
                    .into(),
                shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                shadcn::SelectItem::new("grapes", "Grapes").into(),
                shadcn::SelectItem::new("pineapple", "Pineapple").into(),
            ])
            .into(),
        )
        .chain(std::iter::once(shadcn::SelectSeparator::default().into()))
        .chain(std::iter::once(
            shadcn::SelectGroup::new(
                std::iter::once(shadcn::SelectLabel::new("More").into()).chain((1..=40).map(|i| {
                    let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                    let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                    let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                    shadcn::SelectItem::new(value, label)
                        .test_id(test_id)
                        .disabled(i == 15)
                        .into()
                })),
            )
            .into(),
        ))
        .collect();

        let select = shadcn::Select::new(value.clone(), open)
            .trigger_test_id("ui-gallery-select-trigger")
            .placeholder("Select a fruit")
            .entries(entries)
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx);

        let selected_value = value.clone();
        let selected_label = cx.scope(move |cx| {
            let selected: Arc<str> = cx
                .get_model_cloned(&selected_value, fret_ui::Invalidation::Paint)
                .unwrap_or_default()
                .unwrap_or_else(|| Arc::<str>::from("<none>"));
            shadcn::typography::muted(cx, Arc::<str>::from(format!("Selected: {selected}")))
                .test_id("ui-gallery-select-selected-label")
        });

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![select, selected_label],
        )
        .test_id("ui-gallery-select-demo")
    };

    let align_item = {
        let align = cx
            .watch_model(&align_item_with_trigger)
            .cloned()
            .unwrap_or(true);
        let position = if align {
            shadcn::select::SelectPosition::ItemAligned
        } else {
            shadcn::select::SelectPosition::Popper
        };

        let select = shadcn::Select::new_controllable(cx, None, Some("banana"), None, false)
            .position(position)
            .entries([shadcn::SelectGroup::new([
                shadcn::SelectItem::new("apple", "Apple").into(),
                shadcn::SelectItem::new("banana", "Banana").into(),
                shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                shadcn::SelectItem::new("grapes", "Grapes").into(),
                shadcn::SelectItem::new("pineapple", "Pineapple").into(),
            ])
            .into()])
            .into_element(cx);

        let content = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Align Item")
                        .for_control("ui-gallery-select-align-item-switch")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Toggle to align the item with the trigger.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Switch::new(align_item_with_trigger.clone())
                    .control_id("ui-gallery-select-align-item-switch")
                    .a11y_label("Align item with trigger")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::Field::new([select])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-align-item");

        content
    };

    let groups = {
        let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .placeholder("Select a fruit")
            .entries([
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Fruits").into(),
                    shadcn::SelectItem::new("apple", "Apple").into(),
                    shadcn::SelectItem::new("banana", "Banana").into(),
                    shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                ])
                .into(),
                shadcn::SelectSeparator::default().into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Vegetables").into(),
                    shadcn::SelectItem::new("carrot", "Carrot").into(),
                    shadcn::SelectItem::new("broccoli", "Broccoli").into(),
                    shadcn::SelectItem::new("spinach", "Spinach").into(),
                ])
                .into(),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
            .into_element(cx)
            .test_id("ui-gallery-select-groups");
        select
    };

    let scrollable = {
        let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .placeholder("Select a timezone")
            .entries([
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("North America").into(),
                    shadcn::SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    shadcn::SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    shadcn::SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    shadcn::SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    shadcn::SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    shadcn::SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Europe & Africa").into(),
                    shadcn::SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    shadcn::SelectItem::new("cet", "Central European Time (CET)").into(),
                    shadcn::SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    shadcn::SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    shadcn::SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    shadcn::SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Asia").into(),
                    shadcn::SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    shadcn::SelectItem::new("ist", "India Standard Time (IST)").into(),
                    shadcn::SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    shadcn::SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    shadcn::SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    shadcn::SelectItem::new(
                        "ist_indonesia",
                        "Indonesia Central Standard Time (WITA)",
                    )
                    .into(),
                ])
                .into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Australia & Pacific").into(),
                    shadcn::SelectItem::new("awst", "Australian Western Standard Time (AWST)")
                        .into(),
                    shadcn::SelectItem::new("acst", "Australian Central Standard Time (ACST)")
                        .into(),
                    shadcn::SelectItem::new("aest", "Australian Eastern Standard Time (AEST)")
                        .into(),
                    shadcn::SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    shadcn::SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("South America").into(),
                    shadcn::SelectItem::new("art", "Argentina Time (ART)").into(),
                    shadcn::SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    shadcn::SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    shadcn::SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ])
            .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
            .into_element(cx)
            .test_id("ui-gallery-select-scrollable");
        select
    };

    let disabled = {
        let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .placeholder("Select a fruit")
            .disabled(true)
            .entries([shadcn::SelectGroup::new([
                shadcn::SelectItem::new("apple", "Apple").into(),
                shadcn::SelectItem::new("banana", "Banana").into(),
                shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                shadcn::SelectItem::new("grapes", "Grapes")
                    .disabled(true)
                    .into(),
                shadcn::SelectItem::new("pineapple", "Pineapple").into(),
            ])
            .into()])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
            .into_element(cx)
            .test_id("ui-gallery-select-disabled");
        select
    };

    let invalid = {
        shadcn::Field::new([
            shadcn::FieldLabel::new("Fruit").into_element(cx),
            shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
                .placeholder("Select a fruit")
                .aria_invalid(true)
                .entries([shadcn::SelectGroup::new([
                    shadcn::SelectItem::new("apple", "Apple").into(),
                    shadcn::SelectItem::new("banana", "Banana").into(),
                    shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                ])
                .into()])
                .into_element(cx),
            shadcn::FieldError::new("Please select a fruit.").into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-invalid")
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
                    .placeholder("اختر فاكهة")
                    .entries([
                        shadcn::SelectGroup::new([
                            shadcn::SelectLabel::new("الفواكه").into(),
                            shadcn::SelectItem::new("apple", "تفاح").into(),
                            shadcn::SelectItem::new("banana", "موز").into(),
                            shadcn::SelectItem::new("blueberry", "توت أزرق").into(),
                        ])
                        .into(),
                        shadcn::SelectSeparator::default().into(),
                        shadcn::SelectGroup::new([
                            shadcn::SelectLabel::new("الخضروات").into(),
                            shadcn::SelectItem::new("carrot", "جزر").into(),
                            shadcn::SelectItem::new("broccoli", "بروكلي").into(),
                            shadcn::SelectItem::new("spinach", "سبانخ").into(),
                        ])
                        .into(),
                    ])
                    .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-select-rtl");
        rtl_content
    };

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Select docs order: Demo, Align Item With Trigger, Groups, Scrollable, Disabled, Invalid, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-select-demo")
                .code(
                    "rust",
                    r#"let entries: Vec<shadcn::SelectEntry> = vec![
    shadcn::SelectGroup::new([
        shadcn::SelectLabel::new("Fruits").into(),
        shadcn::SelectItem::new("apple", "Apple").into(),
        shadcn::SelectItem::new("banana", "Banana").into(),
    ])
    .into(),
];

let select = shadcn::Select::new(value, open)
    .placeholder("Select a fruit")
    .entries(entries)
    .into_element(cx);"#,
                ),
            DocSection::new("Align Item With Trigger", align_item)
                .test_id_prefix("ui-gallery-select-align-item")
                .code(
                    "rust",
                    r#"use fret_ui_shadcn::select::SelectPosition;

let select = shadcn::Select::new_controllable(cx, None, Some("banana"), None, false)
    .position(SelectPosition::ItemAligned)
    .entries([...])
    .into_element(cx);"#,
                )
                .max_w(Px(540.0)),
            DocSection::new("Groups", groups)
                .test_id_prefix("ui-gallery-select-groups")
                .code(
                    "rust",
                    r#"let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
    .placeholder("Select a fruit")
    .entries([
        shadcn::SelectGroup::new([...]).into(),
        shadcn::SelectSeparator::default().into(),
        shadcn::SelectGroup::new([...]).into(),
    ])
    .into_element(cx);"#,
                )
                .max_w(Px(540.0)),
            DocSection::new("Scrollable", scrollable)
                .test_id_prefix("ui-gallery-select-scrollable")
                .code(
                    "rust",
                    r#"let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
    .placeholder("Select a timezone")
    .entries([...])
    .into_element(cx);"#,
                )
                .max_w(Px(620.0)),
            DocSection::new("Disabled", disabled)
                .test_id_prefix("ui-gallery-select-disabled")
                .code(
                    "rust",
                    r#"let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
    .placeholder("Select a fruit")
    .disabled(true)
    .entries([...])
    .into_element(cx);"#,
                )
                .max_w(Px(540.0)),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-select-invalid")
                .description("Invalid styling is typically shown with a Field + error message.")
                .code(
                    "rust",
                    r#"let field = shadcn::Field::new([
    shadcn::FieldLabel::new("Fruit").into_element(cx),
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .aria_invalid(true)
        .entries([...])
        .into_element(cx),
    shadcn::FieldError::new("Please select a fruit.").into_element(cx),
])
.into_element(cx);"#,
                )
                .max_w(Px(620.0)),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-select-rtl")
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .placeholder("اختر فاكهة")
        .entries([...])
        .into_element(cx)
});"#,
                )
                .max_w(Px(620.0)),
        ],
    );

    vec![body]
}
