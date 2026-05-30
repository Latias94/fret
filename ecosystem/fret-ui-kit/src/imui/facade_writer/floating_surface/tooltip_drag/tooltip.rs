macro_rules! tooltip_surface_methods {
    () => {
        fn tooltip_text(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            text: impl Into<Arc<str>>,
        ) -> bool {
            self.tooltip_text_with_options(id, trigger, text, TooltipOptions::default())
        }

        fn tooltip_text_with_options(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            text: impl Into<Arc<str>>,
            options: TooltipOptions,
        ) -> bool {
            floating_popup::tooltip_text_with_options(self, id, trigger, text, options)
        }

        fn tooltip(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.tooltip_with_options(id, trigger, TooltipOptions::default(), f)
        }

        fn tooltip_with_options(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            options: TooltipOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::tooltip_with_options(self, id, trigger, options, f)
        }
    };
}

pub(crate) use tooltip_surface_methods;
