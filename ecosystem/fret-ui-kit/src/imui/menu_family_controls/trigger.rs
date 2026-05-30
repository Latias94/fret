use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::element::{PressableA11y, PressableProps};

use crate::imui::label_identity::parse_label_identity;
use crate::imui::{ResponseExt, UiWriterImUiFacadeExt};

use super::{ImUiMenubarPolicyState, visual};

mod behavior;

pub(super) fn menu_trigger_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    logical_key: Arc<str>,
    label: Arc<str>,
    open: bool,
    open_model: Model<bool>,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    enabled: bool,
    test_id: Option<Arc<str>>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::MenuItem),
            label: Some(label.clone()),
            test_id,
            expanded: Some(open),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            behavior::install_menu_trigger_behavior(
                cx,
                id,
                state,
                behavior::MenuTriggerBehaviorInput {
                    logical_key: logical_key.clone(),
                    open_model: open_model.clone(),
                    menubar_policy: menubar_policy.clone(),
                    enabled,
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            vec![visual::menu_trigger_visual(
                cx,
                label.clone(),
                open,
                enabled,
                state,
            )]
        })
    });

    ui.add(element);
    response
}
