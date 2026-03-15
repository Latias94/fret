pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{FontWeight, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let url_value = cx.local_model(String::new);

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new_children([
            fret_ui_shadcn::icon::icon(cx, icon_id("lucide.globe")),
            ui::text("https://")
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .into_element(cx),
        ])
        .into(),
        shadcn::Input::new(url_value)
            .a11y_label("URL")
            .placeholder("my-app")
            .refine_layout(LayoutRefinement::default().w_px(Px(220.0)).min_w_0())
            .into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-text")
}

// endregion: example
