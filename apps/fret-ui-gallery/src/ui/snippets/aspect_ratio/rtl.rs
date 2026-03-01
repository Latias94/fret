// region: example
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn ratio_example<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ratio: f32,
    max_w: Px,
    ratio_label: &'static str,
    caption: &'static str,
    test_id: &'static str,
    content_test_id: &'static str,
) -> AnyElement {
    fn center_align_text(mut element: AnyElement) -> AnyElement {
        use fret_ui::element::ElementKind;
        match &mut element.kind {
            ElementKind::Text(props) => props.align = fret_core::TextAlign::Center,
            ElementKind::StyledText(props) => props.align = fret_core::TextAlign::Center,
            ElementKind::SelectableText(props) => props.align = fret_core::TextAlign::Center,
            _ => {}
        }
        element
    }

    let text_block = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .items_center(),
        move |cx| {
            vec![
                center_align_text(shadcn::typography::h4(cx, ratio_label)),
                center_align_text(shadcn::typography::muted(cx, caption)),
            ]
        },
    )
    .test_id(content_test_id);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().h_full())
            .items_center()
            .justify_center()
            .gap(Space::N1),
        move |_cx| vec![text_block],
    );

    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let border = theme.color_token("border");

    shadcn::AspectRatio::new(ratio, content)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        move |cx| {
            ratio_example(
                cx,
                16.0 / 9.0,
                Px(384.0),
                "16:9",
                "RTL layout sample",
                "ui-gallery-aspect-ratio-rtl",
                "ui-gallery-aspect-ratio-rtl-content",
            )
        },
    )
}
// endregion: example

