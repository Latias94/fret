pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let default_open = cx.local_model_keyed("default_open", || false);
    let small_open = cx.local_model_keyed("small_open", || false);

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        let default_dialog = shadcn::AlertDialog::new(default_open.clone())
            .children([
                shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                    shadcn::Button::new("إظهار الحوار")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-rtl-trigger"),
                )),
                shadcn::AlertDialogPart::content(shadcn::AlertDialogContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::AlertDialogHeader::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogTitle::new("هل أنت متأكد تمامًا؟"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogDescription::new(
                                    "لا يمكن التراجع عن هذا الإجراء. سيؤدي هذا إلى حذف حسابك نهائيًا من خوادمنا.",
                                ),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::AlertDialogFooter::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogCancel::from_scope("إلغاء")
                                    .test_id("ui-gallery-alert-dialog-rtl-cancel"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogAction::from_scope("متابعة")
                                    .test_id("ui-gallery-alert-dialog-rtl-action"),
                            );
                        }),
                    );
                })
                .test_id("ui-gallery-alert-dialog-rtl-content")),
            ])
            .into_element(cx);

        let small_dialog = shadcn::AlertDialog::new(small_open.clone())
            .children([
                shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                    shadcn::Button::new("إظهار الحوار (صغير)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-rtl-small-trigger"),
                )),
                shadcn::AlertDialogPart::content(
                    shadcn::AlertDialogContent::build(|cx, out| {
                        let icon = shadcn::raw::icon::icon_with(
                            cx,
                            fret_icons::IconId::new_static("lucide.bluetooth"),
                            Some(Px(32.0)),
                            None,
                        );
                        let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                        out.push_ui(
                            cx,
                            shadcn::AlertDialogHeader::build(|cx, out| {
                                out.push_ui(
                                    cx,
                                    shadcn::AlertDialogTitle::new("السماح للملحق بالاتصال؟"),
                                );
                                out.push_ui(
                                    cx,
                                    shadcn::AlertDialogDescription::new(
                                        "هل تريد السماح لملحق USB بالاتصال بهذا الجهاز؟",
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
                                    shadcn::AlertDialogCancel::from_scope("عدم السماح")
                                        .test_id("ui-gallery-alert-dialog-rtl-small-cancel"),
                                );
                                out.push_ui(
                                    cx,
                                    shadcn::AlertDialogAction::from_scope("السماح")
                                        .test_id("ui-gallery-alert-dialog-rtl-small-action"),
                                );
                            }),
                        );
                    })
                    .size(shadcn::AlertDialogContentSize::Sm)
                    .test_id("ui-gallery-alert-dialog-rtl-small-content"),
                ),
            ])
            .into_element(cx);

        ui::h_flex(|_cx| vec![default_dialog, small_dialog])
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
    })
}
// endregion: example
