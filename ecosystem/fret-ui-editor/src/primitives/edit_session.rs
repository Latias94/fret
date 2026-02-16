//! A small, reusable edit session primitive for editor-grade controls.
//!
//! This is intentionally policy-light: it only tracks pre-edit state and commit/cancel outcomes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditSessionOutcome {
    Committed,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct EditSession<T> {
    pre_edit: Option<T>,
}

impl<T: Clone> Default for EditSession<T> {
    fn default() -> Self {
        Self { pre_edit: None }
    }
}

impl<T: Clone> EditSession<T> {
    pub fn is_active(&self) -> bool {
        self.pre_edit.is_some()
    }

    pub fn begin(&mut self, current_value: T) {
        if self.pre_edit.is_none() {
            self.pre_edit = Some(current_value);
        }
    }

    pub fn pre_edit_value(&self) -> Option<&T> {
        self.pre_edit.as_ref()
    }

    pub fn commit(&mut self) -> Option<T> {
        self.pre_edit.take()
    }

    pub fn cancel(&mut self) -> Option<T> {
        self.pre_edit.take()
    }
}
