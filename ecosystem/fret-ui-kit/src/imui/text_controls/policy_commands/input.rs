use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::InputTextOptions;

mod resolve;

use resolve::{InputTextPolicyCommands, resolve_input_text_policy_command};

pub(in crate::imui::text_controls) fn install_input_text_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &InputTextOptions,
) {
    let commands = InputTextPolicyCommands::from_options(options);
    if commands.is_empty() {
        return;
    }

    cx.key_add_on_key_down_for(
        id,
        Arc::new(move |host, action_cx, down| {
            let Some(command) = resolve_input_text_policy_command(&commands, down) else {
                return false;
            };

            host.dispatch_command(Some(action_cx.window), command);
            true
        }),
    );
}
