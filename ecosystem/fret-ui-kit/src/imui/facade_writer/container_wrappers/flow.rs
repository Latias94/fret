use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn items(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::items(self, build_focus, f);
    }

    pub fn items_with_options(
        &mut self,
        options: ItemFlowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::items_with_options(self, build_focus, options, f);
    }

    pub fn same_line(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::same_line(self, build_focus, f);
    }

    pub fn same_line_with_options(
        &mut self,
        options: SameLineOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::same_line_with_options(self, build_focus, options, f);
    }

    pub fn dummy(&mut self, size: Size) {
        container_methods::dummy(self, size);
    }

    pub fn dummy_with_options(&mut self, size: Size, options: DummyOptions) {
        container_methods::dummy_with_options(self, size, options);
    }

    pub fn spacing(&mut self) {
        container_methods::spacing(self);
    }

    pub fn spacing_with_options(&mut self, options: SpacingOptions) {
        container_methods::spacing_with_options(self, options);
    }

    pub fn indent(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::indent(self, build_focus, f);
    }

    pub fn indent_with_options(
        &mut self,
        options: IndentOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::indent_with_options(self, build_focus, options, f);
    }
}
