use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub type FormFieldId = Arc<str>;

/// Headless form state (field meta + errors) intended for shadcn-style composition.
///
/// This type is intentionally value-agnostic:
/// - field values typically live in app-owned `Model<T>`s (ADR 0031)
/// - the form tracks lifecycle metadata (dirty/touched/submitting) and validation outcomes
#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub submit_count: u64,
    pub is_submitting: bool,
    pub dirty_fields: HashSet<FormFieldId>,
    pub touched_fields: HashSet<FormFieldId>,
    pub errors: HashMap<FormFieldId, Arc<str>>,
}

impl FormState {
    pub fn is_dirty(&self) -> bool {
        !self.dirty_fields.is_empty()
    }

    pub fn is_touched(&self) -> bool {
        !self.touched_fields.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_error(&self, field: &str) -> bool {
        self.errors.keys().any(|k| k.as_ref() == field)
    }

    pub fn error_for(&self, field: &str) -> Option<&Arc<str>> {
        self.errors
            .iter()
            .find_map(|(k, v)| if k.as_ref() == field { Some(v) } else { None })
    }

    pub fn touch(&mut self, field: impl Into<FormFieldId>) {
        self.touched_fields.insert(field.into());
    }

    pub fn set_dirty(&mut self, field: impl Into<FormFieldId>, dirty: bool) {
        let field = field.into();
        if dirty {
            self.dirty_fields.insert(field);
        } else {
            self.dirty_fields.remove(&field);
        }
    }

    pub fn set_error(&mut self, field: impl Into<FormFieldId>, message: impl Into<Arc<str>>) {
        self.errors.insert(field.into(), message.into());
    }

    pub fn clear_error(&mut self, field: &str) {
        if let Some(key) = self.errors.keys().find(|k| k.as_ref() == field).cloned() {
            self.errors.remove(&key);
        }
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn begin_submit(&mut self) {
        self.is_submitting = true;
    }

    pub fn end_submit(&mut self) {
        self.is_submitting = false;
        self.submit_count = self.submit_count.saturating_add(1);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_state_tracks_dirty_touched_and_errors() {
        let mut st = FormState::default();
        assert!(st.is_valid());
        assert!(!st.is_dirty());
        assert!(!st.is_touched());

        st.touch("name");
        st.set_dirty("name", true);
        st.set_error("name", Arc::from("Required"));

        assert!(st.is_touched());
        assert!(st.is_dirty());
        assert!(!st.is_valid());
        assert!(st.has_error("name"));
        assert_eq!(st.error_for("name").map(|s| s.as_ref()), Some("Required"));

        st.clear_error("name");
        assert!(st.is_valid());
    }
}
