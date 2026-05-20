pub const SOURCE: &str = include_str!("rich_content.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{AppComponentCx, UiChild};
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rich_title_text() -> AttributedText {
    let text: Arc<str> = Arc::from("Delete project and revoke shared access?");
    let prefix = "Delete project and revoke ";
    let emphasis = "shared access";
    let suffix = "?";

    let plain = TextSpan::new(prefix.len());

    let mut underlined = TextSpan::new(emphasis.len());
    underlined.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });

    let trailing = TextSpan::new(suffix.len());

    AttributedText::new(text, Arc::<[TextSpan]>::from([plain, underlined, trailing]))
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    let trigger = shadcn::AlertDialogTrigger::new(
        shadcn::Button::new("Preview Rich Content")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-alert-dialog-rich-content-trigger")
            .into_element(cx),
    );

    shadcn::AlertDialog::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::AlertDialogPortal::new())
        .overlay(shadcn::AlertDialogOverlay::new())
        .content_with(move |cx| {
            let description_body = ui::v_flex(|cx| {
                vec![
                    decl_text::text_paragraph(
                        cx,
                        "This removes the production project from all workspaces and revokes existing collaborator links.",
                    ),
                    decl_text::text_paragraph(
                        cx,
                        "Export an audit archive and notify owners before continuing so the rollback plan is documented.",
                    ),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

            let cancel_visual = ui::h_row(|cx| {
                vec![
                    icon::icon(cx, fret_icons::IconId::new_static("lucide.arrow-left")),
                    decl_text::text_button_label(cx, "Back to safety"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let action_visual = ui::h_row(|cx| {
                vec![
                    icon::icon(cx, fret_icons::IconId::new_static("lucide.trash-2")),
                    decl_text::text_button_label(cx, "Delete project"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            shadcn::AlertDialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        let title = shadcn::AlertDialogTitle::new_children([cx
                            .styled_text(rich_title_text())]);
                        out.push_ui(
                            cx,
                            title,
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogDescription::new_children([description_body]),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .children([cancel_visual])
                                .a11y_label("Cancel deletion")
                                .test_id("ui-gallery-alert-dialog-rich-content-cancel"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogAction::from_scope("Delete project")
                                .children([action_visual])
                                .variant(shadcn::ButtonVariant::Destructive)
                                .test_id("ui-gallery-alert-dialog-rich-content-action"),
                        );
                    }),
                );
            })
            .test_id("ui-gallery-alert-dialog-rich-content")
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
