pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (value, open) = cx.with_state(Models::default, |st| (st.value.clone(), st.open.clone()));

    let value = match value {
        Some(model) => model,
        None => {
            let model: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    let open = match open {
        Some(model) => model,
        None => {
            let model: Model<bool> = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    shadcn::Select::new(value, open)
        .trigger_test_id("ui-gallery-select-shadcn-demo-trigger")
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
                    shadcn::SelectItem::new("grapes", "Grapes").into(),
                    shadcn::SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into()])
            },
        )
}
// endregion: example
