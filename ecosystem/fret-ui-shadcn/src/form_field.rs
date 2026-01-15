use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::headless::form_state::{FormFieldId, FormState};

use crate::form::{FormControl, FormDescription, FormItem, FormLabel, FormMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormErrorVisibility {
    Never,
    WhenSubmitted,
    WhenTouchedOrSubmitted,
    Always,
}

impl Default for FormErrorVisibility {
    fn default() -> Self {
        Self::WhenTouchedOrSubmitted
    }
}

/// shadcn/ui `FormField`-style helper (RHF-aligned taxonomy, Fret-native state).
///
/// In upstream shadcn, `FormField` is integrated with `react-hook-form`. In Fret, this helper
/// composes a `FormItem` from:
/// - `FormLabel` (optional)
/// - `FormControl` (required)
/// - `FormDescription` (optional)
/// - `FormMessage` (optional; controlled by `FormErrorVisibility`)
#[derive(Debug, Clone)]
pub struct FormField {
    form_state: Model<FormState>,
    id: FormFieldId,
    label: Option<Arc<str>>,
    description: Option<Arc<str>>,
    control: Vec<AnyElement>,
    error_visibility: FormErrorVisibility,
}

impl FormField {
    pub fn new(
        form_state: Model<FormState>,
        id: impl Into<FormFieldId>,
        control: impl Into<Vec<AnyElement>>,
    ) -> Self {
        Self {
            form_state,
            id: id.into(),
            label: None,
            description: None,
            control: control.into(),
            error_visibility: FormErrorVisibility::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error_visibility(mut self, visibility: FormErrorVisibility) -> Self {
        self.error_visibility = visibility;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let id = self.id;
        let (submit_count, touched, error) = cx
            .watch_model(&self.form_state)
            .layout()
            .read_ref(|st| {
                (
                    st.submit_count,
                    st.touched_fields.contains(&id),
                    st.errors.get(&id).cloned(),
                )
            })
            .ok()
            .unwrap_or((0, false, None));

        let show_error = match self.error_visibility {
            FormErrorVisibility::Never => false,
            FormErrorVisibility::WhenSubmitted => submit_count > 0,
            FormErrorVisibility::WhenTouchedOrSubmitted => submit_count > 0 || touched,
            FormErrorVisibility::Always => true,
        };

        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(label) = self.label {
            children.push(FormLabel::new(label).into_element(cx));
        }

        children.push(FormControl::new(self.control).into_element(cx));

        if let Some(desc) = self.description {
            children.push(FormDescription::new(desc).into_element(cx));
        }

        if show_error && let Some(err) = error {
            children.push(FormMessage::new(err).into_element(cx));
        }

        FormItem::new(children).into_element(cx)
    }
}
