use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
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
    decorate_control: bool,
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
            decorate_control: true,
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

    /// When enabled (default), `FormField` attempts to decorate common controls:
    /// - sets `a11y_label` on text inputs if missing
    /// - switches border/focus styling to `destructive` when an error is visible
    pub fn decorate_control(mut self, enabled: bool) -> Self {
        self.decorate_control = enabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let id = self.id;
        let a11y_label = self.label.clone();
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

        let invalid = show_error && error.is_some();

        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(label) = self.label.as_ref() {
            children.push(FormLabel::new(Arc::clone(label)).into_element(cx));
        }

        let mut control = self.control;
        if self.decorate_control {
            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let destructive = theme.color_required("destructive");
            let ring_color = Color {
                a: 0.35,
                ..destructive
            };
            let mut ring =
                decl_style::focus_ring(&theme, theme.metric_required("metric.radius.md"));
            ring.color = ring_color;

            form_decorate_control_elements(
                &mut control,
                a11y_label.as_ref(),
                invalid,
                destructive,
                ring,
            );
        }
        children.push(FormControl::new(control).into_element(cx));

        if let Some(desc) = self.description {
            children.push(FormDescription::new(desc).into_element(cx));
        }

        if show_error && let Some(err) = error {
            children.push(FormMessage::new(err).into_element(cx));
        }

        FormItem::new(children).into_element(cx)
    }
}

fn form_decorate_control_elements(
    elements: &mut [AnyElement],
    a11y_label: Option<&Arc<str>>,
    invalid: bool,
    destructive: Color,
    ring: fret_ui::element::RingStyle,
) {
    for el in elements {
        form_decorate_control_element(el, a11y_label, invalid, destructive, ring);
    }
}

fn form_decorate_control_element(
    element: &mut AnyElement,
    a11y_label: Option<&Arc<str>>,
    invalid: bool,
    destructive: Color,
    ring: fret_ui::element::RingStyle,
) {
    match &mut element.kind {
        ElementKind::Pressable(props) => {
            if props.a11y.label.is_none() {
                props.a11y.label = a11y_label.cloned();
            }
            if invalid {
                props.focus_ring = Some(ring);
            }

            for child in element.children.iter_mut() {
                form_decorate_control_element(child, a11y_label, invalid, destructive, ring);
            }
        }
        ElementKind::Container(props) => {
            if invalid
                && (props.border.left.0 > 0.0
                    || props.border.right.0 > 0.0
                    || props.border.top.0 > 0.0
                    || props.border.bottom.0 > 0.0)
            {
                props.border_color = Some(destructive);
            }

            for child in element.children.iter_mut() {
                form_decorate_control_element(child, a11y_label, invalid, destructive, ring);
            }
        }
        ElementKind::TextInput(props) => {
            if props.a11y_label.is_none() {
                props.a11y_label = a11y_label.cloned();
            }
            if invalid {
                let mut ring = ring;
                ring.corner_radii = props.chrome.corner_radii;
                props.chrome.border_color = destructive;
                props.chrome.border_color_focused = destructive;
                props.chrome.focus_ring = Some(ring);
            }
        }
        ElementKind::TextArea(props) => {
            if props.a11y_label.is_none() {
                props.a11y_label = a11y_label.cloned();
            }
            if invalid {
                let mut ring = ring;
                ring.corner_radii = props.chrome.corner_radii;
                props.chrome.border_color = destructive;
                props.chrome.focus_ring = Some(ring);
            }
        }
        _ => {
            for child in element.children.iter_mut() {
                form_decorate_control_element(child, a11y_label, invalid, destructive, ring);
            }
        }
    }
}
