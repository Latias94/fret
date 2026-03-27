pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let website_url_id = "ui-gallery-input-input-group-website-url";
    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::Field::new([
        shadcn::FieldLabel::new("Website URL")
            .for_control(website_url_id)
            .into_element(cx),
        shadcn::InputGroup::new(value)
            .control_id(website_url_id)
            .placeholder("example.com")
            .leading([shadcn::InputGroupText::new("https://").into_element(cx)])
            .trailing([icon::icon(cx, icon_id("lucide.info"))])
            .refine_layout(max_w_xs)
            .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-input-group")
}
// endregion: example
