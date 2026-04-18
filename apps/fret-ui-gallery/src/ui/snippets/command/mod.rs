//! Snippet-backed `Command` examples for UI Gallery.

use fret::AppComponentCx;
use fret_runtime::Model;
use std::sync::Arc;

pub(crate) fn last_action_model(cx: &mut AppComponentCx<'_>) -> Model<Arc<str>> {
    cx.local_model_keyed("ui-gallery-command-last-action", || {
        Arc::<str>::from("<none>")
    })
}

pub(crate) fn write_last_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    last_action: Model<Arc<str>>,
    value: Arc<str>,
) {
    let _ = host
        .models_mut()
        .update(&last_action, |current: &mut Arc<str>| {
            *current = value.clone();
        });
    host.request_redraw(action_cx.window);
}

pub(crate) fn on_select_for_last_action(
    last_action: Model<Arc<str>>,
) -> impl Fn(Arc<str>) -> fret_ui::action::OnActivate + Clone {
    move |tag: Arc<str>| {
        let last_action = last_action.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                write_last_action(host, action_cx, last_action.clone(), tag.clone());
            },
        ) as fret_ui::action::OnActivate
    }
}

pub mod action_first_view;
pub mod basic;
pub mod behavior_demos;
pub mod composable_shell;
pub mod docs_demo;
pub mod groups;
pub mod loading;
pub mod rtl;
pub mod scrollable;
pub mod shortcuts;
pub mod usage;
