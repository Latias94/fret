use fret_core::{Event, Rect, Size};
use fret_runtime::Model;
use fret_ui::Widget as UiWidget;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, UiHost, VecStringDataSource, VirtualList};

/// A simple, virtualized list view for `Vec<String>` items.
///
/// This is a convenience wrapper around `fret_ui::VirtualList<VecStringDataSource>` that binds the
/// list contents and selection to models.
pub struct ListView {
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,

    list: VirtualList<VecStringDataSource>,
    last_bounds: Rect,
    last_items_revision: Option<u64>,
    last_selection_revision: Option<u64>,
}

impl ListView {
    pub fn new(items: Model<Vec<String>>) -> Self {
        Self {
            items,
            selection: None,
            list: VirtualList::from_items(Vec::new()),
            last_bounds: Rect::default(),
            last_items_revision: None,
            last_selection_revision: None,
        }
    }

    pub fn with_selection_model(mut self, selection: Model<Option<usize>>) -> Self {
        self.selection = Some(selection);
        self
    }

    fn sync_models<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        cx.observe_model(self.items, Invalidation::Layout);
        let items_rev = cx.app.models().revision(self.items);
        if items_rev != self.last_items_revision {
            self.last_items_revision = items_rev;
            let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
            self.list.set_items(items);
        }

        if let Some(selection) = self.selection {
            cx.observe_model(selection, Invalidation::Paint);
            let sel_rev = cx.app.models().revision(selection);
            if sel_rev != self.last_selection_revision {
                self.last_selection_revision = sel_rev;
                let desired = cx.app.models().get(selection).cloned().unwrap_or(None);
                if self.list.selected_lead_key() != desired {
                    self.list.set_selected_key(desired);
                }
            }
        }
    }
}

impl<H: UiHost> UiWidget<H> for ListView {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.last_bounds = cx.bounds;

        let prev = self.list.selected_lead_key();
        <VirtualList<VecStringDataSource> as UiWidget<H>>::event(&mut self.list, cx, event);
        let next = self.list.selected_lead_key();

        if prev != next {
            if let Some(selection) = self.selection {
                let _ = cx.app.models_mut().update(selection, |v| *v = next);
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        self.sync_models(cx);
        <VirtualList<VecStringDataSource> as UiWidget<H>>::layout(&mut self.list, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        <VirtualList<VecStringDataSource> as UiWidget<H>>::paint(&mut self.list, cx);
    }
}
