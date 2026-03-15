pub const SOURCE: &str = include_str!("rich_content.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                    ui::text(
                        "This removes the production project from all workspaces and revokes existing collaborator links.",
                    )
                    .into_element(cx),
                    ui::text(
                        "Export an audit archive and notify owners before continuing so the rollback plan is documented.",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

            let header = shadcn::AlertDialogHeader::new(vec![
                shadcn::AlertDialogTitle::new_children([cx.styled_text(rich_title_text())])
                    .into_element(cx),
                shadcn::AlertDialogDescription::new_children([description_body]).into_element(cx),
            ])
            .into_element(cx);

            let cancel_visual = ui::h_row(|cx| {
                vec![
                    fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.arrow-left")),
                    ui::text("Back to safety").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let action_visual = ui::h_row(|cx| {
                vec![
                    fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.trash-2")),
                    ui::text("Delete project").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let footer = shadcn::AlertDialogFooter::new(vec![
                shadcn::AlertDialogCancel::from_scope("Cancel")
                    .children([cancel_visual])
                    .a11y_label("Cancel deletion")
                    .test_id("ui-gallery-alert-dialog-rich-content-cancel")
                    .into_element(cx),
                shadcn::AlertDialogAction::from_scope("Delete project")
                    .children([action_visual])
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-alert-dialog-rich-content-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::AlertDialogContent::new(vec![header, footer])
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-rich-content")
        })
        .into_element(cx)
}
// endregion: example
