pub const SOURCE: &str = include_str!("detached_trigger.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    handle: Option<shadcn::AlertDialogHandle>,
}

fn handle<H: UiHost>(cx: &mut ElementContext<'_, H>) -> shadcn::AlertDialogHandle {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.handle {
        Some(handle) => handle,
        None => {
            let handle = shadcn::AlertDialogHandle::new_controllable(cx, None, false);
            cx.with_state(Models::default, |st| st.handle = Some(handle.clone()));
            handle
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
            let header = shadcn::AlertDialogHeader::new(vec![
                shadcn::AlertDialogTitle::new("Delete the production project?").into_element(cx),
                shadcn::AlertDialogDescription::new(
                    "This dialog is mounted separately from its triggers. Closing restores focus to whichever detached trigger opened it most recently.",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let footer = shadcn::AlertDialogFooter::new(vec![
                shadcn::AlertDialogCancel::from_scope("Cancel")
                    .test_id("ui-gallery-alert-dialog-detached-trigger-cancel")
                    .into_element(cx),
                shadcn::AlertDialogAction::from_scope("Delete")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-alert-dialog-detached-trigger-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::AlertDialogContent::new(vec![header, footer])
                .into_element(cx)
                .test_id("ui-gallery-alert-dialog-detached-trigger-content")
        })
        .into_element(cx);

    ui::v_flex(|_cx| vec![toolbar_trigger, inline_trigger, dialog])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}
// endregion: example
