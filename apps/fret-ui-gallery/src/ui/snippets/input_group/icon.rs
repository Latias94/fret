pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let search = cx.local_model_keyed("search", String::new);
    let email = cx.local_model_keyed("email", String::new);
    let card = cx.local_model_keyed("card", String::new);
    let favorite = cx.local_model_keyed("favorite", String::new);

    let icon = |cx: &mut UiCx<'_>, id: &'static str| icon::icon(cx, IconId::new_static(id));
    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let search_group = shadcn::InputGroup::new(search)
        .a11y_label("Search")
        .placeholder("Search...")
        .leading([icon(cx, "lucide.search")])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let email_group = shadcn::InputGroup::new(email)
        .a11y_label("Email")
        .placeholder("Enter your email")
        .leading([icon(cx, "lucide.mail")])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let card_group = shadcn::InputGroup::new(card)
        .a11y_label("Card number")
        .placeholder("Card number")
        .leading([icon(cx, "lucide.credit-card")])
        .trailing([icon(cx, "lucide.check")])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    let favorite_group = shadcn::InputGroup::new(favorite)
        .a11y_label("Card number")
        .placeholder("Card number")
        .trailing([icon(cx, "lucide.star"), icon(cx, "lucide.info")])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_stack(move |_cx| vec![search_group, email_group, card_group, favorite_group])
        .gap(Space::N4)
        .items_start()
        .layout(max_w)
        .into_element(cx)
        .test_id("ui-gallery-input-group-icon")
}
// endregion: example
