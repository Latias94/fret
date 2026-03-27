pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::window::ColorScheme;
use fret_core::{
    AttributedText, DecorationLineStyle, Px, TextPaintStyle, TextSpan, UnderlineStyle,
};
use fret_runtime::Effect;
use fret_ui::Theme;
use fret_ui::element::{SelectableTextInteractiveSpan, SelectableTextProps};
use fret_ui_kit::{ChromeRefinement, ColorRef};
use fret_ui_shadcn::facade as shadcn;

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

fn destructive_description_props(settings_color: fret_core::Color) -> SelectableTextProps {
    let settings_href: Arc<str> = Arc::from("https://example.com/settings");
    let text: Arc<str> = Arc::from(
        "This will permanently delete this chat conversation. View Settings to delete any memories saved during this chat.",
    );
    let prefix = "This will permanently delete this chat conversation. View ";
    let settings = "Settings";
    let suffix = " to delete any memories saved during this chat.";

    let plain = TextSpan::new(prefix.len());

    let mut settings_span = TextSpan::new(settings.len());
    settings_span.paint = TextPaintStyle::default()
        .with_fg(settings_color)
        .with_underline(UnderlineStyle {
            color: Some(settings_color),
            style: DecorationLineStyle::Solid,
        });

    let trailing = TextSpan::new(suffix.len());

    let rich = AttributedText::new(
        text,
        Arc::<[TextSpan]>::from([plain, settings_span, trailing]),
    );
    let mut props = SelectableTextProps::new(rich);
    props.interactive_spans = Arc::from([SelectableTextInteractiveSpan {
        range: prefix.len()..prefix.len() + settings.len(),
        tag: settings_href,
    }]);
    props
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Delete Chat")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-alert-dialog-destructive-trigger"),
            )),
            shadcn::AlertDialogPart::content_with(|cx| {
                let theme = Theme::global(&*cx.app).snapshot();
                let destructive_fg = theme
                    .color_by_key("destructive")
                    .unwrap_or_else(|| theme.color_token("destructive"));
                let destructive_bg = theme
                    .color_by_key(if theme.color_scheme == Some(ColorScheme::Dark) {
                        "destructive/20"
                    } else {
                        "destructive/10"
                    })
                    .unwrap_or_else(|| theme.color_token("muted"));
                let icon = shadcn::raw::icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.trash-2"),
                    Some(Px(32.0)),
                    None,
                );
                let diag_mode = is_diag_mode();
                let media = shadcn::AlertDialogMedia::new(icon)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(destructive_bg))
                            .text_color(ColorRef::Color(destructive_fg)),
                    )
                    .into_element(cx);

                shadcn::AlertDialogContent::new([])
                    .size(shadcn::AlertDialogContentSize::Sm)
                    .test_id("ui-gallery-alert-dialog-destructive-content")
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::AlertDialogHeader::new([])
                                .media(media)
                                .with_children(cx, |cx| {
                                    vec![
                                        shadcn::AlertDialogTitle::new("Delete chat?")
                                            .into_element(cx),
                                        shadcn::AlertDialogDescription::new_selectable_with(
                                            destructive_description_props(
                                                theme.color_token("primary"),
                                            ),
                                            Some(Arc::new(
                                                move |host, _action_cx, _reason, activation| {
                                                    if !diag_mode
                                                        && is_safe_open_url(&activation.tag)
                                                    {
                                                        host.push_effect(Effect::OpenUrl {
                                                            url: activation.tag.to_string(),
                                                            target: None,
                                                            rel: None,
                                                        });
                                                    }
                                                },
                                            )),
                                        )
                                        .into_element(cx)
                                        .test_id(
                                            "ui-gallery-alert-dialog-destructive-description-link",
                                        ),
                                    ]
                                }),
                            shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogCancel::from_scope("Cancel")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .test_id("ui-gallery-alert-dialog-destructive-cancel")
                                        .into_element(cx),
                                    shadcn::AlertDialogAction::from_scope("Delete")
                                        .variant(shadcn::ButtonVariant::Destructive)
                                        .test_id("ui-gallery-alert-dialog-destructive-action")
                                        .into_element(cx),
                                ]
                            }),
                        ]
                    })
            }),
        ])
        .into_element(cx)
}
// endregion: example
