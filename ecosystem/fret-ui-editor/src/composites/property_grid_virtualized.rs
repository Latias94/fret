//! Virtualized property grid (VirtualList-backed).
//!
//! This is intended for large inspectors where building all rows eagerly becomes expensive.
//! The row UI remains fully composable (declarative subtrees), unlike the windowed-paint canvas
//! approach used by `fret-ui-kit` for non-composable row surfaces.

use std::panic::Location;
use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::VirtualListOptions;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::composites::property_row::{PropertyRowLayoutVariant, PropertyRowOptions};
use crate::primitives::EditorDensity;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;

#[derive(Debug, Clone)]
pub struct PropertyGridVirtualizedOptions {
    pub layout: LayoutStyle,
    pub label_width: Option<Px>,
    pub column_gap: Option<Px>,
    pub row_gap: Option<Px>,

    /// VirtualList overscan rows (per side).
    pub overscan: usize,
    /// Caller-provided revision to force VirtualList key cache refresh when item identities change.
    pub items_revision: u64,
    /// Explicit identity source for scroll state and row-local state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyGridVirtualizedOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            label_width: None,
            column_gap: None,
            row_gap: None,
            overscan: 6,
            items_revision: 0,
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct PropertyGridVirtualized {
    pub options: PropertyGridVirtualizedOptions,
}

impl PropertyGridVirtualized {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: PropertyGridVirtualizedOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        len: usize,
        mut key_at: impl FnMut(usize) -> fret_ui::ItemKey + 'static,
        mut row_at: impl FnMut(
            &mut ElementContext<'_, H>,
            usize,
            PropertyGridVirtualizedRowCx,
        ) -> AnyElement
        + 'static,
    ) -> AnyElement {
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.property_grid_virtualized", id_source),
                |cx| self.into_element_keyed(cx, len, &mut key_at, &mut row_at),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.property_grid_virtualized", callsite),
                |cx| self.into_element_keyed(cx, len, &mut key_at, &mut row_at),
            )
        }
    }

    fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        len: usize,
        key_at: &mut dyn FnMut(usize) -> fret_ui::ItemKey,
        row_at: &mut dyn FnMut(
            &mut ElementContext<'_, H>,
            usize,
            PropertyGridVirtualizedRowCx,
        ) -> AnyElement,
    ) -> AnyElement {
        let (density, row_cx, row_gap) = {
            let theme = Theme::global(&*cx.app);
            let metrics = InspectorLayoutMetrics::resolve(theme);
            let density = metrics.density;
            let column_gap = self.options.column_gap.unwrap_or(metrics.column_gap);
            let row_gap = self.options.row_gap.unwrap_or(metrics.row_gap);

            let row_options = PropertyRowOptions {
                label_width: self.options.label_width,
                gap: Some(column_gap),
                trailing_gap: Some(metrics.trailing_gap),
                value_max_width: Some(metrics.value_max_width),
                status_slot_width: Some(metrics.status_slot_width),
                reset_slot_width: Some(metrics.reset_slot_width),
                variant: PropertyRowLayoutVariant::Auto,
                ..Default::default()
            };

            (
                density,
                PropertyGridVirtualizedRowCx {
                    density,
                    row_options,
                },
                row_gap,
            )
        };

        let scroll = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());
        let mut list_options = VirtualListOptions::new(density.row_height, self.options.overscan);
        list_options.items_revision = self.options.items_revision;
        list_options.gap = row_gap;

        let mut root = cx.virtual_list_keyed_with_layout(
            self.options.layout,
            len,
            list_options,
            &scroll,
            move |i| key_at(i),
            move |cx, i| row_at(cx, i, row_cx.clone()),
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }

        root
    }
}

#[derive(Clone)]
pub struct PropertyGridVirtualizedRowCx {
    pub density: EditorDensity,
    pub row_options: PropertyRowOptions,
}
