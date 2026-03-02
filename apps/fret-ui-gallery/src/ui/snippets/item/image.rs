pub const SOURCE: &str = include_str!("image.rs");

// region: example
use crate::spec::CMD_APP_OPEN;
use fret_app::App;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(448.0)));

    let theme = Theme::global(&*cx.app).snapshot();

    let music = [
        (
            "Midnight City Lights",
            "Neon Dreams",
            "Electric Nights",
            "3:45",
        ),
        (
            "Coffee Shop Conversations",
            "The Morning Brew",
            "Urban Stories",
            "4:05",
        ),
        ("Digital Rain", "Cyber Symphony", "Binary Beats", "3:30"),
    ];

    let mut rows: Vec<AnyElement> = Vec::new();
    for (idx, (title, artist, album, duration)) in music.iter().copied().enumerate() {
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_token("muted")))
                .rounded(Radius::Sm),
            LayoutRefinement::default().size_full(),
        );
        let image = cx
            .container(props, move |cx| vec![shadcn::typography::muted(cx, "IMG")])
            .test_id(format!("ui-gallery-item-image-image-{idx}"));
        let media = shadcn::ItemMedia::new([image])
            .variant(shadcn::ItemMediaVariant::Image)
            .into_element(cx);

        let title_text: Arc<str> = Arc::from(format!("{title} - {album}"));
        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new(title_text).into_element(cx),
            shadcn::ItemDescription::new(artist).into_element(cx),
        ])
        .into_element(cx);

        let duration =
            shadcn::ItemContent::new([shadcn::ItemDescription::new(duration).into_element(cx)])
                .refine_layout(LayoutRefinement::default().flex_none())
                .into_element(cx);

        rows.push(
            shadcn::Item::new([media, content, duration])
                .variant(shadcn::ItemVariant::Outline)
                .render(shadcn::ItemRender::Link {
                    href: Arc::<str>::from("https://example.com/music"),
                    target: None,
                    rel: None,
                })
                .on_click(CMD_APP_OPEN)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id(format!("ui-gallery-item-image-{idx}")),
        );
    }

    let group = shadcn::ItemGroup::new(rows)
        .gap(MetricRef::space(Space::N4).resolve(&theme))
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-image-group");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(max_w_md),
        |_cx| vec![group],
    )
    .test_id("ui-gallery-item-image")
}
// endregion: example
