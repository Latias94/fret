use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_list = |cx: &mut ElementContext<'_, App>, id_prefix: &str| -> AnyElement {
        material3::List::new(value.clone())
            .a11y_label("Material 3 List")
            .test_id(format!("{id_prefix}-list"))
            .items(vec![
                material3::ListItem::new("alpha", "Alpha")
                    .leading_icon(ids::ui::SEARCH)
                    .a11y_label("List item alpha")
                    .test_id(format!("{id_prefix}-list-item-alpha")),
                material3::ListItem::new("beta", "Beta")
                    .leading_icon(ids::ui::SETTINGS)
                    .a11y_label("List item beta")
                    .test_id(format!("{id_prefix}-list-item-beta")),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true)
                    .a11y_label("List item disabled")
                    .test_id(format!("{id_prefix}-list-item-disabled")),
            ])
            .into_element(cx)
    };

    let standard = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Standard").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![build_list(cx, "ui-gallery-material3-standard")])
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let expressive = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Expressive").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            |cx| build_list(cx, "ui-gallery-material3-expressive"),
        )])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let variants = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        move |_cx| [standard, expressive],
    );

    vec![
        cx.text("Material 3 List: roving focus (Up/Down/Home/End) + selection follows focus."),
        cx.text("Compare Standard vs Expressive via subtree override (shape + icon size)."),
        variants,
        cx.text(format!("value={}", current.as_ref())),
    ]
}
