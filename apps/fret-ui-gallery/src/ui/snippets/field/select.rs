pub const SOURCE: &str = include_str!("select.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("engineering")));
    let open = cx.local_model_keyed("open", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Department").into_element(cx),
        shadcn::Select::new(value, open)
            .value(shadcn::SelectValue::new().placeholder("Choose department"))
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
