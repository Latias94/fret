pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_core::window::ColorScheme;
use fret_ui::Theme;
use fret_ui_kit::{ChromeRefinement, ColorRef};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Delete Chat")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-alert-dialog-destructive-trigger"),
            )),
            shadcn::AlertDialogPart::content(shadcn::AlertDialogContent::build(|cx, out| {
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

                out.push_ui(
                    cx,
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push_ui(cx, shadcn::AlertDialogTitle::new("Delete chat?"));
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogDescription::new(
                                "This will permanently delete this chat conversation. View Settings to delete any memories saved during this chat.",
                            ),
                        );
                    })
                    .media(media),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-alert-dialog-destructive-cancel"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogAction::from_scope("Delete")
                                .variant(shadcn::ButtonVariant::Destructive)
                                .test_id("ui-gallery-alert-dialog-destructive-action"),
                        );
                    }),
                );
            })
            .size(shadcn::AlertDialogContentSize::Sm)
            .test_id("ui-gallery-alert-dialog-destructive-content")),
        ])
        .into_element(cx)
}
// endregion: example
