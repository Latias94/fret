macro_rules! disclosure_surface_methods {
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

        /// Build a generic immediate tree node with explicit stable identity and explicit depth.
        ///
        /// Fret does not emulate ImGui's implicit ID/indent stack here. Child nodes should use
        /// their own stable ids (for example `"scene/root/camera"`) and set
        /// `TreeNodeOptions::level` explicitly instead of inventing `"##suffix"` tricks.
        fn tree_node(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            self.tree_node_with_options(id, label, TreeNodeOptions::default(), f)
        }

        fn tree_node_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            options: TreeNodeOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            disclosure_controls::tree_node_with_options(self, id, label.into(), options, f)
        }
    };
}

pub(super) use disclosure_surface_methods;
