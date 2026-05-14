use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn collapsing_header(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.collapsing_header_with_options(id, label, CollapsingHeaderOptions::default(), f)
    }

    pub fn collapsing_header_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: CollapsingHeaderOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::collapsing_header_with_options(
            self, id, label, options, f,
        );
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn tree_node(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.tree_node_with_options(id, label, TreeNodeOptions::default(), f)
    }

    pub fn tree_node_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TreeNodeOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::tree_node_with_options(self, id, label, options, f);
        self.record_focusable(resp.id(), enabled);
        resp
    }
}
