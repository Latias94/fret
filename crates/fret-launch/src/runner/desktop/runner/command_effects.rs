use super::{WinitAppDriver, WinitCommandContext, WinitGlobalContext, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_command_effect(
        &mut self,
        window: Option<fret_core::AppWindowId>,
        command: fret_app::CommandId,
    ) {
        match window {
            Some(window) => {
                if let Some(state) = self.windows.get_mut(window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_command(
                        WinitCommandContext {
                            app: &mut self.app,
                            services,
                            window,
                            state: &mut state.user,
                        },
                        command,
                    );
                }
            }
            None => {
                let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                self.driver.handle_global_command(
                    WinitGlobalContext {
                        app: &mut self.app,
                        services,
                    },
                    command,
                );
            }
        }
    }
}
