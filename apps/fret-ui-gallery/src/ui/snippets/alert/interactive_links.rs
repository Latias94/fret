pub const SOURCE: &str = include_str!("interactive_links.rs");

// region: example
use std::sync::Arc;

use fret_core::{
    AttributedText, DecorationLineStyle, SemanticsRole, TextOverflow, TextPaintStyle, TextSpan,
    TextWrap, UnderlineStyle,
};
use fret_runtime::{Effect, Model};
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

fn underlined_rich_text(label: &'static str) -> AttributedText {
    let mut span = TextSpan::new(label.len());
    span.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });
    AttributedText::new(Arc::<str>::from(label), Arc::<[TextSpan]>::from([span]))
}

fn interactive_link<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    last_link: Model<Option<Arc<str>>>,
    label: &'static str,
    tag: &'static str,
    href: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app).snapshot();
    let diag_mode = is_diag_mode();
    let rich = underlined_rich_text(label);
    let label_arc: Arc<str> = Arc::from(label);
    let tag_arc: Arc<str> = Arc::from(tag);
    let href_arc: Arc<str> = Arc::from(href);
    let href_for_semantics = href_arc.clone();

    cx.pressable_with_id_props(move |cx, st, _id| {
        let last_link_for_activate = last_link.clone();
        let href_for_activate = href_arc.clone();
        let tag_for_activate = tag_arc.clone();
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !diag_mode && is_safe_open_url(&href_for_activate) {
                host.push_effect(Effect::OpenUrl {
                    url: href_for_activate.to_string(),
                    target: None,
                    rel: None,
                });
            }
            let _ = host.models_mut().update(&last_link_for_activate, |value| {
                *value = Some(tag_for_activate.clone())
            });
            host.notify(action_cx);
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
    let last_link = cx.local_model_keyed("last_link", || None::<Arc<str>>);

    let last_link_value = cx
        .app
        .models()
        .read(&last_link, |value| value.clone())
        .unwrap_or(None);

    let status = match last_link_value {
        Some(tag) => shadcn::Badge::new(format!("Activated: {tag}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .test_id(format!("ui-gallery-alert-link-status-{tag}")),
        None => ui::text("Activate a link to verify link routing.")
            .text_sm()
            .into_element(cx)
            .test_id("ui-gallery-alert-link-status-idle"),
    };

    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Need help resolving the billing issue?"),
                    shadcn::AlertDescription::new_children([
                        ui::text(
                            "Review the recovery resources below. These text-like links open safe URLs in normal runs while diagnostics still keep deterministic activation evidence.",
                        )
                        .into_element(cx),
                        interactive_link(
                            cx,
                            last_link.clone(),
                            "Open billing information",
                            "billing-information",
                            "https://example.com/billing-information",
                            "ui-gallery-alert-link-billing",
                        )
                        .into_element(cx),
                        interactive_link(
                            cx,
                            last_link.clone(),
                            "Open support article",
                            "support-article",
                            "https://example.com/support-article",
                            "ui-gallery-alert-link-support",
                        )
                        .into_element(cx),
                    ]),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-interactive-links"),
            status,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
