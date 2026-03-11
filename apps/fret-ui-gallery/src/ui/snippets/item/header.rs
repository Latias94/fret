pub const SOURCE: &str = include_str!("header.rs");

// region: example
use fret_app::App;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xl = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(MetricRef::Px(Px(576.0)));

    let theme = Theme::global(&*cx.app).snapshot();

    let models = [
        ("v0-1.5-sm", "Everyday tasks and UI generation."),
        ("v0-1.5-lg", "Advanced thinking or reasoning."),
        ("v0-2.0-mini", "Open Source model for everyone."),
    ];

    let mut children: Vec<AnyElement> = Vec::new();
    for (idx, (name, description)) in models.iter().copied().enumerate() {
        let header = {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .rounded(Radius::Sm),
                LayoutRefinement::default()
                    .w_full()
                    .aspect_ratio(1.0)
                    .overflow_hidden(),
            );
            let image = cx
                .container(props, move |cx| {
                    vec![shadcn::raw::typography::muted(cx, "IMG")]
                })
                .test_id(format!("ui-gallery-item-header-image-{idx}"));
            shadcn::ItemHeader::new([image]).into_element(cx)
        };

        let content = shadcn::ItemContent::new([
            shadcn::ItemTitle::new(name).into_element(cx),
            shadcn::ItemDescription::new(description).into_element(cx),
        ])
        .into_element(cx);

        children.push(
            shadcn::Item::new([header, content])
                .variant(shadcn::ItemVariant::Outline)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id(format!("ui-gallery-item-header-{idx}")),
        );
    }

    let group = shadcn::ItemGroup::new(children)
        .grid(3)
        .gap(MetricRef::space(Space::N4).resolve(&theme))
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-item-header-group");

    ui::v_stack(|_cx| vec![group])
        .gap(Space::N6)
        .items_start()
        .layout(max_w_xl)
        .into_element(cx)
        .test_id("ui-gallery-item-header")
}
// endregion: example
