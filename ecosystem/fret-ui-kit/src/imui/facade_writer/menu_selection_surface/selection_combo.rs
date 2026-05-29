macro_rules! selection_combo_surface_methods {
    () => {
        fn selectable(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
            self.selectable_with_options(
                label,
                SelectableOptions {
                    selected,
                    ..Default::default()
                },
            )
        }

        fn selectable_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: SelectableOptions,
        ) -> ResponseExt {
            selectable_controls::selectable_with_options(self, label.into(), options)
        }

        fn multi_selectable<K: Clone + PartialEq + 'static>(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
            all_keys: &[K],
            key: K,
        ) -> ResponseExt {
            self.multi_selectable_with_options(
                label,
                model,
                all_keys,
                key,
                SelectableOptions::default(),
            )
        }

        fn multi_selectable_with_options<K: Clone + PartialEq + 'static>(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
            all_keys: &[K],
            key: K,
            options: SelectableOptions,
        ) -> ResponseExt {
            multi_select::multi_selectable_with_options(
                self,
                label.into(),
                model,
                all_keys,
                key,
                options,
            )
        }

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

pub(crate) use selection_combo_surface_methods;
