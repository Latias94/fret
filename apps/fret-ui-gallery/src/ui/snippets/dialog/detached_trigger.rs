pub const SOURCE: &str = include_str!("detached_trigger.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn handle<H: UiHost>(cx: &mut ElementContext<'_, H>) -> shadcn::DialogHandle {
    let slot = cx.keyed_slot_id("handle");
    let current = cx.state_for(slot, || None::<shadcn::DialogHandle>, |st| st.clone());
    match current {
        Some(handle) => handle,
        None => {
            let handle = shadcn::DialogHandle::new_controllable(cx, None, false);
            cx.state_for(
                slot,
                || None::<shadcn::DialogHandle>,
                |st| {
                    *st = Some(handle.clone());
                },
            );
            handle
        }
    }
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let handle = handle(cx);

    let toolbar_trigger = shadcn::DialogTrigger::new(
        shadcn::Button::new("Open From Toolbar")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-dialog-detached-trigger-toolbar")
            .into_element(cx),
    )
    .handle(handle.clone())
    .into_element(cx);

    let inline_trigger = shadcn::DialogTrigger::new(
        shadcn::Button::new("Open Share Dialog")
            .variant(shadcn::ButtonVariant::Secondary)
            .test_id("ui-gallery-dialog-detached-trigger-inline")
            .into_element(cx),
    )
    .handle(handle.clone())
    .into_element(cx);

    let dialog = shadcn::Dialog::from_handle(handle)
        .compose()
        .content_with(move |cx| {
            shadcn::DialogContent::new([])
                .refine_layout(LayoutRefinement::default().max_w(Px(425.0)))
                .with_children(cx, |cx| {
                    vec![
                        shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::DialogTitle::new("Share project").into_element(cx),
                                shadcn::DialogDescription::new(
                                    "This dialog is mounted separately from its triggers. Closing restores focus to whichever detached trigger opened it most recently.",
                                )
                                .into_element(cx),
                            ]
                        }),
                        shadcn::DialogFooter::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::DialogClose::from_scope().build(
                                    cx,
                                    shadcn::Button::new("Cancel")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .test_id("ui-gallery-dialog-detached-trigger-cancel"),
                                ),
                                shadcn::Button::new("Copy Link").into_element(cx),
                            ]
                        }),
                    ]
                })
                .test_id("ui-gallery-dialog-detached-trigger-content")
        })
        .into_element(cx);

    ui::v_flex(|_cx| vec![toolbar_trigger, inline_trigger, dialog])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}
// endregion: example
