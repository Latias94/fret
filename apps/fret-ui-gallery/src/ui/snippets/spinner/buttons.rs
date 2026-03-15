pub const SOURCE: &str = include_str!("buttons.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let button =
        |cx: &mut UiCx<'_>, label: &'static str, variant: Option<shadcn::ButtonVariant>| {
            let mut btn = shadcn::Button::new(label)
                .disabled(true)
                .size(shadcn::ButtonSize::Sm);
            if let Some(variant) = variant {
                btn = btn.variant(variant);
            }
            btn.leading_children([shadcn::Spinner::new().into_element(cx)])
                .into_element(cx)
        };

    ui::v_flex(|cx| {
        vec![
            button(cx, "Loading...", None),
            button(cx, "Please wait", Some(shadcn::ButtonVariant::Outline)),
            button(cx, "Processing", Some(shadcn::ButtonVariant::Secondary)),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-spinner-buttons")
}

// endregion: example
