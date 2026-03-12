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

impl<T: Clone + PartialEq> EditSession<T> {
    pub fn changed_from(&self, current_value: &T) -> bool {
        self.pre_edit
            .as_ref()
            .is_some_and(|pre_edit| pre_edit != current_value)
    }
}

#[cfg(test)]
mod tests {
    use super::EditSession;

    #[test]
    fn changed_from_requires_an_active_session() {
        let session = EditSession::<String>::default();
        assert!(!session.changed_from(&"draft".to_string()));
    }

    #[test]
    fn changed_from_reports_dirty_only_when_value_differs_from_pre_edit() {
        let mut session = EditSession::default();
        session.begin("before".to_string());

        assert!(!session.changed_from(&"before".to_string()));
        assert!(session.changed_from(&"after".to_string()));
    }
}
