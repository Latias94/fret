macro_rules! tree_node_surface_methods {
    () => {
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

pub(crate) use tree_node_surface_methods;
