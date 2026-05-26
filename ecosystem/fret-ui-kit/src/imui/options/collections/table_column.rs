use std::sync::Arc;

use fret_core::Px;

use super::super::super::label_identity::parse_label_identity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableColumnWidth {
    Px(Px),
    Fill(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumnResizeOptions {
    pub min_width: Option<Px>,
    pub max_width: Option<Px>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableColumnPin {
    #[default]
    None,
    Left,
    Right,
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
    header: Option<Arc<str>>,
    id: Option<Arc<str>>,
    width: TableColumnWidth,
    visible: bool,
    sortable: bool,
    sort_direction: Option<TableSortDirection>,
    resize: Option<TableColumnResizeOptions>,
    pin: TableColumnPin,
}

impl TableColumn {
    pub fn px(header: impl Into<Arc<str>>, width: Px) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Px(width),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn fill(header: impl Into<Arc<str>>) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(1.0),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn weighted(header: impl Into<Arc<str>>, weight: f32) -> Self {
        let header = header.into();
        Self {
            id: inferred_column_id(header.as_ref()),
            header: Some(header),
            width: TableColumnWidth::Fill(weight),
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn unlabeled(width: TableColumnWidth) -> Self {
        Self {
            header: None,
            id: None,
            width,
            visible: true,
            sortable: false,
            sort_direction: None,
            resize: None,
            pin: TableColumnPin::None,
        }
    }

    pub fn with_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub(crate) fn header_arc(&self) -> Option<Arc<str>> {
        self.header.clone()
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub(crate) fn id_arc(&self) -> Option<Arc<str>> {
        self.id.clone()
    }

    pub fn width(&self) -> TableColumnWidth {
        self.width
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub(crate) fn set_visible_for_policy(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_sortable(&self) -> bool {
        self.sortable || self.sort_direction.is_some()
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn sort_direction(&self) -> Option<TableSortDirection> {
        self.sort_direction
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

    pub fn resizable(mut self) -> Self {
        self.resize = Some(TableColumnResizeOptions::default());
        self
    }

    pub fn resize_options(&self) -> Option<TableColumnResizeOptions> {
        self.resize
    }

    pub fn resizable_with_limits(mut self, min_width: Option<Px>, max_width: Option<Px>) -> Self {
        self.resize = Some(TableColumnResizeOptions {
            min_width,
            max_width,
        });
        self
    }

    pub fn pin(&self) -> TableColumnPin {
        self.pin
    }

    pub fn pinned_left(mut self) -> Self {
        self.pin = TableColumnPin::Left;
        self
    }

    pub fn pinned_right(mut self) -> Self {
        self.pin = TableColumnPin::Right;
        self
    }

    pub fn with_pin(mut self, pin: TableColumnPin) -> Self {
        self.pin = pin;
        self
    }
}

impl Default for TableColumnResizeOptions {
    fn default() -> Self {
        Self {
            min_width: Some(Px(32.0)),
            max_width: None,
        }
    }
}

fn inferred_column_id(header: &str) -> Option<Arc<str>> {
    let identity = parse_label_identity(header).identity;
    (!identity.is_empty()).then(|| Arc::from(identity))
}
