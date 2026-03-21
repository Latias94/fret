pub const SOURCE: &str = include_str!("field.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::Field::build(|cx, out| {
        out.push_ui(cx, shadcn::FieldLabel::new("Feedback"));
        out.push_ui(
            cx,
            shadcn::Textarea::new(value)
                .placeholder("Your feedback helps us improve...")
                .rows(4),
        );
        out.push_ui(
            cx,
            shadcn::FieldDescription::new("Share your thoughts about our service."),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-field")
}
// endregion: example
