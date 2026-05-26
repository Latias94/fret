use fret_core::Px;

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

impl Default for TableColumnResizeOptions {
    fn default() -> Self {
        Self {
            min_width: Some(Px(32.0)),
            max_width: None,
        }
    }
}
