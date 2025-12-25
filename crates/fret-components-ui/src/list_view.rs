use fret_core::{Event, Rect, Size};
use fret_runtime::Model;
use fret_ui::EventCx;
use fret_ui::Widget as UiWidget;
use fret_ui::primitives::{VecStringDataSource, VirtualList};
use fret_ui::{Invalidation, LayoutCx, PaintCx, Theme, UiHost};

use crate::recipes::list_row::{ListRowHeightMode, list_row_height, list_style};
use crate::{Sizable, Size as ComponentSize};

/// A simple, virtualized list view for `Vec<String>` items.
///
/// This is a convenience wrapper around `fret_ui::primitives::VirtualList<VecStringDataSource>`
/// that binds the
/// list contents and selection to models.
pub struct ListView {
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,
    size: ComponentSize,

    list: VirtualList<VecStringDataSource>,
    last_bounds: Rect,
    last_items_revision: Option<u64>,
    last_selection_revision: Option<u64>,
    last_theme_revision: Option<u64>,
}

impl ListView {
    pub fn new(items: Model<Vec<String>>) -> Self {
        Self {
            items,
            selection: None,
            size: ComponentSize::Medium,
            list: VirtualList::from_items(Vec::new()),
            last_bounds: Rect::default(),
            last_items_revision: None,
            last_selection_revision: None,
            last_theme_revision: None,
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn with_selection_model(mut self, selection: Model<Option<usize>>) -> Self {
        self.selection = Some(selection);
        self
    }

    fn sync_style(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.list.set_style(list_style(theme, self.size));
        self.list
            .set_row_height(list_row_height(theme, self.size, ListRowHeightMode::Fixed));
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

impl Sizable for ListView {
    fn with_size(self, size: ComponentSize) -> Self {
        ListView::with_size(self, size)
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
        self.sync_style(cx.theme());
        self.sync_models(cx);
        <VirtualList<VecStringDataSource> as UiWidget<H>>::layout(&mut self.list, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        <VirtualList<VecStringDataSource> as UiWidget<H>>::paint(&mut self.list, cx);
    }
}
