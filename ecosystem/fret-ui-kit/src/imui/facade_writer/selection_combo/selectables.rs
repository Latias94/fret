use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn selectable(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
        self.selectable_with_options(
            label,
            SelectableOptions {
                selected,
                ..Default::default()
            },
        )
    }

    pub fn selectable_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: SelectableOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::selectable_with_options(self, label, options);
        self.record_focusable(resp.id(), focusable);
        resp
    }

    pub fn multi_selectable<K: Clone + PartialEq + 'static>(
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

    pub fn multi_selectable_with_options<K: Clone + PartialEq + 'static>(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
        all_keys: &[K],
        key: K,
        options: SelectableOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::multi_selectable_with_options(
            self, label, model, all_keys, key, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
