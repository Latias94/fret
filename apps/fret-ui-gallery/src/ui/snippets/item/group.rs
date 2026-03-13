pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let people = [
        ("shadcn", "shadcn@vercel.com", "S"),
        ("maxleiter", "maxleiter@vercel.com", "M"),
        ("evilrabbit", "evilrabbit@vercel.com", "E"),
    ];

    let group = shadcn::item_group(cx, |cx| {
        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, (username, email, initials)) in people.iter().copied().enumerate() {
            let avatar =
                shadcn::Avatar::new([shadcn::AvatarFallback::new(initials).into_element(cx)])
                    .into_element(cx);
            let media = shadcn::ItemMedia::new([avatar]).into_element(cx);
            let content = shadcn::ItemContent::new([
                shadcn::ItemTitle::new(username).into_element(cx),
                shadcn::ItemDescription::new(email).into_element(cx),
            ])
            .into_element(cx);

            let add = shadcn::Button::new("")
                .a11y_label("Add")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::IconSm)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .icon(fret_icons::IconId::new_static("lucide.plus"))
                .into_element(cx)
                .test_id(format!("ui-gallery-item-group-add-{idx}"));
            let actions = shadcn::ItemActions::new([add]).into_element(cx);

            children.push(
                shadcn::Item::new([media, content, actions])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id(format!("ui-gallery-item-group-item-{idx}")),
            );
            if idx + 1 < people.len() {
                children.push(shadcn::ItemSeparator::new().into_element(cx));
            }
        }
        children
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-item-group");

    ui::v_stack(|_cx| vec![group])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-item-group-wrapper")
}
// endregion: example
