use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn combo(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        self.combo_with_options(id, label, preview, ComboOptions::default(), f)
    }

    pub fn combo_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        preview: impl Into<Arc<str>>,
        options: ComboOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ComboResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::combo_with_options(
            self, id, label, preview, options, f,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
