pub const SOURCE: &str = include_str!("interactive_links.rs");

// region: example
use std::sync::Arc;

use fret_core::{
    AttributedText, DecorationLineStyle, TextOverflow, TextPaintStyle, TextSpan, TextWrap,
    UnderlineStyle,
};
use fret_runtime::{Effect, Model};
use fret_ui::action::ActivateReason;
use fret_ui::element::{SelectableTextInteractiveSpan, SelectableTextProps};
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct DemoModels {
    last_link: Option<Model<Option<Arc<str>>>>,
}

fn is_diag_mode() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
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

fn interactive_link<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    last_link: Model<Option<Arc<str>>>,
    label: &'static str,
    tag: &'static str,
    href: &'static str,
    test_id: &'static str,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let diag_mode = is_diag_mode();
    let mut span = TextSpan::new(label.len());
    span.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });

    let rich = AttributedText::new(Arc::<str>::from(label), Arc::<[TextSpan]>::from([span]));
    let tag_arc: Arc<str> = Arc::from(tag);
    let href_arc: Arc<str> = Arc::from(href);

    cx.selectable_text_with_id_props(move |cx, id| {
        let last_link_for_activate = last_link.clone();
        let href_for_activate = href_arc.clone();
        cx.selectable_text_on_activate_span_for(
            id,
            Arc::new(
                move |host, action_cx, _reason: ActivateReason, activation| {
                    if !diag_mode && is_safe_open_url(&href_for_activate) {
                        host.push_effect(Effect::OpenUrl {
                            url: href_for_activate.to_string(),
                            target: None,
                            rel: None,
                        });
                    }
                    let _ = host.models_mut().update(&last_link_for_activate, |value| {
                        *value = Some(activation.tag.clone())
                    });
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                },
            ),
        );

        let mut props = SelectableTextProps::new(rich);
        props.wrap = TextWrap::WordBreak;
        props.overflow = TextOverflow::Clip;
        props.color = Some(theme.color_token("primary"));
        props.interactive_spans = Arc::from([SelectableTextInteractiveSpan {
            range: 0..label.len(),
            tag: tag_arc,
        }]);
        props
    })
    .test_id(test_id)
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let last_link = cx.with_state(DemoModels::default, |state| state.last_link.clone());
    let last_link = match last_link {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(DemoModels::default, |state| {
                state.last_link = Some(model.clone())
            });
            model
        }
    };

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
        None => ui::text("Activate a link to verify span routing.")
            .text_sm()
            .into_element(cx)
            .test_id("ui-gallery-alert-link-status-idle"),
    };

    ui::v_flex(|cx| {
        vec![
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new("Need help resolving the billing issue?").into_element(cx),
                shadcn::AlertDescription::new_children([
                    ui::text(
                        "Review the recovery resources below. The links are backed by selectable-text spans so diagnostics can verify activation deterministically.",
                    )
                    .into_element(cx),
                    interactive_link(
                        cx,
                        last_link.clone(),
                        "Open billing information",
                        "billing-information",
                        "https://example.com/billing-information",
                        "ui-gallery-alert-link-billing",
                    ),
                    interactive_link(
                        cx,
                        last_link.clone(),
                        "Open support article",
                        "support-article",
                        "https://example.com/support-article",
                        "ui-gallery-alert-link-support",
                    ),
                ])
                .into_element(cx),
            ])
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
