use std::sync::Arc;

use fret_core::Px;

use super::super::label_identity::parse_label_identity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableColumnWidth {
    Px(Px),
    Fill(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSortDirection {
    Ascending,
    Descending,
}

impl TableColumnWidth {
    pub fn px(width: Px) -> Self {
        Self::Px(width)
    }

    pub fn fill(weight: f32) -> Self {
        Self::Fill(weight)
    }
}

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: Option<Arc<str>>,
    pub id: Option<Arc<str>>,
    pub width: TableColumnWidth,
    pub sortable: bool,
    pub sort_direction: Option<TableSortDirection>,
}

impl TableColumn {
    pub fn px(header: impl Into<Arc<str>>, width: Px) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Px(width),
            sortable: false,
            sort_direction: None,
        }
    }

    pub fn fill(header: impl Into<Arc<str>>) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(1.0),
            sortable: false,
            sort_direction: None,
        }
    }

    pub fn weighted(header: impl Into<Arc<str>>, weight: f32) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(weight),
            sortable: false,
            sort_direction: None,
        }
    }

    pub fn unlabeled(width: TableColumnWidth) -> Self {
        Self {
            header: None,
            id: None,
            width,
            sortable: false,
            sort_direction: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn sorted(mut self, direction: TableSortDirection) -> Self {
        self.sortable = true;
        self.sort_direction = Some(direction);
        self
    }

    pub fn with_sort_direction(mut self, direction: Option<TableSortDirection>) -> Self {
        self.sort_direction = direction;
        if direction.is_some() {
            self.sortable = true;
        }
        self
    }
}

fn inferred_column_id(header: &str) -> Option<Arc<str>> {
    let identity = parse_label_identity(header).identity;
    (!identity.is_empty()).then(|| Arc::from(identity))
}

#[derive(Debug, Clone)]
pub struct TableOptions {
    pub show_header: bool,
    pub striped: bool,
    pub clip_cells: bool,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            striped: false,
            clip_cells: true,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableRowOptions {
    pub test_id: Option<Arc<str>>,
}

#[derive(Clone)]
pub struct VirtualListOptions {
    /// Bounded viewport height for the virtualized list surface.
    pub viewport_height: Px,
    /// Estimated row height used by the runtime virtualizer.
    pub estimate_row_height: Px,
    /// Overscan row count per side.
    pub overscan: usize,
    /// Caller-provided revision bump when item identities or row-height inputs change.
    pub items_revision: u64,
    /// Runtime measure mode.
    pub measure_mode: fret_ui::element::VirtualListMeasureMode,
    /// Runtime key-cache policy.
    pub key_cache: fret_ui::element::VirtualListKeyCacheMode,
    /// Number of off-window rows a retained host may keep alive.
    pub keep_alive: usize,
    /// Inter-row gap owned by the runtime virtualizer.
    pub gap: Px,
    /// Virtualizer scroll-margin offset.
    pub scroll_margin: Px,
    /// Optional known row-height callback used when `measure_mode == Known`.
    pub known_row_height_at: Option<Arc<dyn Fn(usize) -> Px + Send + Sync>>,
    /// Optional external scroll handle.
    pub handle: Option<fret_ui::scroll::VirtualListScrollHandle>,
    pub test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VirtualListOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListOptions")
            .field("viewport_height", &self.viewport_height)
            .field("estimate_row_height", &self.estimate_row_height)
            .field("overscan", &self.overscan)
            .field("items_revision", &self.items_revision)
            .field("measure_mode", &self.measure_mode)
            .field("key_cache", &self.key_cache)
            .field("keep_alive", &self.keep_alive)
            .field("gap", &self.gap)
            .field("scroll_margin", &self.scroll_margin)
            .field("known_row_height_at", &self.known_row_height_at.is_some())
            .field("handle", &self.handle.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Default for VirtualListOptions {
    fn default() -> Self {
        Self {
            viewport_height: Px(240.0),
            estimate_row_height: Px(28.0),
            overscan: 6,
            items_revision: 0,
            measure_mode: fret_ui::element::VirtualListMeasureMode::Measured,
            key_cache: fret_ui::element::VirtualListKeyCacheMode::AllKeys,
            keep_alive: 0,
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            known_row_height_at: None,
            handle: None,
            test_id: None,
        }
    }
}
