pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::AlertDialog::new(open.clone())
            .children([
                shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                    shadcn::Button::new("عرض الحوار")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-rtl-trigger"),
                )),
                shadcn::AlertDialogPart::content(shadcn::AlertDialogContent::build(|cx, out| {
                    out.push(
                        shadcn::AlertDialogHeader::new([
                            shadcn::AlertDialogTitle::new("هل أنت متأكد تمامًا؟")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "لا يمكن التراجع عن هذا الإجراء. سيؤدي ذلك إلى حذف حسابك نهائيًا وإزالة بياناتك من خوادمنا.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                    out.push(
                        shadcn::AlertDialogFooter::new([
                            shadcn::AlertDialogCancel::from_scope("إلغاء")
                                .test_id("ui-gallery-alert-dialog-rtl-cancel")
                                .into_element(cx),
                            shadcn::AlertDialogAction::from_scope("متابعة")
                                .test_id("ui-gallery-alert-dialog-rtl-action")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                })
                .test_id("ui-gallery-alert-dialog-rtl-content")),
            ])
            .into_element(cx)
    })
}
// endregion: example
