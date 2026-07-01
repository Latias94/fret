use fret_ui::{ElementContext, UiHost};

use super::AppUi;

impl<'cx, 'a, H: UiHost> fret_ui::ElementContextAccess<'a, H> for AppUi<'cx, 'a, H> {
    fn elements(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }
}

impl<'cx, 'a, H: UiHost> fret_ui_kit::command::ElementCommandGatingExt for AppUi<'cx, 'a, H> {
    fn command_is_enabled(&self, command: &fret_runtime::CommandId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::command_is_enabled(
            &*self.cx, command,
        )
    }

    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &fret_runtime::CommandId,
        fallback_input_ctx: fret_runtime::InputContext,
    ) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::command_is_enabled_with_fallback_input_context(
            &*self.cx,
            command,
            fallback_input_ctx,
        )
    }

    fn dispatch_command_if_enabled(&mut self, command: fret_runtime::CommandId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_command_if_enabled(
            self.cx,
            command,
        )
    }

    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: fret_runtime::CommandId,
        fallback_input_ctx: fret_runtime::InputContext,
    ) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_command_if_enabled_with_fallback_input_context(
            self.cx,
            command,
            fallback_input_ctx,
        )
    }

    fn action_is_enabled(&self, action: &fret_runtime::ActionId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::action_is_enabled(
            &*self.cx, action,
        )
    }

    fn dispatch_action_if_enabled(&mut self, action: fret_runtime::ActionId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_action_if_enabled(
            self.cx,
            action,
        )
    }
}

impl<'cx, 'a, H: UiHost> fret_ui_kit::declarative::ElementContextThemeExt for AppUi<'cx, 'a, H> {
    fn with_theme<R>(&mut self, f: impl FnOnce(&fret_ui::Theme) -> R) -> R {
        f(self.cx.theme())
    }

    fn theme_snapshot(&mut self) -> fret_ui::ThemeSnapshot {
        self.cx.theme().snapshot()
    }
}
