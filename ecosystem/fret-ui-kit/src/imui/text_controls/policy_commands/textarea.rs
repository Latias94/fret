use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::TextAreaOptions;

mod resolve;

use resolve::{
    TextAreaPolicyCommandAction, TextAreaPolicyCommands, resolve_textarea_policy_command,
};

pub(in crate::imui::text_controls) fn install_textarea_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &TextAreaOptions,
) {
    let commands = TextAreaPolicyCommands::from_options(options);
    if commands.is_empty() {
        return;
    }

    cx.key_add_on_key_down_capture_for(
        id,
        Arc::new(move |host, action_cx, down| {
            match resolve_textarea_policy_command(&commands, down) {
                TextAreaPolicyCommandAction::Dispatch(command) => {
                    host.dispatch_command(Some(action_cx.window), command);
                    true
                }
                TextAreaPolicyCommandAction::Consume => true,
                TextAreaPolicyCommandAction::Ignore => false,
            }
        }),
    );
}
