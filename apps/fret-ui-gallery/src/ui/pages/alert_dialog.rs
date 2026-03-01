use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct AlertDialogModels {
        basic_open: Option<Model<bool>>,
        small_open: Option<Model<bool>>,
        media_open: Option<Model<bool>>,
        small_media_open: Option<Model<bool>>,
        destructive_open: Option<Model<bool>>,
        parts_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
    }

    let (
        basic_open,
        small_open,
        media_open,
        small_media_open,
        destructive_open,
        parts_open,
        rtl_open,
    ) = cx.with_state(AlertDialogModels::default, |state| {
        (
            state.basic_open.clone(),
            state.small_open.clone(),
            state.media_open.clone(),
            state.small_media_open.clone(),
            state.destructive_open.clone(),
            state.parts_open.clone(),
            state.rtl_open.clone(),
        )
    });

    let (
        basic_open,
        small_open,
        media_open,
        small_media_open,
        destructive_open,
        parts_open,
        rtl_open,
    ) = match (
        basic_open,
        small_open,
        media_open,
        small_media_open,
        destructive_open,
        parts_open,
        rtl_open,
    ) {
        (
            Some(basic_open),
            Some(small_open),
            Some(media_open),
            Some(small_media_open),
            Some(destructive_open),
            Some(parts_open),
            Some(rtl_open),
        ) => (
            basic_open,
            small_open,
            media_open,
            small_media_open,
            destructive_open,
            parts_open,
            rtl_open,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let small_open = cx.app.models_mut().insert(false);
            let media_open = cx.app.models_mut().insert(false);
            let small_media_open = cx.app.models_mut().insert(false);
            let destructive_open = cx.app.models_mut().insert(false);
            let parts_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);
            cx.with_state(AlertDialogModels::default, |state| {
                state.basic_open = Some(basic_open.clone());
                state.small_open = Some(small_open.clone());
                state.media_open = Some(media_open.clone());
                state.small_media_open = Some(small_media_open.clone());
                state.destructive_open = Some(destructive_open.clone());
                state.parts_open = Some(parts_open.clone());
                state.rtl_open = Some(rtl_open.clone());
            });
            (
                basic_open,
                small_open,
                media_open,
                small_media_open,
                destructive_open,
                parts_open,
                rtl_open,
            )
        }
    };

    let build_dialog = |cx: &mut ElementContext<'_, App>,
                        test_id_prefix: &'static str,
                        open_model: Model<bool>,
                        trigger_label: &'static str,
                        trigger_variant: shadcn::ButtonVariant,
                        title: &'static str,
                        description: &'static str,
                        cancel_label: &'static str,
                        action_label: &'static str,
                        action_variant: shadcn::ButtonVariant,
                        content_size: shadcn::AlertDialogContentSize,
                        media_icon: Option<&'static str>| {
        let open_for_trigger = open_model.clone();
        let open_for_children = open_model.clone();
        shadcn::AlertDialog::new(open_model).into_element(
            cx,
            move |cx| {
                shadcn::Button::new(trigger_label)
                    .variant(trigger_variant)
                    .test_id(format!("{test_id_prefix}-trigger"))
                    .toggle_model(open_for_trigger.clone())
                    .into_element(cx)
            },
            move |cx| {
                let mut header = shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new(title).into_element(cx),
                    shadcn::AlertDialogDescription::new(description).into_element(cx),
                ]);
                if let Some(icon_name) = media_icon {
                    let icon = shadcn::icon::icon_with(
                        cx,
                        fret_icons::IconId::new_static(icon_name),
                        Some(Px(32.0)),
                        None,
                    );
                    header = header.media(shadcn::AlertDialogMedia::new(icon).into_element(cx));
                }
                let header = header.into_element(cx);
                let footer = shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new(cancel_label, open_for_children.clone())
                        .test_id(format!("{test_id_prefix}-cancel"))
                        .into_element(cx),
                    shadcn::AlertDialogAction::new(action_label, open_for_children.clone())
                        .variant(action_variant)
                        .test_id(format!("{test_id_prefix}-action"))
                        .into_element(cx),
                ])
                .into_element(cx);

                shadcn::AlertDialogContent::new(vec![header, footer])
                    .size(content_size)
                    .into_element(cx)
                    .test_id(format!("{test_id_prefix}-content"))
            },
        )
    };

    let demo_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-demo",
        open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Are you absolutely sure?",
        "This action cannot be undone. This will permanently delete your account from our servers.",
        "Cancel",
        "Continue",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Default,
        None,
    );

    let basic_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-basic",
        basic_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Are you absolutely sure?",
        "This action cannot be undone. This will permanently delete your account from our servers.",
        "Cancel",
        "Continue",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Default,
        None,
    );

    let small_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-small",
        small_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Allow accessory to connect?",
        "Do you want to allow the USB accessory to connect to this device?",
        "Don't allow",
        "Allow",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Sm,
        None,
    );

    let media_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-media",
        media_open,
        "Share Project",
        shadcn::ButtonVariant::Outline,
        "Share this project?",
        "Anyone with the link will be able to view and edit this project.",
        "Cancel",
        "Share",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Default,
        Some("lucide.circle-plus"),
    );

    let small_with_media_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-small-media",
        small_media_open,
        "Show Dialog",
        shadcn::ButtonVariant::Outline,
        "Allow accessory to connect?",
        "Do you want to allow the USB accessory to connect to this device?",
        "Don't allow",
        "Allow",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Sm,
        Some("lucide.bluetooth"),
    );

    let destructive_content = build_dialog(
        cx,
        "ui-gallery-alert-dialog-destructive",
        destructive_open,
        "Delete Chat",
        shadcn::ButtonVariant::Destructive,
        "Delete chat?",
        "This will permanently delete this chat conversation. Review settings if you need to clear related memories.",
        "Cancel",
        "Delete",
        shadcn::ButtonVariant::Destructive,
        shadcn::AlertDialogContentSize::Default,
        Some("lucide.trash-2"),
    );

    let parts_content = {
        let open_for_trigger = parts_open.clone();
        let open_for_children = parts_open.clone();
        shadcn::AlertDialog::new(parts_open).into_element_parts(
            cx,
            move |cx| {
                let trigger = shadcn::Button::new("Show Dialog (Parts)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-parts-trigger")
                    .toggle_model(open_for_trigger.clone())
                    .into_element(cx);
                shadcn::AlertDialogTrigger::new(trigger)
            },
            shadcn::AlertDialogPortal::new(),
            shadcn::AlertDialogOverlay::new(),
            move |cx| {
                let header = shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("Part-based AlertDialog").into_element(cx),
                    shadcn::AlertDialogDescription::new(
                        "Thin adapters for shadcn-style authoring (Trigger/Portal/Overlay).",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                let footer = shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                        .test_id("ui-gallery-alert-dialog-parts-cancel")
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("Continue", open_for_children.clone())
                        .test_id("ui-gallery-alert-dialog-parts-action")
                        .into_element(cx),
                ])
                .into_element(cx);

                shadcn::AlertDialogContent::new(vec![header, footer])
                    .into_element(cx)
                    .test_id("ui-gallery-alert-dialog-parts-content")
            },
        )
    };

    let rtl_dialog = doc_layout::rtl(cx, |cx| {
        build_dialog(
            cx,
            "ui-gallery-alert-dialog-rtl",
            rtl_open,
            "عرض الحوار",
            shadcn::ButtonVariant::Outline,
            "هل أنت متأكد تمامًا؟",
            "لا يمكن التراجع عن هذا الإجراء. سيؤدي ذلك إلى حذف حسابك نهائيًا من خوادمنا.",
            "إلغاء",
            "متابعة",
            shadcn::ButtonVariant::Default,
            shadcn::AlertDialogContentSize::Default,
            None,
        )
    });

    let notes = doc_layout::notes(
        cx,
        [
            "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
            "Use `AlertDialogCancel` + `AlertDialogAction` with the same open model to guarantee close behavior stays predictable.",
            "Keep dialog copy concise and explicit, and ensure destructive actions have clear labels and visual hierarchy.",
        ],
    );

    let usage = doc_layout::notes(
        cx,
        [
            "Use `AlertDialog` when you need explicit confirmation for destructive/irreversible actions.",
            "Prefer `AlertDialogCancel` + `AlertDialogAction` over custom buttons to preserve consistent semantics and focus handling.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Alert Dialog docs order and keeps each state in a separate section for quick lookup.",
        ),
        vec![
            DocSection::new("Demo", demo_content)
                .description("Default-sized modal alert dialog.")
                .test_id_prefix("ui-gallery-alert-dialog-demo-docsec")
                .code(
                    "rust",
                    r#"shadcn::AlertDialog::new(open).into_element(
    cx,
    |cx| {
        shadcn::Button::new("Show Dialog")
            .toggle_model(open.clone())
            .into_element(cx)
    },
    |cx| {
        let header = shadcn::AlertDialogHeader::new([
            shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
            shadcn::AlertDialogDescription::new("This action cannot be undone.").into_element(cx),
        ])
        .into_element(cx);

        let footer = shadcn::AlertDialogFooter::new([
            shadcn::AlertDialogCancel::new("Cancel", open.clone()).into_element(cx),
            shadcn::AlertDialogAction::new("Continue", open.clone()).into_element(cx),
        ])
        .into_element(cx);

        shadcn::AlertDialogContent::new([header, footer])
            .into_element(cx)
    },
)"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Basic", basic_content)
                .description("A minimal alert dialog with default buttons.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);

let dialog = build_dialog(
    cx,
    "ui-gallery-alert-dialog-basic",
    open,
    "Show Dialog",
    shadcn::ButtonVariant::Outline,
    "Are you absolutely sure?",
    "This action cannot be undone.",
    "Cancel",
    "Continue",
    shadcn::ButtonVariant::Default,
    shadcn::AlertDialogContentSize::Default,
    None,
);"#,
                ),
            DocSection::new("Small", small_content)
                .description("Compact dialog size for short copy.")
                .test_id_prefix("ui-gallery-alert-dialog-small-docsec")
                .code(
                    "rust",
                    r#"shadcn::AlertDialogContent::new([...])
    .size(shadcn::AlertDialogContentSize::Sm)
    .into_element(cx);"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Media", media_content)
                .description("Dialogs can optionally show a leading media/icon in the header.")
                .test_id_prefix("ui-gallery-alert-dialog-media-docsec")
                .code(
                    "rust",
                    r#"let icon = shadcn::icon::icon_with(
    cx,
    fret_icons::IconId::new_static("lucide.circle-plus"),
    Some(Px(32.0)),
    None,
);

let header = shadcn::AlertDialogHeader::new([title, description])
    .media(shadcn::AlertDialogMedia::new(icon).into_element(cx));"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Small with Media", small_with_media_content)
                .description("Small size + media variant.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);

build_dialog(
    cx,
    "ui-gallery-alert-dialog-small-media",
    open,
    "Show Dialog",
    shadcn::ButtonVariant::Outline,
    "Allow accessory to connect?",
    "Do you want to allow the USB accessory to connect to this device?",
    "Don't allow",
    "Allow",
    shadcn::ButtonVariant::Default,
    shadcn::AlertDialogContentSize::Sm,
    Some("lucide.bluetooth"),
);"#,
                ),
            DocSection::new("Destructive", destructive_content)
                .description("Destructive styling for irreversible actions.")
                .test_id_prefix("ui-gallery-alert-dialog-destructive-docsec")
                .code(
                    "rust",
                    r#"shadcn::AlertDialogAction::new("Delete", open.clone())
    .variant(shadcn::ButtonVariant::Destructive)
    .into_element(cx);"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Parts", parts_content)
                .description("Part surface adapters for shadcn-style call sites.")
                .test_id_prefix("ui-gallery-alert-dialog-parts-docsec")
                .code(
                    "rust",
                    r#"use shadcn::{AlertDialog, AlertDialogOverlay, AlertDialogPortal, AlertDialogTrigger};

let dialog = AlertDialog::new(open).into_element_parts(
    cx,
    |cx| AlertDialogTrigger::new(shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx)),
    AlertDialogPortal::new(),
    AlertDialogOverlay::new(),
    |cx| shadcn::AlertDialogContent::new([/* content */]).into_element(cx),
);"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("RTL", rtl_dialog)
                .description("All shadcn components should work under an RTL direction provider.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| build_dialog(
        cx,
        "ui-gallery-alert-dialog-rtl",
        open,
        "عرض الحوار",
        shadcn::ButtonVariant::Outline,
        "هل أنت متأكد تمامًا؟",
        "لا يمكن التراجع عن هذا الإجراء.",
        "إلغاء",
        "متابعة",
        shadcn::ButtonVariant::Default,
        shadcn::AlertDialogContentSize::Default,
        None,
    ),
);"#,
                ),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Quick reference for composing an alert dialog.")
                .max_w(Px(760.0)),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-section-notes-title")
                .description("Guidelines and best practices for alert dialogs.")
                .max_w(Px(760.0)),
        ],
    );

    vec![body]
}
