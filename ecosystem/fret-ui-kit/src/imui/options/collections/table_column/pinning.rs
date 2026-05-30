use super::{TableColumn, TableColumnPin};

impl TableColumn {
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
