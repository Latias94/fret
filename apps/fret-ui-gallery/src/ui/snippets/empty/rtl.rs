pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model_keyed("query", String::new);
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let icon = icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));
        let input = shadcn::InputGroup::new(query)
            .a11y_label("RTL search")
            .leading([shadcn::InputGroupText::new("亘丨孬").into_element(cx)])
            .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
            .test_id("ui-gallery-empty-rtl-input-group")
            .into_element(cx);

        shadcn::empty(|cx| {
            ui::children![
                cx;
                shadcn::empty_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::empty_media(|cx| ui::children![cx; icon])
                            .variant(shadcn::EmptyMediaVariant::Icon),
                        shadcn::empty_title("RTL Empty State"),
                        shadcn::empty_description(
                            "This empty state uses RTL direction context for layout and alignment.",
                        ),
                    ]
                }),
                shadcn::empty_content(|cx| {
                    ui::children![cx; input, shadcn::Button::new("Create Project"),]
                }),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-empty-rtl")
}
// endregion: example
