pub const SOURCE: &str = include_str!("field_association.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("dark")));
    let open = cx.local_model_keyed("open", || false);

    shadcn::Field::build(|cx, out| {
        out.push_ui(cx, shadcn::FieldLabel::new("Theme"));
        out.push_ui(
            cx,
            shadcn::Select::new(value, open)
                .test_id_prefix("ui-gallery-select-field-association")
                .value(shadcn::SelectValue::new().placeholder("Select theme"))
                .items([
                    shadcn::SelectItem::new("light", "Light"),
                    shadcn::SelectItem::new("dark", "Dark"),
                    shadcn::SelectItem::new("system", "System"),
                ]),
        );
        out.push_ui(
            cx,
            shadcn::FieldDescription::new("Inside `Field::build(...)`, Select can inherit label and description association automatically."),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-select-field-association")
}
// endregion: example
