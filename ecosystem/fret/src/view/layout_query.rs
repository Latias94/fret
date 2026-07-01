use fret_ui::{Invalidation, UiHost};

use super::AppUi;

impl<'cx, 'a, H: UiHost> AppUi<'cx, 'a, H> {
    /// Observe the committed bounds for a layout-query region from the default app-facing lane.
    pub fn layout_query_bounds(
        &mut self,
        region: fret_ui::GlobalElementId,
        invalidation: Invalidation,
    ) -> Option<fret_core::Rect> {
        self.cx.layout_query_bounds(region, invalidation)
    }

    /// Create a layout-query region on the default app-facing render lane and pass its region id.
    ///
    /// The nested builder keeps the same grouped action-registration surface as the outer `AppUi`
    /// scope instead of reopening the raw `ElementContext` lane.
    #[track_caller]
    pub fn layout_query_region_with_id<I>(
        &mut self,
        props: fret_ui::element::LayoutQueryRegionProps,
        f: impl for<'b> FnOnce(&mut AppUi<'b, 'a, H>, fret_ui::GlobalElementId) -> I,
    ) -> fret_ui::element::AnyElement
    where
        I: IntoIterator<Item = fret_ui::element::AnyElement>,
    {
        let action_root = self.action_root;
        let mut carried_action_handlers = Some(std::mem::take(&mut self.action_handlers));
        let mut carried_action_handlers_used = self.action_handlers_used;

        let out = self.cx.layout_query_region_with_id(props, |cx, id| {
            let action_handlers = carried_action_handlers
                .take()
                .expect("AppUi layout_query_region_with_id should carry handlers once");
            let mut nested = AppUi {
                cx,
                action_root,
                action_handlers,
                action_handlers_used: carried_action_handlers_used,
            };
            let built = f(&mut nested, id);
            carried_action_handlers = Some(nested.action_handlers);
            carried_action_handlers_used = nested.action_handlers_used;
            built
        });

        self.action_handlers = carried_action_handlers
            .take()
            .expect("AppUi layout_query_region_with_id should restore handlers");
        self.action_handlers_used = carried_action_handlers_used;
        out
    }

    /// Create a layout-query region on the default app-facing render lane.
    #[track_caller]
    pub fn layout_query_region<I>(
        &mut self,
        props: fret_ui::element::LayoutQueryRegionProps,
        f: impl for<'b> FnOnce(&mut AppUi<'b, 'a, H>) -> I,
    ) -> fret_ui::element::AnyElement
    where
        I: IntoIterator<Item = fret_ui::element::AnyElement>,
    {
        self.layout_query_region_with_id(props, |cx, _id| f(cx))
    }

    /// Read the committed viewport bounds from the default app-facing render lane.
    pub fn environment_viewport_bounds(&mut self, invalidation: Invalidation) -> fret_core::Rect {
        self.cx.environment_viewport_bounds(invalidation)
    }
}
