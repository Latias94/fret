macro_rules! collapsing_header_surface_methods {
    () => {
        /// Build a generic immediate collapsing header with explicit stable identity.
        ///
        /// `id` must be stable and semantic across frames. Do not derive identity from the visible
        /// label alone; prefer domain keys such as `"scene.sections.rendering"`.
        fn collapsing_header(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            self.collapsing_header_with_options(id, label, CollapsingHeaderOptions::default(), f)
        }

        fn collapsing_header_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            options: CollapsingHeaderOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            disclosure_controls::collapsing_header_with_options(self, id, label.into(), options, f)
        }
    };
}

pub(crate) use collapsing_header_surface_methods;
