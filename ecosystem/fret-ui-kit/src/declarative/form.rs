use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_runtime::{Model, ModelHost, ModelId, ModelStore};
use fret_ui::action::UiActionHost;

use crate::headless::form_state::{FormFieldId, FormState, FormValidateMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormRevalidateMode {
    #[default]
    Never,
    OnChange,
}

#[derive(Debug, Clone)]
pub struct FormRegistryOptions {
    pub touch_on_change: bool,
    pub revalidate_mode: FormRevalidateMode,
}

impl Default for FormRegistryOptions {
    fn default() -> Self {
        Self {
            touch_on_change: false,
            revalidate_mode: FormRevalidateMode::OnChange,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldEval {
    pub dirty: bool,
    pub error: Option<Arc<str>>,
}

type FieldEvalFn = Arc<dyn Fn(&ModelStore, bool) -> FieldEval + 'static>;

#[derive(Clone)]
struct RegisteredField {
    id: FormFieldId,
    eval: FieldEvalFn,
}

/// Narrow interop bridge for form registries that track app-owned values in `Model<T>`.
///
/// This intentionally stays specific to the form registry surface rather than widening into a
/// crate-wide `IntoModel<T>` story.
pub trait IntoFormValueModel<T> {
    fn into_form_value_model(self) -> Model<T>;
}

impl<T> IntoFormValueModel<T> for Model<T> {
    fn into_form_value_model(self) -> Model<T> {
        self
    }
}

impl<T> IntoFormValueModel<T> for &Model<T> {
    fn into_form_value_model(self) -> Model<T> {
        self.clone()
    }
}

/// A lightweight, opt-in registry that connects app-owned `Model<T>` values to a `FormState`.
///
/// Notes:
/// - This is intentionally app-owned state (store it in your window state/driver), not a model.
/// - Validation remains value-driven: callers provide `validate(&T) -> Option<Arc<str>>`.
/// - The registry only reads values from the `ModelStore`; it never owns field values.
#[derive(Clone, Default)]
pub struct FormRegistry {
    options: FormRegistryOptions,
    fields: Vec<RegisteredField>,
    by_model_id: HashMap<ModelId, usize>,
}

impl std::fmt::Debug for FormRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormRegistry")
            .field("options", &self.options)
            .field("fields_len", &self.fields.len())
            .finish()
    }
}

impl FormRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: FormRegistryOptions) -> Self {
        self.options = options;
        self
    }

    pub fn field_ids(&self) -> impl Iterator<Item = &FormFieldId> {
        self.fields.iter().map(|f| &f.id)
    }

    pub fn register_into_form_state<H: ModelHost>(
        &self,
        host: &mut H,
        form_state: &Model<FormState>,
    ) {
        let fields: Vec<FormFieldId> = self.field_ids().cloned().collect();
        let _ = host.models_mut().update(form_state, move |st| {
            for id in fields.iter().cloned() {
                st.register_field(id);
            }
        });
    }

    pub fn register_field<T>(
        &mut self,
        id: impl Into<FormFieldId>,
        model: impl IntoFormValueModel<T>,
        initial: T,
        validate: impl Fn(&T) -> Option<Arc<str>> + 'static,
    ) where
        T: Clone + PartialEq + 'static,
    {
        let id: FormFieldId = id.into();
        let model = model.into_form_value_model();
        let model_id = model.id();
        let eval: FieldEvalFn = Arc::new(move |store, force_validate| {
            let current = store
                .read(&model, |v| v.clone())
                .unwrap_or_else(|_| initial.clone());
            let dirty = current != initial;
            let error = force_validate.then(|| validate(&current)).flatten();
            FieldEval { dirty, error }
        });

        let idx = self.fields.len();
        self.fields.push(RegisteredField { id, eval });
        self.by_model_id.insert(model_id, idx);
    }

    pub fn handle_model_changes<H: ModelHost>(
        &self,
        host: &mut H,
        form_state: &Model<FormState>,
        changed: &[ModelId],
    ) {
        if self.fields.is_empty() || changed.is_empty() {
            return;
        }

        let (validate_mode, submit_count, error_fields) = host
            .models()
            .read(form_state, |st| {
                (
                    st.validate_mode,
                    st.submit_count,
                    st.errors.keys().cloned().collect::<HashSet<_>>(),
                )
            })
            .unwrap_or((FormValidateMode::default(), 0, HashSet::new()));

        let store = host.models();
        let mut updates: Vec<(FormFieldId, FieldEval)> = Vec::new();
        for &id in changed {
            let Some(&idx) = self.by_model_id.get(&id) else {
                continue;
            };
            let field = &self.fields[idx];
            let has_error = error_fields.contains(&field.id);
            let should_validate = match validate_mode {
                FormValidateMode::OnChange | FormValidateMode::All => true,
                FormValidateMode::OnSubmit => {
                    has_error
                        || (submit_count > 0
                            && matches!(self.options.revalidate_mode, FormRevalidateMode::OnChange))
                }
            };
            let eval = (field.eval)(store, should_validate);
            updates.push((Arc::clone(&field.id), eval));
        }

        if updates.is_empty() {
            return;
        }

        let touch_on_change = self.options.touch_on_change;
        let _ = host.models_mut().update(form_state, move |st| {
            for (id, eval) in updates.iter() {
                st.set_dirty(Arc::clone(id), eval.dirty);
                if touch_on_change && eval.dirty {
                    st.touch(Arc::clone(id));
                }
                st.set_error_opt(Arc::clone(id), eval.error.clone());
            }
        });
    }

    pub fn submit<H: ModelHost>(&self, host: &mut H, form_state: &Model<FormState>) -> bool {
        let store = host.models();
        let mut evals: Vec<(FormFieldId, FieldEval)> = self
            .fields
            .iter()
            .map(|f| (Arc::clone(&f.id), (f.eval)(store, true)))
            .collect();

        let _ = host.models_mut().update(form_state, move |st| {
            st.begin_submit();
            st.touch_all_registered();
            for (id, eval) in evals.drain(..) {
                st.set_dirty(Arc::clone(&id), eval.dirty);
                st.set_error_opt(id, eval.error);
            }
            st.end_submit();
        });

        host.models()
            .read(form_state, |st| st.is_valid())
            .unwrap_or(false)
    }

    /// Object-safe form submission helper for action hooks.
    ///
    /// `fret_ui::action` callbacks receive a `&mut dyn UiActionHost` (object-safe by design),
    /// while `submit()` is generic over `ModelHost`. This helper bridges that gap without
    /// exposing `FormRegistry` internals to call sites.
    pub fn submit_action_host(
        &self,
        host: &mut dyn UiActionHost,
        form_state: &Model<FormState>,
    ) -> bool {
        let store = host.models_mut();
        let mut evals: Vec<(FormFieldId, FieldEval)> = self
            .fields
            .iter()
            .map(|f| (Arc::clone(&f.id), (f.eval)(&*store, true)))
            .collect();

        let _ = store.update(form_state, move |st| {
            st.begin_submit();
            st.touch_all_registered();
            for (id, eval) in evals.drain(..) {
                st.set_dirty(Arc::clone(&id), eval.dirty);
                st.set_error_opt(id, eval.error);
            }
            st.end_submit();
        });

        store.read(form_state, |st| st.is_valid()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("form.rs");

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect()
    }

    #[test]
    fn form_registry_register_field_keeps_a_narrow_model_bridge() {
        let implementation = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        let normalized = normalize_ws(implementation);

        assert!(
            normalized.contains(
                "pubtraitIntoFormValueModel<T>{fninto_form_value_model(self)->Model<T>;}"
            ),
            "form registry should keep a dedicated narrow bridge trait instead of a broad generic model conversion story"
        );
        assert!(
            normalized.contains(
                "pubfnregister_field<T>(&mutself,id:implInto<FormFieldId>,model:implIntoFormValueModel<T>,initial:T,validate:implFn(&T)->Option<Arc<str>>+'static,)whereT:Clone+PartialEq+'static,"
            ),
            "register_field should accept the dedicated form-value bridge"
        );
        assert!(
            !normalized.contains(
                "pubfnregister_field<T>(&mutself,id:implInto<FormFieldId>,model:Model<T>,"
            ),
            "register_field should not regress to a raw Model<T>-only signature"
        );
    }
}
