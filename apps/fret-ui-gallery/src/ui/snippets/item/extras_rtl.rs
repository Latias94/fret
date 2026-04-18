pub const SOURCE: &str = include_str!("extras_rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

fn outline_button_sm(
    cx: &mut AppComponentCx<'_>,
    label: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

fn item_basic(
    cx: &mut AppComponentCx<'_>,
    title: &'static str,
    description: &'static str,
    actions: Vec<AnyElement>,
    test_id: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let content_children = [
        shadcn::ItemTitle::new(title).into_element(cx),
        shadcn::ItemDescription::new(description).into_element(cx),
    ];

    let mut children = vec![shadcn::ItemContent::new(content_children).into_element(cx)];
    if !actions.is_empty() {
        children.push(shadcn::ItemActions::new(actions).into_element(cx));
    }

    shadcn::Item::new(children)
        .variant(shadcn::ItemVariant::Outline)
        .action(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let rtl = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let action = outline_button_sm(cx, "فتح").into_element(cx);
        item_basic(
            cx,
            "لوحة التحكم",
            "نظرة عامة على حسابك ونشاطك.",
            vec![action],
            "ui-gallery-item-rtl",
        )
        .into_element(cx)
    });

    rtl.test_id("ui-gallery-item-rtl-wrapper")
}
// endregion: example
