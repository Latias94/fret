use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::horizontal(self, build_focus, f);
    }

    pub fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::horizontal_with_options(self, build_focus, options, f);
    }

    pub fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::vertical(self, build_focus, f);
    }

    pub fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::vertical_with_options(self, build_focus, options, f);
    }

    pub fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::grid(self, build_focus, f);
    }

    pub fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::grid_with_options(self, build_focus, options, f);
    }

    pub fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::scroll(self, build_focus, f);
    }

    pub fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::scroll_with_options(self, build_focus, options, f);
    }

    pub fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let build_focus = self.build_focus.clone();
        container_methods::child_region(self, build_focus, id, f)
    }

    pub fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let build_focus = self.build_focus.clone();
        container_methods::child_region_with_options(self, build_focus, id, options, f)
    }
}
