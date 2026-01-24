use fret_runtime::{CommandId, InputContext, Platform, PlatformCapabilities};
use fret_ui::{ElementContext, UiHost};

pub fn default_fallback_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext::fallback(Platform::current(), caps)
}

pub trait ElementCommandGatingExt {
    fn command_is_enabled(&self, command: &CommandId) -> bool;
    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool;

    fn dispatch_command_if_enabled(&mut self, command: CommandId) -> bool;
    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool;
}

impl<H: UiHost> ElementCommandGatingExt for ElementContext<'_, H> {
    fn command_is_enabled(&self, command: &CommandId) -> bool {
        let fallback_input_ctx = default_fallback_input_context(&*self.app);
        fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            command,
            fallback_input_ctx,
        )
    }

    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool {
        fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            command,
            fallback_input_ctx,
        )
    }

    fn dispatch_command_if_enabled(&mut self, command: CommandId) -> bool {
        let fallback_input_ctx = default_fallback_input_context(&*self.app);
        self.dispatch_command_if_enabled_with_fallback_input_context(command, fallback_input_ctx)
    }

    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool {
        if !fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            &command,
            fallback_input_ctx,
        ) {
            return false;
        }
        self.app.push_effect(fret_runtime::Effect::Command {
            window: Some(self.window),
            command,
        });
        true
    }
}
