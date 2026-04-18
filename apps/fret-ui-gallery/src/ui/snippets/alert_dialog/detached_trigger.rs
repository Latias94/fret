pub const SOURCE: &str = include_str!("detached_trigger.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn handle<H: UiHost>(cx: &mut ElementContext<'_, H>) -> shadcn::AlertDialogHandle {
    let slot = cx.keyed_slot_id("handle");
    let current = cx.state_for(slot, || None::<shadcn::AlertDialogHandle>, |st| st.clone());
    match current {
        Some(handle) => handle,
        None => {
            let handle = shadcn::AlertDialogHandle::new_controllable(cx, None, false);
            cx.state_for(
                slot,
                || None::<shadcn::AlertDialogHandle>,
                |st| {
                    *st = Some(handle.clone());
                },
            );
            handle
        }
    }
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let handle = handle(cx);

    let toolbar_trigger = shadcn::AlertDialogTrigger::new(
        shadcn::Button::new("Open From Toolbar")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-alert-dialog-detached-trigger-toolbar")
            .into_element(cx),
    )
    .handle(handle.clone())
    .into_element(cx);

    let inline_trigger = shadcn::AlertDialogTrigger::new(
        shadcn::Button::new("Open Danger Prompt")
            .variant(shadcn::ButtonVariant::Destructive)
            .test_id("ui-gallery-alert-dialog-detached-trigger-inline")
            .into_element(cx),
    )
    .handle(handle.clone())
    .into_element(cx);

    let dialog = shadcn::AlertDialog::from_handle(handle)
        .compose()
        .content_with(move |cx| {
            shadcn::AlertDialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogTitle::new("Delete the production project?"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogDescription::new(
                                "This dialog is mounted separately from its triggers. Closing restores focus to whichever detached trigger opened it most recently.",
                            ),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .test_id("ui-gallery-alert-dialog-detached-trigger-cancel"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogAction::from_scope("Delete")
                                .variant(shadcn::ButtonVariant::Destructive)
                                .test_id("ui-gallery-alert-dialog-detached-trigger-action"),
                        );
                    }),
                );
            })
            .test_id("ui-gallery-alert-dialog-detached-trigger-content")
            .into_element(cx)
        })
        .into_element(cx);

    ui::v_flex(|_cx| vec![toolbar_trigger, inline_trigger, dialog])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}
// endregion: example
