// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<String>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);
    let theme = Theme::global(&*cx.app);
    let muted_fg = theme.color_token("muted-foreground");

    let search_icon = shadcn::icon::icon_with(
        cx,
        fret_icons::IconId::new_static("lucide.search"),
        Some(Px(16.0)),
        Some(ColorRef::Color(muted_fg)),
    );

    shadcn::InputGroup::new(value)
        .a11y_label("Search")
        .leading([search_icon])
        .trailing([
            shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
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

