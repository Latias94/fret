pub const SOURCE: &str = include_str!("select.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("engineering")));
    let open = cx.local_model_keyed("open", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::build(|cx, out| {
        out.push_ui(cx, shadcn::FieldLabel::new("Department"));
        out.push_ui(
            cx,
            shadcn::Select::new(value, open)
                .value(shadcn::SelectValue::new().placeholder("Choose department"))
                .items([
                    shadcn::SelectItem::new("engineering", "Engineering"),
                    shadcn::SelectItem::new("design", "Design"),
                    shadcn::SelectItem::new("marketing", "Marketing"),
                    shadcn::SelectItem::new("sales", "Sales"),
                    shadcn::SelectItem::new("support", "Customer Support"),
                    shadcn::SelectItem::new("hr", "Human Resources"),
                    shadcn::SelectItem::new("finance", "Finance"),
                    shadcn::SelectItem::new("operations", "Operations"),
                ]),
        );
        out.push_ui(
            cx,
            shadcn::FieldDescription::new("Select your department or area of work."),
        );
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-select")
}
// endregion: example
