pub const SOURCE: &str = include_str!("item.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let progress_value = cx.local_model_keyed("spinner_item_progress", || 0.75_f32);

    let item = shadcn::Item::new([
        shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx),
        shadcn::ItemContent::new([
            shadcn::ItemTitle::new("Downloading...").into_element(cx),
            shadcn::ItemDescription::new("129 MB / 1000 MB").into_element(cx),
        ])
        .into_element(cx),
        shadcn::ItemActions::new([shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)])
        .into_element(cx),
        shadcn::ItemFooter::new([shadcn::Progress::new(progress_value).into_element(cx)])
            .into_element(cx),
    ])
    .variant(shadcn::ItemVariant::Outline)
    .into_element(cx);

    ui::v_flex(|_cx| vec![item])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().max_w(Px(448.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-item")
}
// endregion: example
