macro_rules! payload_action_button_surface_methods {
    () => {
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
    };
}

pub(crate) use payload_action_button_surface_methods;
