use super::super::*;

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ToggleGroupModels {
        value: Option<Model<Option<Arc<str>>>>,
    }

    let value = cx.with_state(ToggleGroupModels::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(ToggleGroupModels::default, |st| {
                st.value = Some(model.clone())
            });
            model
        }
    };

    let group = shadcn::ToggleGroup::single(value.clone())
        .item(shadcn::ToggleGroupItem::new(
            "bold",
            [ui::label(cx, "B").into_element(cx)],
        ))
        .item(shadcn::ToggleGroupItem::new(
            "italic",
            [ui::label(cx, "I").into_element(cx)],
        ))
        .item(shadcn::ToggleGroupItem::new(
            "underline",
            [ui::label(cx, "U").into_element(cx)],
        ))
        .into_element(cx);

    let selected = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![group, cx.text(format!("selected={}", selected.as_ref()))]
}
