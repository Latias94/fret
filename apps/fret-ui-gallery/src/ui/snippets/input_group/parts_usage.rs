pub const SOURCE: &str = include_str!("parts_usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model_keyed("parts_usage", String::new);

    shadcn::InputGroup::new(query)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element_parts(cx, |cx| {
            vec![
                shadcn::InputGroupPart::input(
                    shadcn::InputGroupInput::new()
                        .a11y_label("Search")
                        .placeholder("Search...")
                        .test_id("ui-gallery-input-group-parts-usage-control"),
                ),
                shadcn::InputGroupPart::addon(
                    shadcn::InputGroupAddon::new([icon::icon(
                        cx,
                        IconId::new_static("lucide.search"),
                    )])
                    .align(shadcn::InputGroupAddonAlign::InlineStart),
                ),
                shadcn::InputGroupPart::addon(
                    shadcn::InputGroupAddon::new([
                        shadcn::InputGroupText::new("12 results").into_element(cx)
                    ])
                    .align(shadcn::InputGroupAddonAlign::InlineEnd),
                ),
            ]
        })
        .test_id("ui-gallery-input-group-parts-usage")
}
// endregion: example
