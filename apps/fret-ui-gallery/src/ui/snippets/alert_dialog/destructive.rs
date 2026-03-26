pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::window::ColorScheme;
use fret_core::{
    AttributedText, DecorationLineStyle, Px, TextPaintStyle, TextSpan, UnderlineStyle,
};
use fret_ui::Theme;
use fret_ui_kit::{ChromeRefinement, ColorRef};
use fret_ui_shadcn::facade as shadcn;

fn destructive_description_text(settings_color: fret_core::Color) -> AttributedText {
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

    AttributedText::new(
        text,
        Arc::<[TextSpan]>::from([plain, settings_span, trailing]),
    )
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
                                        shadcn::AlertDialogDescription::new_children([cx
                                            .styled_text(destructive_description_text(
                                                theme.color_token("primary"),
                                            ))])
                                        .into_element(cx),
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
