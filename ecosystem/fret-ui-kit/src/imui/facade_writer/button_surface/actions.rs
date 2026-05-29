macro_rules! action_button_surface_methods {
    () => {
        fn action_button(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
        ) -> ResponseExt {
            self.action_button_with_options(label, action, ButtonOptions::default())
        }

        fn action_button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::action_button_with_options(self, label.into(), action.into(), options)
        }

        fn action_payload_button<T>(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
            payload: T,
        ) -> ResponseExt
        where
            T: Any + Clone + Send + Sync + 'static,
        {
            self.action_payload_button_with_options(
                label,
                action,
                payload,
                ButtonOptions::default(),
            )
        }

        fn action_payload_button_with_options<T>(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
            payload: T,
            options: ButtonOptions,
        ) -> ResponseExt
        where
            T: Any + Clone + Send + Sync + 'static,
        {
            button_controls::action_payload_button_with_options(
                self,
                label.into(),
                action.into(),
                payload,
                options,
            )
        }

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

pub(crate) use action_button_surface_methods;
