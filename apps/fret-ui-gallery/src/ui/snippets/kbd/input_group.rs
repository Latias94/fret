pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", String::new);
    let theme = Theme::global(&*cx.app);
    let muted_fg = theme.color_token("muted-foreground");

    let search_icon = icon::icon_with(
        cx,
        fret_icons::IconId::new_static("lucide.search"),
        Some(Px(16.0)),
        Some(ColorRef::Color(muted_fg)),
    );

    shadcn::InputGroup::new(value)
        .a11y_label("Search")
        .leading([search_icon])
        .trailing([
            shadcn::Kbd::from_children([shadcn::raw::kbd::kbd_icon(
                cx,
                fret_icons::IconId::new_static("lucide.command"),
            )])
            .into_element(cx),
            shadcn::Kbd::new("K").into_element(cx),
        ])
        .trailing_has_kbd(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
        .into_element(cx)
        .test_id("ui-gallery-kbd-input-group")
}
// endregion: example
