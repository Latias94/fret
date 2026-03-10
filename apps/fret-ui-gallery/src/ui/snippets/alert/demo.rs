pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn link_like_rich_text(text: &'static str, underlined_fragment: &'static str) -> AttributedText {
    let full: Arc<str> = Arc::from(text);
    let start = text
        .find(underlined_fragment)
        .expect("underlined_fragment must exist in text");
    let end = start + underlined_fragment.len();
    let mut spans: Vec<TextSpan> = Vec::new();

    if start > 0 {
        spans.push(TextSpan::new(start));
    }

    let mut underlined = TextSpan::new(underlined_fragment.len());
    underlined.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });
    spans.push(underlined);

    if end < text.len() {
        spans.push(TextSpan::new(text.len() - end));
    }

    AttributedText::new(full, Arc::<[TextSpan]>::from(spans.into_boxed_slice()))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new_children([cx
                    .styled_text(link_like_rich_text(
                        "Let's try one with icon, title and a link.",
                        "link",
                    ))])
                .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-title-link"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertDescription::new_children([
                    cx.styled_text(link_like_rich_text(
                        "This one has an icon and a description only. No title. But it has a link and a second link.",
                        "link",
                    )),
                    cx.styled_text(link_like_rich_text(
                        "It also demonstrates a second link in the same description block.",
                        "second link",
                    )),
                ])
                .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-description-link"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new("Success! Your changes have been saved")
                    .into_element(cx),
                shadcn::AlertDescription::new("This is an alert with icon, title and description.")
                    .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-success"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new(
                    "This is a very long alert title that demonstrates how the component handles extended text content and potentially wraps across multiple lines",
                )
                .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-long-title"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertDescription::new(
                    "This is a very long alert description that demonstrates how the component handles extended text content and potentially wraps across multiple lines.",
                )
                .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-long-description"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new(
                    "This is an extremely long alert title that spans multiple lines to demonstrate how the component handles very lengthy headings while maintaining readability and proper text wrapping behavior",
                )
                .into_element(cx),
                shadcn::AlertDescription::new(
                    "This is an equally long description that contains detailed information about the alert. It shows how the component can accommodate extensive content while preserving proper spacing, alignment, and readability across different screen sizes and viewport widths.",
                )
                .into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-long-combined"),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-alert-demo")
}
// endregion: example
