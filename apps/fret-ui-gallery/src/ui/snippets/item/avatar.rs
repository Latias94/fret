pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";

fn icon_button(
    cx: &mut ElementContext<'_, App>,
    icon_id: &'static str,
    variant: shadcn::ButtonVariant,
    test_id: &'static str,
) -> AnyElement {
    shadcn::Button::new("")
        .a11y_label(icon_id)
        .variant(variant)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static(icon_id))
        .into_element(cx)
        .test_id(test_id)
}

fn item_team(
    cx: &mut ElementContext<'_, App>,
    test_id: &'static str,
    action_test_id: &'static str,
) -> AnyElement {
    let avatars = ui::h_row(|cx| {
        vec![
            shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
                .into_element(cx),
            shadcn::Avatar::new([shadcn::AvatarFallback::new("LR").into_element(cx)])
                .into_element(cx),
            shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)])
                .into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx);
    let media = shadcn::ItemMedia::new([avatars]).into_element(cx);
    let content = shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Design Department").into_element(cx),
        shadcn::ItemDescription::new("Meet our team of designers, engineers, and researchers.")
            .into_element(cx),
    ])
    .into_element(cx);

    let chevron = icon_button(
        cx,
        "lucide.chevron-right",
        shadcn::ButtonVariant::Outline,
        action_test_id,
    );
    let actions = shadcn::ItemActions::new([chevron])
        .refine_layout(LayoutRefinement::default().mt(Space::N1))
        .into_element(cx);

    shadcn::Item::new([media, content, actions])
        .on_click(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(520.0)));

    let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new("ER").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(40.0)).h_px(Px(40.0)))
        .into_element(cx);
    let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
    let content = shadcn::ItemContent::new([
        shadcn::ItemTitle::new("Evil Rabbit").into_element(cx),
        shadcn::ItemDescription::new("Last seen 5 months ago").into_element(cx),
    ])
    .into_element(cx);

    let invite = shadcn::Button::new("")
        .a11y_label("Invite")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .icon(fret_icons::IconId::new_static("lucide.plus"))
        .into_element(cx)
        .test_id("ui-gallery-item-avatar-invite");
    let actions = shadcn::ItemActions::new([invite]).into_element(cx);

    let item_one = shadcn::Item::new([media, content, actions])
        .variant(shadcn::ItemVariant::Outline)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-avatar-one");

    let team = item_team(
        cx,
        "ui-gallery-item-avatar-team",
        "ui-gallery-item-avatar-team-action",
    );

    ui::v_stack(|_cx| vec![item_one, team])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_lg)
        .into_element(cx)
        .test_id("ui-gallery-item-avatar")
}
// endregion: example
