use super::super::super::super::*;

pub(in crate::ui) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| children,
        )
    };

    let badge_icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static(name),
            Some(Px(12.0)),
            Some(fg),
        )
    };

    let variants = {
        let children = vec![
            shadcn::Badge::new("Default").into_element(cx),
            shadcn::Badge::new("Secondary")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Badge::new("Destructive")
                .variant(shadcn::BadgeVariant::Destructive)
                .into_element(cx),
            shadcn::Badge::new("Outline")
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Variants", body)
    };

    let with_icon = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Verified")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                .into_element(cx),
            shadcn::Badge::new("Bookmark")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.bookmark", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Icon", body)
    };

    let with_spinner = {
        let destructive_fg = ColorRef::Color(theme.color_required("destructive-foreground"));
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let children = vec![
            shadcn::Badge::new("Deleting")
                .variant(shadcn::BadgeVariant::Destructive)
                .children([shadcn::Spinner::new()
                    .color(destructive_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
            shadcn::Badge::new("Generating")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([shadcn::Spinner::new()
                    .color(secondary_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Spinner", body)
    };

    let link = {
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Open Link")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Link", body)
    };

    let custom_colors = {
        let border_transparent =
            ChromeRefinement::default().border_color(ColorRef::Color(CoreColor::TRANSPARENT));

        let children = vec![
            shadcn::Badge::new("Blue")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.95,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Green")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.91,
                            g: 0.98,
                            b: 0.91,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Sky")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.97,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Purple")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.95,
                            g: 0.92,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Red")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 1.00,
                            g: 0.92,
                            b: 0.92,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Custom Colors", body)
    };

    let rtl = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let children = vec![
                    shadcn::Badge::new("شارة").into_element(cx),
                    shadcn::Badge::new("ثانوي")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    shadcn::Badge::new("متحقق")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                        .into_element(cx),
                ];
                row(cx, children)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![variants, with_icon, with_spinner, link, custom_colors, rtl],
    )]
}
