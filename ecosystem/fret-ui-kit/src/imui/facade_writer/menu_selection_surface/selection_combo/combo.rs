macro_rules! combo_surface_methods {
    () => {
        fn combo(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            preview: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ComboResponse {
            self.combo_with_options(id, label, preview, ComboOptions::default(), f)
        }

        fn combo_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            preview: impl Into<Arc<str>>,
            options: ComboOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ComboResponse {
            combo_controls::combo_with_options(self, id, label.into(), preview.into(), options, f)
        }
    };
}

pub(crate) use combo_surface_methods;
