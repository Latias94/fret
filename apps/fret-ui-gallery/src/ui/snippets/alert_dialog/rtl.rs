pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Px, TextOverflow, TextWrap};
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
                shadcn::AlertDialogPart::content_with(|cx| {
                    shadcn::AlertDialogContent::new([])
                        .test_id("ui-gallery-alert-dialog-rtl-content")
                        .with_children(cx, |cx| {
                            vec![
                                shadcn::AlertDialogHeader::new([]).with_children(cx, |cx| {
                                    vec![
                                        shadcn::AlertDialogTitle::new("هل أنت متأكد تمامًا؟")
                                            .into_element(cx),
                                        shadcn::AlertDialogDescription::new(
                                            "لا يمكن التراجع عن هذا الإجراء. سيؤدي هذا إلى حذف حسابك نهائيًا من خوادمنا.",
                                        )
                                        .into_element(cx),
                                    ]
                                }),
                                shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                                    vec![
                                        shadcn::AlertDialogCancel::from_scope("إلغاء")
                                            .test_id("ui-gallery-alert-dialog-rtl-cancel")
                                            .into_element(cx),
                                        shadcn::AlertDialogAction::from_scope("متابعة")
                                            .test_id("ui-gallery-alert-dialog-rtl-action")
                                            .into_element(cx),
                                    ]
                                }),
                            ]
                        })
                }),
            ])
            .into_element(cx);

        let small_dialog =
            shadcn::AlertDialog::new(small_open.clone())
                .children([
                    shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                        shadcn::Button::new("إظهار الحوار (صغير)")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-alert-dialog-rtl-small-trigger"),
                    )),
                    shadcn::AlertDialogPart::content_with(|cx| {
                        let icon = shadcn::raw::icon::icon_with(
                            cx,
                            fret_icons::IconId::new_static("lucide.bluetooth"),
                            Some(Px(32.0)),
                            None,
                        );
                        let media = shadcn::AlertDialogMedia::new(icon).into_element(cx);

                        shadcn::AlertDialogContent::new([])
                            .size(shadcn::AlertDialogContentSize::Sm)
                            .test_id("ui-gallery-alert-dialog-rtl-small-content")
                            .with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogHeader::new([])
                                        .media(media)
                                        .with_children(cx, |cx| {
                                            vec![
                                            shadcn::AlertDialogTitle::new_children([ui::text(
                                                "السماح للملحق بالاتصال؟",
                                            )
                                            .wrap(TextWrap::Word)
                                            .overflow(TextOverflow::Clip)
                                            .test_id("ui-gallery-alert-dialog-rtl-small-title")
                                            .into_element(cx)])
                                                .into_element(cx),
                                            shadcn::AlertDialogDescription::new_children([ui::text(
                                                "هل تريد السماح لملحق USB بالاتصال بهذا الجهاز؟",
                                            )
                                            .wrap(TextWrap::Word)
                                            .overflow(TextOverflow::Clip)
                                            .test_id(
                                                "ui-gallery-alert-dialog-rtl-small-description",
                                            )
                                            .into_element(cx)])
                                            .into_element(cx),
                                        ]
                                        }),
                                    shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                                        vec![
                                            shadcn::AlertDialogCancel::from_scope("عدم السماح")
                                                .test_id("ui-gallery-alert-dialog-rtl-small-cancel")
                                                .into_element(cx),
                                            shadcn::AlertDialogAction::from_scope("السماح")
                                                .test_id("ui-gallery-alert-dialog-rtl-small-action")
                                                .into_element(cx),
                                        ]
                                    }),
                                ]
                            })
                    }),
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
