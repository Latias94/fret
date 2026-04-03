use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, ElementKind};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::ColorRef;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::form_state::{FormFieldId, FormState};

use crate::form::{FormControl, FormDescription, FormItem, FormLabel, FormMessage};
use crate::form_state_model::IntoFormStateModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormErrorVisibility {
    Never,
    WhenSubmitted,
    #[default]
    WhenTouchedOrSubmitted,
    Always,
}

/// shadcn/ui `FormField`-style helper (RHF-aligned taxonomy, Fret-native state).
///
/// In upstream shadcn, `FormField` is integrated with `react-hook-form`. In Fret, this helper
/// composes a `FormItem` from:
/// - `FormLabel` (optional)
/// - `FormControl` (required)
/// - `FormDescription` (optional)
/// - `FormMessage` (optional; controlled by `FormErrorVisibility`)
#[derive(Debug)]
pub struct FormField {
    form_state: Model<FormState>,
    id: FormFieldId,
    label: Option<Arc<str>>,
    description: Option<Arc<str>>,
    control: Vec<AnyElement>,
    required: bool,
    error_visibility: FormErrorVisibility,
    decorate_control: bool,
}

impl FormField {
    pub fn new(
        form_state: impl IntoFormStateModel,
        id: impl Into<FormFieldId>,
        control: impl Into<Vec<AnyElement>>,
    ) -> Self {
        Self {
            form_state: form_state.into_form_state_model(),
            id: id.into(),
            label: None,
            description: None,
            control: control.into(),
            required: false,
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

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// When enabled (default), `FormField` attempts to decorate common controls:
    /// - sets `a11y_label` on text inputs if missing
    /// - propagates required semantics when `FormField::required(true)` is set
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
        let required = self.required;

        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(label) = self.label.as_ref() {
            let mut label = FormLabel::new(Arc::clone(label));
            if invalid {
                label = label.text_color(ColorRef::Color(
                    fret_ui::Theme::global(&*cx.app).color_token("destructive"),
                ));
            }
            children.push(label.into_element(cx));
        }

        let mut control = self.control;
        if self.decorate_control {
            let theme = fret_ui::Theme::global(&*cx.app).snapshot();
            let destructive = theme.color_token("destructive");
            let default_ring =
                decl_style::focus_ring(&theme, theme.metric_token("metric.radius.md"));
            let mut ring = default_ring;
            ring.color = crate::theme_variants::invalid_control_ring_color(&theme, destructive);
            let shadow_focus_ring_color = theme
                .color_by_key("ring/50")
                .or_else(|| theme.color_by_key("ring"))
                .unwrap_or_else(|| theme.color_token("ring"));

            form_decorate_control_elements(
                &mut control,
                a11y_label.as_ref(),
                required,
                invalid,
                destructive,
                default_ring,
                ring,
                shadow_focus_ring_color,
            );
        }
        children.push(FormControl::new(control).into_element(cx));

        if let Some(desc) = self.description {
            children.push(FormDescription::new(desc).into_element(cx));
        }

        if show_error
            && let Some(err) = error
            && !err.trim().is_empty()
        {
            children.push(FormMessage::new(err).into_element(cx));
        }

        FormItem::new(children).into_element(cx)
    }
}

fn form_decorate_control_elements(
    elements: &mut [AnyElement],
    a11y_label: Option<&Arc<str>>,
    required: bool,
    invalid: bool,
    destructive: Color,
    default_ring: fret_ui::element::RingStyle,
    ring: fret_ui::element::RingStyle,
    shadow_focus_ring_color: Color,
) {
    for el in elements {
        form_decorate_control_element(
            el,
            a11y_label,
            required,
            invalid,
            destructive,
            default_ring,
            ring,
            shadow_focus_ring_color,
        );
    }
}

fn recolor_animated_color(current: Color, source_base: Color, target_base: Color) -> Color {
    let progress = if source_base.a.abs() > f32::EPSILON {
        (current.a / source_base.a).clamp(0.0, 1.0)
    } else if current.a.abs() > f32::EPSILON {
        1.0
    } else {
        0.0
    };

    Color {
        a: (target_base.a * progress).clamp(0.0, 1.0),
        ..target_base
    }
}

fn container_shadow_looks_like_focus_ring(props: &ContainerProps) -> bool {
    let Some(shadow) = props.shadow else {
        return false;
    };

    props.layout.position == fret_ui::element::PositionStyle::Absolute
        && props.background.is_none()
        && props.border.left.0 <= 1e-6
        && props.border.right.0 <= 1e-6
        && props.border.top.0 <= 1e-6
        && props.border.bottom.0 <= 1e-6
        && shadow.secondary.is_none()
        && shadow.primary.offset_x.0.abs() <= 1e-6
        && shadow.primary.offset_y.0.abs() <= 1e-6
        && shadow.primary.blur.0.abs() <= 1e-6
        && shadow.primary.spread.0 > 0.0
}

fn form_decorate_control_element(
    element: &mut AnyElement,
    a11y_label: Option<&Arc<str>>,
    required: bool,
    invalid: bool,
    destructive: Color,
    default_ring: fret_ui::element::RingStyle,
    ring: fret_ui::element::RingStyle,
    shadow_focus_ring_color: Color,
) {
    match &mut element.kind {
        ElementKind::Pressable(props) => {
            if props.a11y.label.is_none() {
                props.a11y.label = a11y_label.cloned();
            }
            if required {
                props.a11y.required = true;
            }
            if invalid {
                props.a11y.invalid = Some(fret_core::SemanticsInvalid::True);
                if let Some(existing_ring) = props.focus_ring.as_mut() {
                    existing_ring.color =
                        recolor_animated_color(existing_ring.color, default_ring.color, ring.color);
                    match (
                        existing_ring.offset_color.as_mut(),
                        default_ring.offset_color,
                        ring.offset_color,
                    ) {
                        (Some(existing_offset), Some(default_offset), Some(target_offset)) => {
                            *existing_offset = recolor_animated_color(
                                *existing_offset,
                                default_offset,
                                target_offset,
                            );
                        }
                        (Some(existing_offset), None, Some(target_offset)) => {
                            *existing_offset = Color {
                                a: existing_offset.a,
                                ..target_offset
                            };
                        }
                        (None, _, Some(target_offset)) => {
                            existing_ring.offset_color = Some(target_offset);
                        }
                        _ => {}
                    }
                } else {
                    props.focus_ring = Some(ring);
                }
            }

            for child in element.children.iter_mut() {
                form_decorate_control_element(
                    child,
                    a11y_label,
                    required,
                    invalid,
                    destructive,
                    default_ring,
                    ring,
                    shadow_focus_ring_color,
                );
            }
        }
        ElementKind::Semantics(props) => {
            if required && props.role == fret_core::SemanticsRole::RadioGroup {
                props.required = true;
            }
            if invalid && props.role == fret_core::SemanticsRole::RadioGroup {
                props.invalid = Some(fret_core::SemanticsInvalid::True);
            }

            for child in element.children.iter_mut() {
                form_decorate_control_element(
                    child,
                    a11y_label,
                    required,
                    invalid,
                    destructive,
                    default_ring,
                    ring,
                    shadow_focus_ring_color,
                );
            }
        }
        ElementKind::Container(props) => {
            if invalid {
                if props.border.left.0 > 0.0
                    || props.border.right.0 > 0.0
                    || props.border.top.0 > 0.0
                    || props.border.bottom.0 > 0.0
                {
                    props.border_color = Some(destructive);
                }

                if props.focus_border_color.is_some() {
                    props.focus_border_color = Some(destructive);
                }

                if let Some(existing_ring) = props.focus_ring.as_mut() {
                    existing_ring.color =
                        recolor_animated_color(existing_ring.color, default_ring.color, ring.color);
                    match (
                        existing_ring.offset_color.as_mut(),
                        default_ring.offset_color,
                        ring.offset_color,
                    ) {
                        (Some(existing_offset), Some(default_offset), Some(target_offset)) => {
                            *existing_offset = recolor_animated_color(
                                *existing_offset,
                                default_offset,
                                target_offset,
                            );
                        }
                        (Some(existing_offset), None, Some(target_offset)) => {
                            *existing_offset = Color {
                                a: existing_offset.a,
                                ..target_offset
                            };
                        }
                        (None, _, Some(target_offset)) => {
                            existing_ring.offset_color = Some(target_offset);
                        }
                        _ => {}
                    }
                }

                if container_shadow_looks_like_focus_ring(props)
                    && let Some(shadow) = props.shadow.as_mut()
                {
                    shadow.primary.color = recolor_animated_color(
                        shadow.primary.color,
                        shadow_focus_ring_color,
                        ring.color,
                    );
                }
            }

            for child in element.children.iter_mut() {
                form_decorate_control_element(
                    child,
                    a11y_label,
                    required,
                    invalid,
                    destructive,
                    default_ring,
                    ring,
                    shadow_focus_ring_color,
                );
            }
        }
        ElementKind::TextInput(props) => {
            if props.a11y_label.is_none() {
                props.a11y_label = a11y_label.cloned();
            }
            if required {
                props.a11y_required = true;
            }
            if invalid {
                let mut ring = ring;
                ring.corner_radii = props.chrome.corner_radii;
                props.chrome.border_color = destructive;
                props.chrome.border_color_focused = destructive;
                props.chrome.focus_ring = Some(ring);
                props.a11y_invalid = Some(fret_core::SemanticsInvalid::True);
            }
        }
        ElementKind::TextArea(props) => {
            if props.a11y_label.is_none() {
                props.a11y_label = a11y_label.cloned();
            }
            if required {
                props.a11y_required = true;
            }
            if invalid {
                let mut ring = ring;
                ring.corner_radii = props.chrome.corner_radii;
                props.chrome.border_color = destructive;
                props.chrome.border_color_focused = destructive;
                props.chrome.focus_ring = Some(ring);
                props.a11y_invalid = Some(fret_core::SemanticsInvalid::True);
            }
        }
        _ => {
            for child in element.children.iter_mut() {
                form_decorate_control_element(
                    child,
                    a11y_label,
                    required,
                    invalid,
                    destructive,
                    default_ring,
                    ring,
                    shadow_focus_ring_color,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("form_field.rs");

    use fret_app::App;
    use fret_core::window::ColorScheme;
    use fret_core::{
        AppWindowId, NodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle, Point,
        Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
        WindowFrameClockService,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::Theme;
    use fret_ui::ThemeConfig;
    use fret_ui::element::{ContainerProps, ElementKind};
    use fret_ui::tree::UiTree;
    use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
    use fret_ui_headless::form_state::FormState;
    use std::time::Duration;
    use time::{Date, Month};

    use crate::checkbox::Checkbox;
    use crate::combobox::{Combobox, ComboboxItem};
    use crate::date_picker::DatePicker;
    use crate::date_picker_with_presets::DatePickerWithPresets;
    use crate::date_range_picker::DateRangePicker;
    use crate::input::Input;
    use crate::input_group::InputGroup;
    use crate::input_otp::InputOtp;
    use crate::native_select::{NativeSelect, NativeSelectOption};
    use crate::radio_group::{RadioGroup, RadioGroupItem};
    use crate::select::{Select, SelectItem};
    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};
    use crate::switch::Switch;
    use crate::textarea::Textarea;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(220.0)),
        )
    }

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect()
    }

    fn find_text_area_props(el: &AnyElement) -> Option<&fret_ui::element::TextAreaProps> {
        if let ElementKind::TextArea(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_text_area_props)
    }

    fn find_text_input_props(el: &AnyElement) -> Option<&fret_ui::element::TextInputProps> {
        if let ElementKind::TextInput(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_text_input_props)
    }

    fn find_container_with_focus_ring(el: &AnyElement) -> Option<&ContainerProps> {
        if let ElementKind::Container(props) = &el.kind
            && props.focus_ring.is_some()
        {
            return Some(props);
        }
        el.children.iter().find_map(find_container_with_focus_ring)
    }

    fn find_element_by_test_id<'a>(el: &'a AnyElement, test_id: &str) -> Option<&'a AnyElement> {
        let semantics_test_id = el
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id);
        let pressable_test_id = matches!(
            &el.kind,
            ElementKind::Pressable(props) if props.a11y.test_id.as_deref() == Some(test_id)
        );

        if semantics_test_id || pressable_test_id {
            return Some(el);
        }

        el.children
            .iter()
            .find_map(|child| find_element_by_test_id(child, test_id))
    }

    fn find_first_pressable(el: &AnyElement) -> Option<&fret_ui::element::PressableProps> {
        if let ElementKind::Pressable(props) = &el.kind {
            return Some(props);
        }

        el.children.iter().find_map(find_first_pressable)
    }

    fn find_pressable_chrome(el: &AnyElement) -> Option<&ContainerProps> {
        match &el.kind {
            ElementKind::Pressable(_) => el.children.first().and_then(|child| {
                if let ElementKind::Container(props) = &child.kind {
                    Some(props)
                } else {
                    None
                }
            }),
            _ => el.children.iter().find_map(find_pressable_chrome),
        }
    }

    #[test]
    fn form_field_new_keeps_a_narrow_form_state_bridge() {
        let implementation = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        let normalized = normalize_ws(implementation);
        assert!(
            normalized.contains(
                "pubfnnew(form_state:implIntoFormStateModel,id:implInto<FormFieldId>,control:implInto<Vec<AnyElement>>,)->Self{"
            ),
            "FormField::new should accept the dedicated form-state bridge"
        );
        assert!(
            !normalized.contains("pubfnnew(form_state:Model<FormState>,"),
            "FormField::new should not regress to a raw Model<FormState>-only signature"
        );
    }

    fn find_slot_border_color(el: &AnyElement, test_id: &str) -> Option<Color> {
        let slot = find_element_by_test_id(el, test_id)?;
        match &slot.kind {
            ElementKind::Container(props) => props.border_color,
            _ => None,
        }
    }

    fn find_focus_ring_shadow_color(el: &AnyElement, ring_spread: Px) -> Option<Color> {
        let mut best: Option<Color> = None;
        let mut stack = vec![el];
        while let Some(node) = stack.pop() {
            if let ElementKind::Container(props) = &node.kind
                && let Some(shadow) = props.shadow
                && (shadow.primary.offset_x.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.offset_y.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.blur.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.spread.0 - ring_spread.0).abs() <= 1e-6
            {
                best = Some(best.map_or(shadow.primary.color, |current| {
                    if current.a >= shadow.primary.color.a {
                        current
                    } else {
                        shadow.primary.color
                    }
                }));
            }

            stack.extend(node.children.iter());
        }
        best
    }

    fn node_id_by_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) -> NodeId {
        snap.nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("expected semantics node with test_id={id:?}"))
            .id
    }

    fn colors_match_rgb(actual: Color, expected: Color, eps: f32) -> bool {
        (actual.r - expected.r).abs() <= eps
            && (actual.g - expected.g).abs() <= eps
            && (actual.b - expected.b).abs() <= eps
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn form_field_invalid_textarea_uses_destructive_border_for_focused_state() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("bio");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-invalid-textarea",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [Textarea::new(model.clone()).into_element(cx)],
                )
                .label("Bio")
                .into_element(cx)
            },
        );

        let destructive = Theme::global(&app).color_token("destructive");
        let props = find_text_area_props(&el).expect("expected textarea inside form field");
        assert_eq!(props.chrome.border_color, destructive);
        assert_eq!(props.chrome.border_color_focused, destructive);
        assert_eq!(props.a11y_invalid, Some(fret_core::SemanticsInvalid::True));
        assert!(
            props.chrome.focus_ring.is_some(),
            "expected invalid textarea to receive destructive focus ring decoration"
        );
    }

    #[test]
    fn form_field_invalid_input_marks_text_input_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("name");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-invalid-input",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [Input::new(model.clone()).into_element(cx)],
                )
                .label("Name")
                .into_element(cx)
            },
        );

        let destructive = Theme::global(&app).color_token("destructive");
        let props = find_text_input_props(&el).expect("expected text input inside form field");
        assert_eq!(props.chrome.border_color, destructive);
        assert_eq!(props.chrome.border_color_focused, destructive);
        assert_eq!(props.a11y_invalid, Some(fret_core::SemanticsInvalid::True));
    }

    #[test]
    fn form_field_invalid_input_group_uses_destructive_container_focus_ring() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("email");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-invalid-input-group",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [InputGroup::new(model.clone())
                        .placeholder("name@example.com")
                        .into_element(cx)],
                )
                .label("Email")
                .into_element(cx)
            },
        );

        let theme = Theme::global(&app).snapshot();
        let destructive = theme.color_token("destructive");
        let expected_ring_color =
            crate::theme_variants::invalid_control_ring_color(&theme, destructive);
        let props =
            find_container_with_focus_ring(&el).expect("expected input group root with focus ring");
        assert_eq!(props.border_color, Some(destructive));
        let actual_ring = props
            .focus_ring
            .as_ref()
            .map(|ring| ring.color)
            .expect("expected input group focus ring");
        assert!(
            colors_match_rgb(actual_ring, expected_ring_color, 1e-6),
            "expected invalid input group ring rgb to match destructive ring; actual={actual_ring:?} expected={expected_ring_color:?}"
        );
    }

    #[test]
    fn form_field_invalid_input_otp_marks_hidden_input_invalid_and_uses_destructive_active_ring() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                color_scheme: Some(ColorScheme::Light),
                ..ThemeConfig::default()
            });
        });
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("verification_code");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let theme = Theme::global(&app).snapshot();
        let destructive = theme.color_token("destructive");
        let expected_ring_color =
            crate::theme_variants::invalid_control_ring_color(&theme, destructive);
        let ring_spread = theme
            .metric_by_key("component.ring.width")
            .unwrap_or(Px(3.0));

        fn render_capture(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            form_state: Model<FormState>,
            field_id: Arc<str>,
            model: Model<String>,
            hidden_input_invalid_out: &mut Option<fret_core::SemanticsInvalid>,
            slot_border_out: &mut Option<Color>,
            ring_color_out: &mut Option<Color>,
            ring_spread: Px,
        ) {
            let window = AppWindowId::default();
            ui.set_window(window);

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds(),
                "form-field-invalid-input-otp",
                |cx| {
                    let el = FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [InputOtp::new(model.clone())
                            .length(6)
                            .test_id_prefix("otp")
                            .into_element(cx)],
                    )
                    .label("Verification code")
                    .into_element(cx);
                    *hidden_input_invalid_out =
                        find_text_input_props(&el).and_then(|props| props.a11y_invalid);
                    *slot_border_out = find_slot_border_color(&el, "otp.slot.0");
                    *ring_color_out = find_focus_ring_shadow_color(&el, ring_spread);
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds(), 1.0);
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let mut hidden_input_invalid_out: Option<fret_core::SemanticsInvalid> = None;
        let mut slot_border_out: Option<Color> = None;
        let mut ring_color_out: Option<Color> = None;
        render_capture(
            &mut ui,
            &mut app,
            &mut services,
            form_state.clone(),
            field_id.clone(),
            model.clone(),
            &mut hidden_input_invalid_out,
            &mut slot_border_out,
            &mut ring_color_out,
            ring_spread,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = node_id_by_test_id(snap, "otp.input");
        ui.set_focus(Some(input));

        let settle = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            Duration::from_millis(150),
        ) + 2;
        for n in 0..settle {
            let tick = 2 + n;
            app.set_tick_id(TickId(tick));
            app.set_frame_id(FrameId(tick));
            render_capture(
                &mut ui,
                &mut app,
                &mut services,
                form_state.clone(),
                field_id.clone(),
                model.clone(),
                &mut hidden_input_invalid_out,
                &mut slot_border_out,
                &mut ring_color_out,
                ring_spread,
            );
        }

        assert_eq!(
            hidden_input_invalid_out,
            Some(fret_core::SemanticsInvalid::True)
        );
        assert_eq!(
            slot_border_out,
            Some(destructive),
            "expected invalid FormField to recolor OTP slot border"
        );
        let ring_color = ring_color_out.expect("expected OTP active slot ring shadow");
        assert!(
            colors_match_rgb(ring_color, expected_ring_color, 1e-4),
            "expected OTP active slot ring rgb to match invalid ring; actual={ring_color:?} expected={expected_ring_color:?}"
        );
        assert!(
            (ring_color.a - expected_ring_color.a).abs() <= 1e-4,
            "expected OTP active slot ring alpha to settle to invalid ring alpha; actual={ring_color:?} expected={expected_ring_color:?}"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp.input"))
            .expect("expected semantics node otp.input");
        assert_eq!(node.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }

    #[test]
    fn form_field_invalid_select_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("country");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-select",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Select::new(model.clone(), open.clone())
                            .items([
                                SelectItem::new("cn", "China"),
                                SelectItem::new("jp", "Japan"),
                            ])
                            .trigger_test_id("form-field-select-trigger")
                            .into_element(cx)],
                    )
                    .label("Country")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-select-trigger"))
            .expect("select trigger semantics");
        assert_eq!(trigger.role, fret_core::SemanticsRole::ComboBox);
        assert_eq!(
            trigger.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_combobox_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("framework");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-combobox",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Combobox::new(model.clone(), open.clone())
                            .test_id_prefix("form-field-combobox")
                            .items(vec![
                                ComboboxItem::new("fret", "Fret"),
                                ComboboxItem::new("gpui", "GPUI"),
                            ])
                            .into_element(cx)],
                    )
                    .label("Framework")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-combobox-trigger"))
            .expect("combobox trigger semantics");
        assert_eq!(trigger.role, fret_core::SemanticsRole::ComboBox);
        assert_eq!(
            trigger.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_radio_group_marks_group_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let field_id: Arc<str> = Arc::from("plan");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-radio-group",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [RadioGroup::new(model.clone())
                            .a11y_label("Plan")
                            .item(RadioGroupItem::new("free", "Free"))
                            .item(RadioGroupItem::new("pro", "Pro"))
                            .into_element(cx)],
                    )
                    .label("Plan")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let group = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::RadioGroup && n.label.as_deref() == Some("Plan")
            })
            .expect("radio group semantics");
        assert_eq!(group.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }

    #[test]
    fn form_field_invalid_date_picker_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);
        let field_id: Arc<str> = Arc::from("due_date");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-date-picker",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [
                            DatePicker::new(open.clone(), month.clone(), selected.clone())
                                .test_id_prefix("form-field-date-picker")
                                .into_element(cx),
                        ],
                    )
                    .label("Due date")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-date-picker-trigger"))
            .expect("date picker trigger semantics");
        assert_eq!(
            trigger.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_date_picker_uses_destructive_trigger_chrome() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);
        let field_id: Arc<str> = Arc::from("due_date");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-invalid-date-picker-chrome",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [
                        DatePicker::new(open.clone(), month.clone(), selected.clone())
                            .test_id_prefix("form-field-date-picker")
                            .into_element(cx),
                    ],
                )
                .label("Due date")
                .into_element(cx)
            },
        );

        let trigger = find_element_by_test_id(&el, "form-field-date-picker-trigger")
            .expect("date picker trigger element");
        let pressable = find_first_pressable(trigger).expect("date picker trigger pressable");
        let chrome = find_pressable_chrome(trigger).expect("date picker trigger chrome");

        let theme = Theme::global(&app).snapshot();
        let expected_border = theme.color_token("destructive");
        let mut expected_ring =
            crate::theme_variants::invalid_control_ring_color(&theme, expected_border);
        expected_ring.a = 0.0;

        assert_eq!(chrome.border_color, Some(expected_border));
        assert_eq!(
            pressable.focus_ring.expect("focus ring").color,
            expected_ring
        );
    }

    #[test]
    fn form_field_invalid_date_picker_with_presets_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);
        let field_id: Arc<str> = Arc::from("ship_date");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root =
            fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds(),
                "form-field-invalid-date-picker-with-presets",
                |cx| {
                    vec![
                        FormField::new(
                            form_state.clone(),
                            field_id.clone(),
                            [DatePickerWithPresets::new(
                                open.clone(),
                                month.clone(),
                                selected.clone(),
                            )
                            .test_id_prefix("form-field-date-picker-with-presets")
                            .into_element(cx)],
                        )
                        .label("Ship date")
                        .into_element(cx),
                    ]
                },
            );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-date-picker-with-presets-trigger"))
            .expect("date picker with presets trigger semantics");
        assert_eq!(
            trigger.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_date_range_picker_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(DateRangeSelection::default());
        let field_id: Arc<str> = Arc::from("travel_dates");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-date-range-picker",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [
                            DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                                .test_id_prefix("form-field-date-range-picker")
                                .into_element(cx),
                        ],
                    )
                    .label("Travel dates")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-date-range-picker-trigger"))
            .expect("date range picker trigger semantics");
        assert_eq!(
            trigger.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_checkbox_marks_control_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("accept_terms");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-checkbox",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Checkbox::new(model.clone())
                            .test_id("form-field-checkbox")
                            .into_element(cx)],
                    )
                    .label("Accept terms")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-checkbox"))
            .expect("checkbox semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::Checkbox);
        assert_eq!(
            control.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_checkbox_uses_destructive_chrome() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("accept_terms");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-invalid-checkbox-chrome",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [Checkbox::new(model.clone())
                        .test_id("form-field-checkbox")
                        .into_element(cx)],
                )
                .label("Accept terms")
                .into_element(cx)
            },
        );

        let control =
            find_element_by_test_id(&el, "form-field-checkbox").expect("checkbox element");
        let pressable = find_first_pressable(control).expect("checkbox pressable");
        let chrome = find_pressable_chrome(control).expect("checkbox chrome");

        let theme = Theme::global(&app).snapshot();
        let expected_border = theme.color_token("destructive");
        let mut expected_ring =
            crate::theme_variants::invalid_control_ring_color(&theme, expected_border);
        expected_ring.a = 0.0;

        assert_eq!(chrome.border_color, Some(expected_border));
        assert_eq!(
            pressable.focus_ring.expect("focus ring").color,
            expected_ring
        );
    }

    #[test]
    fn form_field_invalid_switch_marks_control_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("airplane_mode");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-switch",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Switch::new(model.clone())
                            .test_id("form-field-switch")
                            .into_element(cx)],
                    )
                    .label("Airplane mode")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-switch"))
            .expect("switch semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::Switch);
        assert_eq!(
            control.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_invalid_native_select_marks_trigger_semantics_invalid() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("country");
        let error: Arc<str> = Arc::from("Required");

        let _ = app.models_mut().update(&form_state, |st| {
            st.touch(field_id.clone());
            st.set_error(field_id.clone(), error.clone());
        });

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-invalid-native-select",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [NativeSelect::new(value.clone(), open.clone())
                            .option(NativeSelectOption::new("cn", "China"))
                            .trigger_test_id("form-field-native-select")
                            .into_element(cx)],
                    )
                    .label("Country")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-native-select"))
            .expect("native select semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::ComboBox);
        assert_eq!(
            control.flags.invalid,
            Some(fret_core::SemanticsInvalid::True)
        );
    }

    #[test]
    fn form_field_required_input_marks_text_input_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("name");

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-required-input",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [Input::new(model.clone()).into_element(cx)],
                )
                .label("Name")
                .required(true)
                .into_element(cx)
            },
        );

        let props = find_text_input_props(&el).expect("expected text input inside form field");
        assert!(props.a11y_required);
    }

    #[test]
    fn form_field_required_textarea_marks_text_area_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("bio");

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-required-textarea",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [Textarea::new(model.clone()).into_element(cx)],
                )
                .label("Bio")
                .required(true)
                .into_element(cx)
            },
        );

        let props = find_text_area_props(&el).expect("expected textarea inside form field");
        assert!(props.a11y_required);
    }

    #[test]
    fn form_field_required_input_group_marks_built_in_input_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("email");

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-required-input-group",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [InputGroup::new(model.clone())
                        .placeholder("name@example.com")
                        .into_element(cx)],
                )
                .label("Email")
                .required(true)
                .into_element(cx)
            },
        );

        let props = find_text_input_props(&el).expect("expected input group built-in text input");
        assert!(props.a11y_required);
    }

    #[test]
    fn form_field_required_input_otp_marks_hidden_input_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(String::new());
        let field_id: Arc<str> = Arc::from("verification_code");

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-field-required-input-otp",
            |cx| {
                FormField::new(
                    form_state.clone(),
                    field_id.clone(),
                    [InputOtp::new(model.clone())
                        .length(6)
                        .test_id_prefix("otp")
                        .into_element(cx)],
                )
                .label("Verification code")
                .required(true)
                .into_element(cx)
            },
        );

        let props = find_text_input_props(&el).expect("expected hidden otp input");
        assert!(props.a11y_required);
    }

    #[test]
    fn form_field_required_select_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("country");

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-select",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Select::new(model.clone(), open.clone())
                            .items([
                                SelectItem::new("cn", "China"),
                                SelectItem::new("jp", "Japan"),
                            ])
                            .trigger_test_id("form-field-required-select-trigger")
                            .into_element(cx)],
                    )
                    .label("Country")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-select-trigger"))
            .expect("select trigger semantics");
        assert_eq!(trigger.role, fret_core::SemanticsRole::ComboBox);
        assert!(trigger.flags.required);
    }

    #[test]
    fn form_field_required_combobox_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("framework");

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-combobox",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Combobox::new(model.clone(), open.clone())
                            .test_id_prefix("form-field-required-combobox")
                            .items(vec![
                                ComboboxItem::new("fret", "Fret"),
                                ComboboxItem::new("gpui", "GPUI"),
                            ])
                            .into_element(cx)],
                    )
                    .label("Framework")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-combobox-trigger"))
            .expect("combobox trigger semantics");
        assert_eq!(trigger.role, fret_core::SemanticsRole::ComboBox);
        assert!(trigger.flags.required);
    }

    #[test]
    fn form_field_required_radio_group_marks_group_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(None::<Arc<str>>);
        let field_id: Arc<str> = Arc::from("plan");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-radio-group",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [RadioGroup::new(model.clone())
                            .a11y_label("Plan")
                            .item(RadioGroupItem::new("free", "Free"))
                            .item(RadioGroupItem::new("pro", "Pro"))
                            .into_element(cx)],
                    )
                    .label("Plan")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let group = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::RadioGroup && n.label.as_deref() == Some("Plan")
            })
            .expect("radio group semantics");
        assert!(group.flags.required);
    }

    #[test]
    fn form_field_required_date_picker_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);
        let field_id: Arc<str> = Arc::from("due_date");

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-date-picker",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [
                            DatePicker::new(open.clone(), month.clone(), selected.clone())
                                .test_id_prefix("form-field-required-date-picker")
                                .into_element(cx),
                        ],
                    )
                    .label("Due date")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-date-picker-trigger"))
            .expect("date picker trigger semantics");
        assert!(trigger.flags.required);
    }

    #[test]
    fn form_field_required_date_picker_with_presets_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);
        let field_id: Arc<str> = Arc::from("ship_date");

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root =
            fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds(),
                "form-field-required-date-picker-with-presets",
                |cx| {
                    vec![
                        FormField::new(
                            form_state.clone(),
                            field_id.clone(),
                            [DatePickerWithPresets::new(
                                open.clone(),
                                month.clone(),
                                selected.clone(),
                            )
                            .test_id_prefix("form-field-required-date-picker-with-presets")
                            .into_element(cx)],
                        )
                        .label("Ship date")
                        .required(true)
                        .into_element(cx),
                    ]
                },
            );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| {
                n.test_id.as_deref() == Some("form-field-required-date-picker-with-presets-trigger")
            })
            .expect("date picker with presets trigger semantics");
        assert!(trigger.flags.required);
    }

    #[test]
    fn form_field_required_date_range_picker_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(DateRangeSelection::default());
        let field_id: Arc<str> = Arc::from("travel_dates");

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-date-range-picker",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [
                            DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                                .test_id_prefix("form-field-required-date-range-picker")
                                .into_element(cx),
                        ],
                    )
                    .label("Travel dates")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-date-range-picker-trigger"))
            .expect("date range picker trigger semantics");
        assert!(trigger.flags.required);
    }

    #[test]
    fn form_field_required_checkbox_marks_control_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("accept_terms");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-checkbox",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Checkbox::new(model.clone())
                            .test_id("form-field-required-checkbox")
                            .into_element(cx)],
                    )
                    .label("Accept terms")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-checkbox"))
            .expect("checkbox semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::Checkbox);
        assert!(control.flags.required);
    }

    #[test]
    fn form_field_required_switch_marks_control_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let model = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("airplane_mode");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-switch",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [Switch::new(model.clone())
                            .test_id("form-field-required-switch")
                            .into_element(cx)],
                    )
                    .label("Airplane mode")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-switch"))
            .expect("switch semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::Switch);
        assert!(control.flags.required);
    }

    #[test]
    fn form_field_required_native_select_marks_trigger_semantics_required() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut services = FakeServices;
        let form_state = app.models_mut().insert(FormState::default());
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let field_id: Arc<str> = Arc::from("country");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "form-field-required-native-select",
            |cx| {
                vec![
                    FormField::new(
                        form_state.clone(),
                        field_id.clone(),
                        [NativeSelect::new(value.clone(), open.clone())
                            .option(NativeSelectOption::new("cn", "China"))
                            .trigger_test_id("form-field-required-native-select")
                            .into_element(cx)],
                    )
                    .label("Country")
                    .required(true)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds());
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let control = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("form-field-required-native-select"))
            .expect("native select semantics");
        assert_eq!(control.role, fret_core::SemanticsRole::ComboBox);
        assert!(control.flags.required);
    }
}
