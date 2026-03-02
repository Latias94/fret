// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<Option<Arc<str>>>, Model<bool>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.value, state.open) {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let models = cx.app.models_mut();
            let value = models.insert(Some(Arc::<str>::from("engineering")));
            let open = models.insert(false);
            cx.with_state(Models::default, |st| {
                st.value = Some(value.clone());
                st.open = Some(open.clone());
            });
            (value, open)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (value, open) = ensure_models(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Department").into_element(cx),
        shadcn::Select::new(value, open)
            .placeholder("Choose department")
            .items([
                shadcn::SelectItem::new("engineering", "Engineering"),
                shadcn::SelectItem::new("design", "Design"),
                shadcn::SelectItem::new("marketing", "Marketing"),
                shadcn::SelectItem::new("operations", "Operations"),
            ])
            .into_element(cx),
        shadcn::FieldDescription::new("Select your department or area of work.").into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-select")
}
// endregion: example

