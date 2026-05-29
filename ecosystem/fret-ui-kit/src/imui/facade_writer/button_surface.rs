macro_rules! button_surface_methods {
    () => {
        fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
            self.button_with_options(label, ButtonOptions::default())
        }

        fn small_button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
            self.small_button_with_options(label, ButtonOptions::default())
        }

        fn small_button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::small_button_with_options(self, label.into(), options)
        }

        fn arrow_button(&mut self, id: &str, direction: ButtonArrowDirection) -> ResponseExt {
            self.arrow_button_with_options(id, direction, ButtonOptions::default())
        }

        fn arrow_button_with_options(
            &mut self,
            id: &str,
            direction: ButtonArrowDirection,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::arrow_button_with_options(self, id, direction, options)
        }

        fn invisible_button(&mut self, id: &str, size: Size) -> ResponseExt {
            self.invisible_button_with_options(id, size, ButtonOptions::default())
        }

        fn invisible_button_with_options(
            &mut self,
            id: &str,
            size: Size,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::invisible_button_with_options(self, id, size, options)
        }

        fn image_item(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
            self.image_item_with_options(id, image, size, ImageItemOptions::default())
        }

        fn image_item_with_options(
            &mut self,
            id: &str,
            image: fret_core::ImageId,
            size: Size,
            options: ImageItemOptions,
        ) -> ResponseExt {
            image_items::image_item_with_options(self, id, image, size, options)
        }

        fn image_button(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
            self.image_button_with_options(id, image, size, ImageItemOptions::button())
        }

        fn image_button_with_options(
            &mut self,
            id: &str,
            image: fret_core::ImageId,
            size: Size,
            options: ImageItemOptions,
        ) -> ResponseExt {
            image_items::image_button_with_options(self, id, image, size, options)
        }

        fn button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::button_with_options(self, label.into(), options)
        }

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

pub(super) use button_surface_methods;
