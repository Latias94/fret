use super::super::*;

pub(super) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct KanbanModels {
        items: Option<Model<Vec<shadcn::extras::KanbanItem>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(860.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let announcement = {
        shadcn::extras::Announcement::new([
            shadcn::extras::AnnouncementTag::new("New").into_element(cx),
            shadcn::extras::AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-announcement")
    };

    let banner = {
        let icon = shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info"));
        shadcn::extras::Banner::new([
            shadcn::extras::BannerIcon::new(icon).into_element(cx),
            shadcn::extras::BannerTitle::new("A new version is available.").into_element(cx),
            shadcn::extras::BannerAction::new("Upgrade").into_element(cx),
            shadcn::extras::BannerClose::new().into_element(cx),
        ])
        .inset(true)
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-banner")
    };

    let tags =
        shadcn::extras::Tags::new(["Alpha", "Beta", "Gamma", "A much longer tag label", "Zeta"])
            .into_element(cx)
            .test_id("ui-gallery-shadcn-extras-tags");

    let marquee = shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
        .pause_on_hover(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-marquee");

    let kanban = cx.named("shadcn-extras-kanban-demo", |cx| {
        let items = cx.with_state(KanbanModels::default, |st| st.items.clone());
        let items = items.unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(vec![
                shadcn::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
                shadcn::extras::KanbanItem::new("card-2", "Port block", "backlog"),
                shadcn::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
                shadcn::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
                shadcn::extras::KanbanItem::new("card-5", "Ship", "done"),
            ]);
            cx.with_state(KanbanModels::default, |st| {
                st.items = Some(model.clone());
            });
            model
        });

        let columns = vec![
            shadcn::extras::KanbanColumn::new("backlog", "Backlog"),
            shadcn::extras::KanbanColumn::new("in_progress", "In Progress"),
            shadcn::extras::KanbanColumn::new("done", "Done"),
        ];

        shadcn::extras::Kanban::new(columns, items)
            .test_id("ui-gallery-shadcn-extras-kanban")
            .into_element(cx)
    });

    let ticker_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap_x(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::extras::Ticker::new("AAPL")
                    .price("$199.18")
                    .change("+1.01%")
                    .change_kind(shadcn::extras::TickerChangeKind::Up)
                    .into_element(cx)
                    .test_id("ui-gallery-shadcn-extras-ticker-aapl"),
                shadcn::extras::Ticker::new("TSLA")
                    .price("$187.42")
                    .change("-2.31%")
                    .change_kind(shadcn::extras::TickerChangeKind::Down)
                    .into_element(cx)
                    .test_id("ui-gallery-shadcn-extras-ticker-tsla"),
            ]
        },
    )
    .test_id("ui-gallery-shadcn-extras-ticker-row");

    let relative_time = shadcn::extras::RelativeTime::new([
        shadcn::extras::RelativeTimeZone::new("UTC", "February 9, 2026", "15:04:05")
            .into_element(cx),
        shadcn::extras::RelativeTimeZone::new("PST", "February 9, 2026", "07:04:05")
            .into_element(cx),
        shadcn::extras::RelativeTimeZone::new("CET", "February 9, 2026", "16:04:05")
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-relative-time");

    let rating = shadcn::extras::Rating::uncontrolled(3)
        .count(5)
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-rating");

    let avatar_stack = {
        let a = shadcn::Avatar::new([shadcn::AvatarFallback::new("A").into_element(cx)]);
        let b = shadcn::Avatar::new([shadcn::AvatarFallback::new("B").into_element(cx)]);
        let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("C").into_element(cx)]);
        let d = shadcn::Avatar::new([shadcn::AvatarFallback::new("D").into_element(cx)]);
        let e = shadcn::Avatar::new([shadcn::AvatarFallback::new("E").into_element(cx)]);

        shadcn::extras::AvatarStack::new([a, b, c, d, e])
            .size_px(Px(40.0))
            .max_visible(4)
            .into_element(cx)
            .test_id("ui-gallery-shadcn-extras-avatar-stack")
    };

    vec![
        section_card(cx, "Announcement", announcement),
        section_card(cx, "Banner (dismissible)", banner),
        section_card(cx, "Tags", tags),
        section_card(cx, "Marquee (pause on hover)", marquee),
        section_card(cx, "Kanban (drag & drop)", kanban),
        section_card(cx, "Ticker", ticker_row),
        section_card(cx, "Relative time", relative_time),
        section_card(cx, "Rating", rating),
        section_card(cx, "Avatar stack", avatar_stack),
    ]
}
