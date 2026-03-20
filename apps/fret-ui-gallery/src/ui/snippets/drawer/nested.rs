pub const SOURCE: &str = include_str!("nested.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Open parent drawer")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-drawer-nested-parent-trigger"),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
                let child = shadcn::Drawer::new_controllable(cx, None, false)
                    .modal(false)
                    .children([
                        shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                            shadcn::Button::new("Open nested drawer")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-drawer-nested-child-trigger"),
                        )),
                        shadcn::DrawerPart::content_with(move |cx| {
                            shadcn::DrawerContent::build(|cx, out| {
                                out.push_ui(
                                    cx,
                                    shadcn::DrawerHeader::build(|cx, out| {
                                        out.push_ui(cx, shadcn::DrawerTitle::new("Nested drawer"));
                                        out.push_ui(
                                            cx,
                                            shadcn::DrawerDescription::new(
                                                "This follow-up keeps the child draggable without letting the parent steal the handle.",
                                            ),
                                        );
                                    }),
                                );
                                out.push_ui(
                                    cx,
                                    shadcn::DrawerFooter::build(|cx, out| {
                                        let close = shadcn::DrawerClose::from_scope().build(
                                            cx,
                                            shadcn::Button::new("Close nested drawer")
                                                .variant(shadcn::ButtonVariant::Outline)
                                                .test_id("ui-gallery-drawer-nested-child-close"),
                                        );
                                        out.push(close);
                                    }),
                                );
                            })
                            .drag_handle_test_id("ui-gallery-drawer-nested-child-handle")
                            .test_id("ui-gallery-drawer-nested-child-content")
                            .into_element(cx)
                        }),
                    ])
                    .into_element(cx);

                let body = ui::v_stack(|cx| {
                    vec![
                        shadcn::DrawerHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::DrawerTitle::new("Parent drawer"));
                            out.push_ui(
                                cx,
                                shadcn::DrawerDescription::new(
                                    "Nested drawers stay outside the shadcn docs path, but Fret now keeps a non-modal child above the parent barrier for input routing.",
                                ),
                            );
                        })
                        .into_element(cx),
                        ui::v_stack(move |cx| {
                            vec![
                                ui::text(
                                    "Open the child drawer, then drag its handle. The child should track the drag while the parent remains stationary.",
                                )
                                .text_sm()
                                .into_element(cx),
                                child,
                            ]
                        })
                        .gap(Space::N4)
                        .px_4()
                        .pb(Space::N0)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                        shadcn::DrawerFooter::build(|cx, out| {
                            let close = shadcn::DrawerClose::from_scope().build(
                                cx,
                                shadcn::Button::new("Close parent drawer")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-drawer-nested-parent-close"),
                            );
                            out.push(close);
                        })
                        .into_element(cx),
                    ]
                })
                .gap(Space::N0)
                .items_stretch()
                .layout(
                    LayoutRefinement::default()
                        .w_full()
                        .max_w(Px(384.0))
                        .min_w_0()
                        .mx_auto(),
                    )
                    .into_element(cx);

                shadcn::DrawerContent::build(move |_cx, out| {
                    out.push(body);
                })
                .test_id("ui-gallery-drawer-nested-parent-content")
                .into_element(cx)
            }),
        ])
        .into_element(cx)
}
// endregion: example
