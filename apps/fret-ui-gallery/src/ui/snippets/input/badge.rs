pub const SOURCE: &str = include_str!("badge.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let label = ui::h_row(|cx| {
        vec![
            shadcn::FieldLabel::new("Webhook URL").into_element(cx),
            shadcn::Badge::new("Beta")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .justify_between()
    .items_center()
    .w_full()
    .into_element(cx);

    shadcn::Field::new([
        label,
        shadcn::Input::new(value)
            .a11y_label("Webhook URL")
            .placeholder("https://api.example.com/webhook")
            .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-badge")
}
// endregion: example
