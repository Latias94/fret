pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use std::sync::Arc;

use fret_core::{
    AttributedText, DecorationLineStyle, SemanticsRole, TextOverflow, TextPaintStyle, TextSpan,
    TextWrap, UnderlineStyle,
};
use fret_runtime::Effect;
use fret_ui::element::{PressableKeyActivation, PressableProps, StyledTextProps};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn is_diag_mode() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
}

fn is_safe_open_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return false;
    }

    let lower = url.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }

    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

fn link_like_rich_text(
    text: &'static str,
    underlined_fragment: &'static str,
) -> (AttributedText, std::ops::Range<usize>) {
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

    (
        AttributedText::new(full, Arc::<[TextSpan]>::from(spans.into_boxed_slice())),
        start..end,
    )
}

fn interactive_link_text<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    text: &'static str,
    underlined_fragment: &'static str,
    href: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let (rich, _range) = link_like_rich_text(text, underlined_fragment);
    let theme = Theme::global(&*cx.app).snapshot();
    let diag_mode = is_diag_mode();
    let label_arc: Arc<str> = Arc::from(text);
    let href_arc: Arc<str> = Arc::from(href);
    let href_for_semantics = href_arc.clone();

    cx.pressable_with_id_props(move |cx, st, _id| {
        let href_for_activate = href_arc.clone();
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !diag_mode && is_safe_open_url(&href_for_activate) {
                host.push_effect(Effect::OpenUrl {
                    url: href_for_activate.to_string(),
                    target: None,
                    rel: None,
                });
            }
            host.request_redraw(action_cx.window);
        }));

        let mut props = PressableProps::default();
        props.key_activation = PressableKeyActivation::EnterOnly;
        props.a11y.role = Some(SemanticsRole::Link);
        props.a11y.label = Some(label_arc.clone());

        let mut text_props = StyledTextProps::new(rich.clone());
        text_props.wrap = TextWrap::WordBreak;
        text_props.overflow = TextOverflow::Clip;
        text_props.color = Some(if st.hovered {
            theme.color_token("foreground")
        } else {
            theme.color_token("primary")
        });

        (props, [cx.styled_text_props(text_props)])
    })
    .a11y_value(href_for_semantics)
    .test_id(test_id)
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new_children([interactive_link_text(
                        cx,
                        "Let's try one with icon, title and a link.",
                        "link",
                        "https://example.com/alert-title-link",
                        "ui-gallery-alert-demo-title-link-inline",
                    )
                    .into_element(cx)]),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-title-link"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertDescription::new_children([
                        interactive_link_text(
                            cx,
                            "This one has an icon and a description only. No title. But it has a link and a second link.",
                            "link",
                            "https://example.com/alert-description-link",
                            "ui-gallery-alert-demo-description-link-primary",
                        )
                        .into_element(cx),
                        interactive_link_text(
                            cx,
                            "It also demonstrates a second link in the same description block.",
                            "second link",
                            "https://example.com/alert-description-second-link",
                            "ui-gallery-alert-demo-description-link-secondary",
                        )
                        .into_element(cx),
                    ]),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-description-link"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Success! Your changes have been saved"),
                    shadcn::AlertDescription::new(
                        "This is an alert with icon, title and description.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-success"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new(
                        "This is a very long alert title that demonstrates how the component handles extended text content and potentially wraps across multiple lines",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-long-title"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertDescription::new(
                        "This is a very long alert description that demonstrates how the component handles extended text content and potentially wraps across multiple lines.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-long-description"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new(
                        "This is an extremely long alert title that spans multiple lines to demonstrate how the component handles very lengthy headings while maintaining readability and proper text wrapping behavior",
                    ),
                    shadcn::AlertDescription::new(
                        "This is an equally long description that contains detailed information about the alert. It shows how the component can accommodate extensive content while preserving proper spacing, alignment, and readability across different screen sizes and viewport widths.",
                    ),
                ]
            })
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
