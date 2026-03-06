pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (value, open) = cx.with_state(Models::default, |st| (st.value.clone(), st.open.clone()));
    let (value, open) = match (value, open) {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let value = cx.app.models_mut().insert(None::<Arc<str>>);
            let open = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.value = Some(value.clone());
                st.open = Some(open.clone());
            });
            (value, open)
        }
    };

    let control_id = "ui-gallery-select-label-control";
    let label = shadcn::FieldLabel::new("Fruit")
        .for_control(control_id)
        .test_id("ui-gallery-select-label-label")
        .into_element(cx);

    let select =
        shadcn::Select::new_controllable(cx, Some(value), None::<Arc<str>>, Some(open), false)
            .control_id(control_id)
            .trigger_test_id("ui-gallery-select-label-trigger")
            .into_element_parts(
                cx,
                |_cx| {
                    shadcn::SelectTrigger::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                },
                |_cx| shadcn::SelectValue::new().placeholder("Select a fruit"),
                |_cx| {
                    shadcn::SelectContent::new().with_entries([shadcn::SelectGroup::new([
                        shadcn::SelectLabel::new("Fruits").into(),
                        shadcn::SelectItem::new("apple", "Apple").into(),
                        shadcn::SelectItem::new("banana", "Banana").into(),
                        shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                    ])
                    .into()])
                },
            )
            .test_id("ui-gallery-select-label-control");

    shadcn::Field::new(vec![label, select])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(240.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-label")
}
// endregion: example
