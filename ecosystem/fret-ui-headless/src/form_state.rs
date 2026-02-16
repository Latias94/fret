use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub type FormFieldId = Arc<str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormValidateMode {
    /// Validate only when explicitly requested (e.g. submit).
    #[default]
    OnSubmit,
    /// Validate whenever a field's value changes.
    OnChange,
    /// Validate on submit and on change.
    All,
}

/// Headless form state (field meta + errors) intended for shadcn-style composition.
///
/// This type is intentionally value-agnostic:
/// - field values typically live in app-owned `Model<T>`s (ADR 0031)
/// - the form tracks lifecycle metadata (dirty/touched/submitting) and validation outcomes
#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub validate_mode: FormValidateMode,
    pub submit_count: u64,
    pub is_submitting: bool,
    pub registered_fields: Vec<FormFieldId>,
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

    pub fn is_registered(&self, field: &str) -> bool {
        self.registered_fields.iter().any(|k| k.as_ref() == field)
    }

    pub fn register_field(&mut self, field: impl Into<FormFieldId>) {
        let field = field.into();
        if self
            .registered_fields
            .iter()
            .any(|k| k.as_ref() == field.as_ref())
        {
            return;
        }
        self.registered_fields.push(field);
    }

    pub fn unregister_field(&mut self, field: &str) {
        if let Some(idx) = self
            .registered_fields
            .iter()
            .position(|k| k.as_ref() == field)
        {
            let removed = self.registered_fields.remove(idx);
            self.dirty_fields.remove(&removed);
            self.touched_fields.remove(&removed);
            self.errors.remove(&removed);
        }
    }

    pub fn touch(&mut self, field: impl Into<FormFieldId>) {
        self.touched_fields.insert(field.into());
    }

    pub fn touch_all_registered(&mut self) {
        for field in self.registered_fields.iter().cloned() {
            self.touched_fields.insert(field);
        }
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

    pub fn set_error_opt(&mut self, field: impl Into<FormFieldId>, message: Option<Arc<str>>) {
        let field = field.into();
        match message {
            Some(message) => {
                self.errors.insert(field, message);
            }
            None => {
                self.errors.remove(&field);
            }
        }
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

    pub fn validate_field(
        &mut self,
        field: impl Into<FormFieldId>,
        validate: impl FnOnce() -> Option<Arc<str>>,
    ) -> bool {
        let field = field.into();
        let error = validate();
        self.set_error_opt(field, error);
        self.is_valid()
    }

    pub fn validate_registered_fields(
        &mut self,
        mut validate: impl FnMut(&FormFieldId) -> Option<Arc<str>>,
    ) -> bool {
        let fields: Vec<FormFieldId> = self.registered_fields.to_vec();
        for field in fields.iter() {
            let error = validate(field);
            self.set_error_opt(Arc::clone(field), error);
        }
        self.is_valid()
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

    #[test]
    fn registered_fields_drive_bulk_validation() {
        let mut st = FormState::default();
        st.register_field("name");
        st.register_field("email");

        st.validate_registered_fields(|id| match id.as_ref() {
            "name" => Some(Arc::from("Required")),
            "email" => None,
            _ => None,
        });

        assert!(!st.is_valid());
        assert_eq!(st.error_for("name").map(|v| v.as_ref()), Some("Required"));
        assert!(st.error_for("email").is_none());

        st.unregister_field("name");
        assert!(st.is_valid());
    }
}
