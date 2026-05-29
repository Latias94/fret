macro_rules! command_button_surface_methods {
    () => {
        fn button_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
            self.button_command_with_options(command, ButtonOptions::default())
        }

        fn button_command_with_options(
            &mut self,
            command: impl Into<CommandId>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_actions::button_command_with_options(self, command.into(), options)
        }
    };
}

pub(crate) use command_button_surface_methods;
