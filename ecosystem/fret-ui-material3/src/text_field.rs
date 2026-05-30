//! Material 3 text field (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven chrome via `md.comp.(outlined|filled)-text-field.*`.
//! - Hover/focus/error/disabled outcomes via theme tokens (best-effort).

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, NodeId, Point, Px, SemanticsRole, SvgFit, TextOverflow,
    TextStrutStyle, TextStyle, TextWrap, Transform2D,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnPressablePointerDown, PointerDownCx, PressablePointerDownResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, HoverRegionProps, Length, MainAlign,
    Overflow, PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextAreaProps,
    TextInputProps, TextProps, VisualTransformProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Invalidation, TextAreaStyle, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::typography::{self, TextIntent};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::field::{
    material_field_active_indicator_layer, material_field_text_start_inset_x,
};
use crate::foundation::field_motion::{FieldInputPhase, FieldMotionTargets, field_motion_frame};
use crate::foundation::floating_label;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interactive_size::minimum_interactive_size;
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::test_id::part_test_id;
use crate::tokens::autocomplete as autocomplete_tokens;
use crate::tokens::text_field as text_field_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TextFieldTokenNamespace {
    #[default]
    TextField,
    Autocomplete,
}

#[derive(Debug, Clone, Default)]
pub struct TextFieldStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub text_color: OverrideSlot<ColorRef>,
    pub placeholder_color: OverrideSlot<ColorRef>,
    pub caret_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
}

impl TextFieldStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn placeholder_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    pub fn caret_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.caret_color = Some(color);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.text_color.is_some() {
            self.text_color = other.text_color;
        }
        if other.placeholder_color.is_some() {
            self.placeholder_color = other.placeholder_color;
        }
        if other.caret_color.is_some() {
            self.caret_color = other.caret_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.supporting_text_color.is_some() {
            self.supporting_text_color = other.supporting_text_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
struct TextFieldPartTestIds {
    chrome: Arc<str>,
    active_indicator: Arc<str>,
    label: Arc<str>,
    supporting_text: Arc<str>,
    leading_icon: Arc<str>,
    trailing_icon: Arc<str>,
}

impl TextFieldPartTestIds {
    fn from_base(base: &Arc<str>) -> Self {
        Self {
            chrome: part_test_id(base, "chrome"),
            active_indicator: part_test_id(base, "active-indicator"),
            label: part_test_id(base, "label"),
            supporting_text: part_test_id(base, "supporting-text"),
            leading_icon: part_test_id(base, "leading-icon"),
            trailing_icon: part_test_id(base, "trailing-icon"),
        }
    }
}

fn maybe_force_strut_from_style(mut style: fret_core::TextStyle) -> fret_core::TextStyle {
    if style.line_height.is_none() && style.line_height_em.is_none() {
        return style;
    }

    style.strut_style = Some(TextStrutStyle {
        line_height: style.line_height,
        line_height_em: style.line_height_em,
        force: true,
        ..Default::default()
    });
    style
}

fn material_text_field_input_text_style<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    multiline: bool,
    stable_line_boxes: bool,
) -> TextStyle {
    let base_style = crate::foundation::context::inherited_text_style(cx).unwrap_or_else(|| {
        let theme = Theme::global(&*cx.app);
        theme
            .text_style_by_key("md.sys.typescale.body-large")
            .unwrap_or_default()
    });

    if multiline && !stable_line_boxes {
        return typography::with_intent(base_style, TextIntent::Content);
    }

    let style = typography::with_intent(base_style, TextIntent::Control);
    if multiline {
        maybe_force_strut_from_style(style)
    } else {
        style
    }
}

fn text_style_line_height(style: &TextStyle) -> Px {
    if let Some(line_height) = style.line_height {
        return line_height;
    }
    if let Some(line_height_em) = style.line_height_em {
        return Px((style.size.0 * line_height_em).max(style.size.0));
    }
    style.size
}

fn multiline_line_limit_container_height(base_height: Px, line_height: Px, lines: usize) -> Px {
    let extra_lines = lines.saturating_sub(1) as f32;
    Px(base_height.0 + line_height.0.max(0.0) * extra_lines)
}

fn text_area_style_from_text_input_style(input: fret_ui::TextInputStyle) -> TextAreaStyle {
    let mut preedit_bg_color = input.selection_color;
    preedit_bg_color.a = (preedit_bg_color.a * 0.35).clamp(0.0, 1.0);

    TextAreaStyle {
        padding_x: input.padding.left,
        padding_y: input.padding.top,
        background: input.background,
        border: input.border,
        border_color: input.border_color,
        border_color_focused: input.border_color_focused,
        focus_ring: input.focus_ring,
        corner_radii: input.corner_radii,
        text_color: input.text_color,
        placeholder_color: input.placeholder_color,
        selection_color: input.selection_color,
        caret_color: input.caret_color,
        preedit_bg_color,
        preedit_underline_color: input.preedit_underline_color,
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    variant: TextFieldVariant,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    style: TextFieldStyle,
    field_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    leading_icon: Option<IconId>,
    leading_icon_a11y_label: Option<Arc<str>>,
    leading_icon_test_id: Option<Arc<str>>,
    on_leading_icon_pointer_down: Option<OnPressablePointerDown>,
    trailing_icon: Option<IconId>,
    trailing_icon_a11y_label: Option<Arc<str>>,
    trailing_icon_test_id: Option<Arc<str>>,
    trailing_icon_rotation_progress: Option<f32>,
    on_trailing_icon_pointer_down: Option<OnPressablePointerDown>,
    disabled: bool,
    error: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    active_descendant: Option<NodeId>,
    active_descendant_element: Option<u64>,
    controls_element: Option<u64>,
    expanded: Option<bool>,
    input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    multiline: bool,
    stable_line_boxes: bool,
    multiline_min_lines: usize,
    multiline_max_lines: Option<usize>,
    multiline_min_height: Option<Px>,
    token_namespace: TextFieldTokenNamespace,
}

impl std::fmt::Debug for TextField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextField")
            .field("variant", &self.variant)
            .field("label", &self.label)
            .field("placeholder", &self.placeholder)
            .field("supporting_text", &self.supporting_text)
            .field("style", &self.style)
            .field(
                "leading_icon",
                &self.leading_icon.as_ref().map(|i| i.as_str()),
            )
            .field("disabled", &self.disabled)
            .field("error", &self.error)
            .field("multiline", &self.multiline)
            .field("stable_line_boxes", &self.stable_line_boxes)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("a11y_role", &self.a11y_role)
            .field("token_namespace", &self.token_namespace)
            .finish()
    }
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            variant: TextFieldVariant::default(),
            label: None,
            placeholder: None,
            supporting_text: None,
            style: TextFieldStyle::default(),
            field_id_out: None,
            leading_icon: None,
            leading_icon_a11y_label: None,
            leading_icon_test_id: None,
            on_leading_icon_pointer_down: None,
            trailing_icon: None,
            trailing_icon_a11y_label: None,
            trailing_icon_test_id: None,
            trailing_icon_rotation_progress: None,
            on_trailing_icon_pointer_down: None,
            disabled: false,
            error: false,
            a11y_label: None,
            test_id: None,
            a11y_role: None,
            active_descendant: None,
            active_descendant_element: None,
            controls_element: None,
            expanded: None,
            input_id_out: None,
            multiline: false,
            stable_line_boxes: true,
            multiline_min_lines: 1,
            multiline_max_lines: None,
            multiline_min_height: None,
            token_namespace: TextFieldTokenNamespace::TextField,
        }
    }

    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<String>>,
        default_value: impl Into<String>,
    ) -> Self {
        let value =
            controllable_state::use_controllable_model(cx, value, || default_value.into()).model();
        Self::new(value)
    }

    pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::new_controllable(cx, None, String::new())
    }

    pub fn value_model(&self) -> Model<String> {
        self.model.clone()
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub(crate) fn token_namespace(mut self, namespace: TextFieldTokenNamespace) -> Self {
        self.token_namespace = namespace;
        self
    }

    /// When true, uses a multiline text area surface instead of a single-line text input.
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// If true, opts into stable multiline line boxes (fixed line height + forced strut).
    ///
    /// This is intended for UI/form surfaces where baseline stability matters more than avoiding
    /// ink clipping for tall fallback glyphs.
    pub fn stable_line_boxes(mut self, stable: bool) -> Self {
        self.stable_line_boxes = stable;
        self
    }

    /// Minimum number of visible text lines in multiline mode.
    pub fn min_lines(mut self, min_lines: usize) -> Self {
        self.multiline_min_lines = min_lines.max(1);
        self
    }

    /// Maximum number of visible text lines in multiline mode.
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.multiline_max_lines = Some(max_lines.max(1));
        self
    }

    /// Visible line bounds for multiline mode.
    pub fn line_limits(mut self, min_lines: usize, max_lines: usize) -> Self {
        let min_lines = min_lines.max(1);
        self.multiline_min_lines = min_lines;
        self.multiline_max_lines = Some(max_lines.max(min_lines));
        self
    }

    /// Optional minimum height for multiline mode.
    pub fn multiline_min_height(mut self, min_height: Px) -> Self {
        self.multiline_min_height = Some(min_height);
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn style(mut self, style: TextFieldStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn leading_icon_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.leading_icon_a11y_label = Some(label.into());
        self
    }

    pub fn leading_icon_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.leading_icon_test_id = Some(id.into());
        self
    }

    pub fn on_leading_icon_pointer_down(mut self, on_pointer_down: OnPressablePointerDown) -> Self {
        self.on_leading_icon_pointer_down = Some(on_pointer_down);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn trailing_icon_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.trailing_icon_a11y_label = Some(label.into());
        self
    }

    pub fn trailing_icon_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trailing_icon_test_id = Some(id.into());
        self
    }

    pub fn trailing_icon_rotation_progress(mut self, progress: f32) -> Self {
        self.trailing_icon_rotation_progress = Some(progress);
        self
    }

    pub fn on_trailing_icon_pointer_down(
        mut self,
        on_pointer_down: OnPressablePointerDown,
    ) -> Self {
        self.on_trailing_icon_pointer_down = Some(on_pointer_down);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn a11y_role(mut self, role: SemanticsRole) -> Self {
        self.a11y_role = Some(role);
        self
    }

    pub(crate) fn active_descendant(mut self, node: Option<NodeId>) -> Self {
        self.active_descendant = node;
        self
    }

    pub(crate) fn active_descendant_element(mut self, element: Option<u64>) -> Self {
        self.active_descendant_element = element;
        self
    }

    pub(crate) fn controls_element(mut self, element: Option<u64>) -> Self {
        self.controls_element = element;
        self
    }

    pub(crate) fn expanded(mut self, expanded: Option<bool>) -> Self {
        self.expanded = expanded;
        self
    }

    pub(crate) fn input_id_out(mut self, out: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.input_id_out = Some(out);
        self
    }

    pub(crate) fn field_id_out(mut self, out: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.field_id_out = Some(out);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let TextField {
                model,
                variant,
                label,
                placeholder,
                supporting_text,
                style: style_override,
                field_id_out,
                leading_icon,
                leading_icon_a11y_label,
                leading_icon_test_id,
                on_leading_icon_pointer_down,
                trailing_icon,
                trailing_icon_a11y_label,
                trailing_icon_test_id,
                trailing_icon_rotation_progress,
                on_trailing_icon_pointer_down,
                disabled,
                error,
                a11y_label,
                test_id,
                a11y_role,
                active_descendant,
                active_descendant_element,
                controls_element,
                expanded,
                input_id_out,
                multiline,
                stable_line_boxes,
                multiline_min_lines,
                multiline_max_lines,
                multiline_min_height,
                token_namespace,
            } = self;
            let height = {
                let theme = Theme::global(&*cx.app);
                match token_namespace {
                    TextFieldTokenNamespace::TextField => {
                        text_field_tokens::container_height(theme, variant)
                    }
                    TextFieldTokenNamespace::Autocomplete => {
                        autocomplete_tokens::text_field_container_height(theme, variant)
                    }
                }
            };
            let height = if multiline {
                multiline_min_height
                    .map(|min_height| Px(height.0.max(min_height.0)))
                    .unwrap_or(height)
            } else {
                height
            };
            let input_text_style =
                material_text_field_input_text_style(cx, multiline, stable_line_boxes);
            let multiline_min_lines = multiline_min_lines.max(1);
            let multiline_max_lines = multiline_max_lines.map(|lines| lines.max(1));
            let multiline_max_lines =
                multiline_max_lines.map(|lines| lines.max(multiline_min_lines));
            let multiline_content_lines = if multiline {
                cx.read_model_ref(&model, Invalidation::Layout, |value| {
                    value.split('\n').count().max(1)
                })
                .ok()
                .unwrap_or(1)
            } else {
                1
            };
            let multiline_line_height = text_style_line_height(&input_text_style);
            let multiline_min_container_height = multiline.then(|| {
                multiline_line_limit_container_height(
                    height,
                    multiline_line_height,
                    multiline_min_lines,
                )
            });
            let multiline_max_container_height = multiline_max_lines.map(|lines| {
                let line_limit_height =
                    multiline_line_limit_container_height(height, multiline_line_height, lines);
                if let Some(min_height) = multiline_min_container_height {
                    Px(line_limit_height.0.max(min_height.0))
                } else {
                    line_limit_height
                }
            });
            let multiline_container_height = multiline.then(|| {
                let visible_lines = multiline_content_lines.max(multiline_min_lines);
                let visible_lines = multiline_max_lines
                    .map(|max_lines| visible_lines.min(max_lines))
                    .unwrap_or(visible_lines);
                multiline_line_limit_container_height(height, multiline_line_height, visible_lines)
            });
            let part_test_ids = test_id.as_ref().map(TextFieldPartTestIds::from_base);
            let chrome_test_id = part_test_ids.as_ref().map(|ids| ids.chrome.clone());
            let active_indicator_test_id = part_test_ids
                .as_ref()
                .map(|ids| ids.active_indicator.clone());
            let label_test_id = part_test_ids.as_ref().map(|ids| ids.label.clone());
            let supporting_text_test_id = part_test_ids
                .as_ref()
                .map(|ids| ids.supporting_text.clone());
            let leading_icon_test_id = leading_icon_test_id
                .or_else(|| part_test_ids.as_ref().map(|ids| ids.leading_icon.clone()));
            let trailing_icon_test_id = trailing_icon_test_id
                .or_else(|| part_test_ids.as_ref().map(|ids| ids.trailing_icon.clone()));
            let label_element_id_out = cx.slot_state(
                || Rc::new(Cell::new(None::<GlobalElementId>)),
                |id| id.clone(),
            );
            let supporting_text_element_id_out = cx.slot_state(
                || Rc::new(Cell::new(None::<GlobalElementId>)),
                |id| id.clone(),
            );

            let mut hover_layout = fret_ui::element::LayoutStyle::default();
            hover_layout.size.width = Length::Fill;
            hover_layout.overflow = Overflow::Visible;
            let hover = HoverRegionProps {
                layout: hover_layout,
            };

            let variant_for_children = variant;
            let a11y_label = a11y_label
                .or_else(|| label.clone())
                .or_else(|| placeholder.clone());

            cx.hover_region(hover, move |cx, hovered| {
                let mut focused = false;
                let mut input_id = GlobalElementId(0);
                let mut states = WidgetStates::empty();
                let mut input_bg = {
                    let theme = Theme::global(&*cx.app);
                    theme
                        .color_by_key("md.sys.color.surface")
                        .unwrap_or_else(|| theme.color_token("md.sys.color.surface"))
                };
                let mut outline_width_for_notch = Px(0.0);
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.overflow = Overflow::Visible;
                            layout
                        },
                        direction: fret_core::Axis::Vertical,
                        gap: Px(4.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Start,
                        wrap: false,
                    },
                    move |cx| {
                        let mut children: Vec<AnyElement> = Vec::new();
                        let mut float_progress = 0.0f32;
                        let mut active_indicator_el: Option<AnyElement> = None;
                        let mut leading_icon_content_size: Option<Px> = None;
                        if label.is_none() {
                            label_element_id_out.set(None);
                        }
                        if supporting_text.is_none() {
                            supporting_text_element_id_out.set(None);
                        }

                        let input = cx.named("text_input", |cx| {
                            let populated = cx
                                .read_model_ref(&model, Invalidation::Layout, |v| !v.is_empty())
                                .ok()
                                .unwrap_or(false);
                            let input_text_style = input_text_style.clone();

                            let mut container = ContainerProps::default();
                            container.layout.size.width = Length::Fill;
                            container.layout.size.height = Length::Px(height);
                            container.layout.overflow = Overflow::Clip;

                            let state_layer = (hovered && !disabled)
                                .then(|| {
                                    let theme = Theme::global(&*cx.app);
                                    text_field_tokens::hover_state_layer(
                                        theme,
                                        variant_for_children,
                                        error,
                                    )
                                })
                                .flatten()
                                .map(|(color, opacity)| {
                                    let mut out = color;
                                    out.a = (out.a * opacity).clamp(0.0, 1.0);
                                    out
                                })
                                .filter(|c| c.a > 0.0);

                            let text_input = if multiline {
                                cx.text_area_with_id_props(|cx, id| {
                                    input_id = id;
                                    focused = cx.is_focused_element(id);
                                    states =
                                        text_field_widget_states(cx, hovered, focused, disabled);

                                    let mut chrome = {
                                        let theme = Theme::global(&*cx.app);
                                        let mut chrome = match token_namespace {
                                            TextFieldTokenNamespace::TextField => {
                                                text_field_tokens::text_input_style(
                                                    theme,
                                                    variant_for_children,
                                                    focused,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                )
                                            }
                                            TextFieldTokenNamespace::Autocomplete => {
                                                autocomplete_tokens::text_input_style(
                                                    theme,
                                                    variant_for_children,
                                                    focused,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                )
                                            }
                                        };
                                        apply_text_field_input_overrides(
                                            theme,
                                            states,
                                            &style_override,
                                            &mut chrome,
                                        );
                                        chrome
                                    };

                                    let (
                                        leading_icon_hit_width,
                                        trailing_icon_hit_width,
                                        next_leading_icon_content_size,
                                    ) = {
                                        let theme = Theme::global(&*cx.app);
                                        let min_touch_target = minimum_interactive_size(theme);
                                        let leading =
                                            leading_icon.is_some().then_some(min_touch_target);
                                        let trailing =
                                            trailing_icon.is_some().then_some(min_touch_target);
                                        let leading_size =
                                            leading_icon.as_ref().map(|_| match token_namespace {
                                                TextFieldTokenNamespace::TextField => {
                                                    text_field_tokens::leading_icon_size(
                                                        theme,
                                                        variant_for_children,
                                                    )
                                                }
                                                TextFieldTokenNamespace::Autocomplete => {
                                                    autocomplete_tokens::leading_icon_size(
                                                        theme,
                                                        variant_for_children,
                                                    )
                                                }
                                            });
                                        (
                                            leading.unwrap_or(Px(0.0)),
                                            trailing.unwrap_or(Px(0.0)),
                                            leading_size,
                                        )
                                    };
                                    leading_icon_content_size = next_leading_icon_content_size;
                                    if leading_icon_hit_width.0 > 0.0 {
                                        chrome.padding.left = Px(chrome.padding.left.0.max(
                                            material_field_text_start_inset_x(
                                                leading_icon_hit_width,
                                                next_leading_icon_content_size,
                                            )
                                            .0,
                                        ));
                                    }
                                    if trailing_icon_hit_width.0 > 0.0 {
                                        chrome.padding.right = Px(chrome
                                            .padding
                                            .right
                                            .0
                                            .max(trailing_icon_hit_width.0));
                                    }

                                    let expanded_for_float = expanded.unwrap_or(false);
                                    let should_float = focused || expanded_for_float || populated;
                                    let input_phase = if focused {
                                        FieldInputPhase::Focused
                                    } else if populated {
                                        FieldInputPhase::UnfocusedNotEmpty
                                    } else {
                                        FieldInputPhase::UnfocusedEmpty
                                    };

                                    let placeholder_target_opacity: f32 = if label.is_some() {
                                        if (focused || expanded_for_float) && !populated {
                                            1.0
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        1.0
                                    };

                                    input_bg = chrome.background;
                                    outline_width_for_notch = match variant_for_children {
                                        TextFieldVariant::Outlined => chrome.border.top,
                                        TextFieldVariant::Filled => Px(0.0),
                                    };

                                    let spatial = sys_spring_in_scope(
                                        &*cx,
                                        Theme::global(&*cx.app),
                                        MotionSchemeKey::FastSpatial,
                                    );
                                    let fast_effects = sys_spring_in_scope(
                                        &*cx,
                                        Theme::global(&*cx.app),
                                        MotionSchemeKey::FastEffects,
                                    );
                                    let slow_effects = sys_spring_in_scope(
                                        &*cx,
                                        Theme::global(&*cx.app),
                                        MotionSchemeKey::SlowEffects,
                                    );
                                    let motion = field_motion_frame(
                                        cx,
                                        FieldMotionTargets {
                                            disabled,
                                            should_float,
                                            input_phase,
                                            placeholder_target_opacity,
                                            border: chrome.border,
                                            border_color: chrome.border_color,
                                            spatial,
                                            fast_effects,
                                            slow_effects,
                                        },
                                    );
                                    float_progress = motion.float_progress.clamp(0.0, 1.0);

                                    let mut container_border = motion.border;
                                    if variant_for_children == TextFieldVariant::Filled
                                        && motion.border.bottom.0 > 0.0
                                    {
                                        active_indicator_el =
                                            Some(material_field_active_indicator_layer(
                                                cx,
                                                motion.border.bottom,
                                                motion.border_color,
                                                active_indicator_test_id.clone(),
                                            ));
                                        container_border.bottom = Px(0.0);
                                    }

                                    container.background =
                                        (chrome.background.a > 0.0).then_some(chrome.background);
                                    container.corner_radii = chrome.corner_radii;
                                    container.border = container_border;
                                    container.border_color = Some(motion.border_color);
                                    if let Some(min_height) = multiline_container_height {
                                        container.layout.size.height = Length::Auto;
                                        container.layout.size.min_height =
                                            Some(Length::Px(min_height));
                                        container.layout.size.max_height =
                                            multiline_max_container_height.map(Length::Px);
                                    }

                                    chrome.background = Color::TRANSPARENT;
                                    chrome.border = Edges::all(Px(0.0));
                                    chrome.border_color = Color::TRANSPARENT;
                                    chrome.border_color_focused = Color::TRANSPARENT;

                                    chrome.placeholder_color = alpha_mul(
                                        chrome.placeholder_color,
                                        motion.placeholder_opacity.clamp(0.0, 1.0),
                                    );

                                    let mut props = TextAreaProps::new(model.clone());
                                    props.layout.size.width = Length::Fill;
                                    props.layout.size.height = Length::Fill;
                                    props.a11y_label = a11y_label.clone();
                                    props.labelled_by_element =
                                        label_element_id_out.get().map(|id| id.0);
                                    props.described_by_element =
                                        supporting_text.as_ref().and_then(|_| {
                                            supporting_text_element_id_out.get().map(|id| id.0)
                                        });
                                    props.test_id = test_id.clone();
                                    props.placeholder = placeholder.clone();
                                    props.min_height = height;
                                    if let Some(min_height) = multiline_container_height {
                                        props.layout.size.height = Length::Auto;
                                        props.min_height = min_height;
                                        props.max_height = multiline_max_container_height;
                                    }
                                    props.chrome = text_area_style_from_text_input_style(chrome);
                                    props.text_style = input_text_style;

                                    props
                                })
                            } else {
                                cx.text_input_with_id_props(|cx, id| {
                                    input_id = id;
                                    focused = cx.is_focused_element(id);
                                    states =
                                        text_field_widget_states(cx, hovered, focused, disabled);

                                    let (mut chrome, spatial, fast_effects, slow_effects) = {
                                        let theme = Theme::global(&*cx.app);
                                        let mut chrome = match token_namespace {
                                            TextFieldTokenNamespace::TextField => {
                                                text_field_tokens::text_input_style(
                                                    theme,
                                                    variant_for_children,
                                                    focused,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                )
                                            }
                                            TextFieldTokenNamespace::Autocomplete => {
                                                autocomplete_tokens::text_input_style(
                                                    theme,
                                                    variant_for_children,
                                                    focused,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                )
                                            }
                                        };
                                        apply_text_field_input_overrides(
                                            theme,
                                            states,
                                            &style_override,
                                            &mut chrome,
                                        );

                                        let spatial = sys_spring_in_scope(
                                            &*cx,
                                            theme,
                                            MotionSchemeKey::FastSpatial,
                                        );
                                        let fast_effects = sys_spring_in_scope(
                                            &*cx,
                                            theme,
                                            MotionSchemeKey::FastEffects,
                                        );
                                        let slow_effects = sys_spring_in_scope(
                                            &*cx,
                                            theme,
                                            MotionSchemeKey::SlowEffects,
                                        );

                                        (chrome, spatial, fast_effects, slow_effects)
                                    };

                                    let (
                                        leading_icon_hit_width,
                                        trailing_icon_hit_width,
                                        next_leading_icon_content_size,
                                    ) = {
                                        let theme = Theme::global(&*cx.app);
                                        let min_touch_target = minimum_interactive_size(theme);
                                        let leading =
                                            leading_icon.is_some().then_some(min_touch_target);
                                        let trailing =
                                            trailing_icon.is_some().then_some(min_touch_target);
                                        let leading_size =
                                            leading_icon.as_ref().map(|_| match token_namespace {
                                                TextFieldTokenNamespace::TextField => {
                                                    text_field_tokens::leading_icon_size(
                                                        theme,
                                                        variant_for_children,
                                                    )
                                                }
                                                TextFieldTokenNamespace::Autocomplete => {
                                                    autocomplete_tokens::leading_icon_size(
                                                        theme,
                                                        variant_for_children,
                                                    )
                                                }
                                            });
                                        (
                                            leading.unwrap_or(Px(0.0)),
                                            trailing.unwrap_or(Px(0.0)),
                                            leading_size,
                                        )
                                    };
                                    leading_icon_content_size = next_leading_icon_content_size;
                                    if leading_icon_hit_width.0 > 0.0 {
                                        chrome.padding.left = Px(chrome.padding.left.0.max(
                                            material_field_text_start_inset_x(
                                                leading_icon_hit_width,
                                                next_leading_icon_content_size,
                                            )
                                            .0,
                                        ));
                                    }
                                    if trailing_icon_hit_width.0 > 0.0 {
                                        chrome.padding.right = Px(chrome
                                            .padding
                                            .right
                                            .0
                                            .max(trailing_icon_hit_width.0));
                                    }

                                    let expanded_for_float = expanded.unwrap_or(false);
                                    let should_float = focused || expanded_for_float || populated;
                                    let input_phase = if focused {
                                        FieldInputPhase::Focused
                                    } else if populated {
                                        FieldInputPhase::UnfocusedNotEmpty
                                    } else {
                                        FieldInputPhase::UnfocusedEmpty
                                    };

                                    let placeholder_target_opacity = if label.is_some() {
                                        if (focused || expanded_for_float) && !populated {
                                            1.0
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        1.0
                                    };

                                    let motion = field_motion_frame(
                                        cx,
                                        FieldMotionTargets {
                                            disabled,
                                            should_float,
                                            input_phase,
                                            placeholder_target_opacity,
                                            border: chrome.border,
                                            border_color: chrome.border_color,
                                            spatial,
                                            fast_effects,
                                            slow_effects,
                                        },
                                    );
                                    float_progress = motion.float_progress.clamp(0.0, 1.0);

                                    input_bg = chrome.background;
                                    outline_width_for_notch = match variant_for_children {
                                        TextFieldVariant::Outlined => motion.border.top,
                                        TextFieldVariant::Filled => Px(0.0),
                                    };

                                    let mut container_border = motion.border;
                                    if variant_for_children == TextFieldVariant::Filled
                                        && motion.border.bottom.0 > 0.0
                                    {
                                        active_indicator_el =
                                            Some(material_field_active_indicator_layer(
                                                cx,
                                                motion.border.bottom,
                                                motion.border_color,
                                                active_indicator_test_id.clone(),
                                            ));
                                        container_border.bottom = Px(0.0);
                                    }

                                    container.background =
                                        (chrome.background.a > 0.0).then_some(chrome.background);
                                    container.corner_radii = chrome.corner_radii;
                                    container.border = container_border;
                                    container.border_color = Some(motion.border_color);

                                    chrome.background = Color::TRANSPARENT;
                                    chrome.border = Edges::all(Px(0.0));
                                    chrome.border_color = Color::TRANSPARENT;
                                    chrome.border_color_focused = Color::TRANSPARENT;

                                    chrome.placeholder_color = alpha_mul(
                                        chrome.placeholder_color,
                                        motion.placeholder_opacity.clamp(0.0, 1.0),
                                    );

                                    let mut props = TextInputProps::new(model.clone());
                                    props.layout.size.width = Length::Fill;
                                    props.layout.size.height = Length::Fill;
                                    props.a11y_label = a11y_label.clone();
                                    props.a11y_role =
                                        Some(a11y_role.unwrap_or(SemanticsRole::TextField));
                                    props.labelled_by_element =
                                        label_element_id_out.get().map(|id| id.0);
                                    props.described_by_element =
                                        supporting_text.as_ref().and_then(|_| {
                                            supporting_text_element_id_out.get().map(|id| id.0)
                                        });
                                    props.test_id = test_id.clone();
                                    props.placeholder = placeholder.clone();
                                    props.active_descendant = active_descendant;
                                    props.active_descendant_element = active_descendant_element;
                                    props.controls_element = controls_element;
                                    props.expanded = expanded;
                                    props.chrome = chrome;
                                    props.text_style = input_text_style;

                                    props
                                })
                            };
                            if let Some(out) = input_id_out.as_ref() {
                                out.set(Some(input_id));
                            }

                            // Keep subtree shape stable across hover transitions (ADR 0166).
                            // We always include the overlay node, but only paint when `state_layer`
                            // is present.
                            let overlay = {
                                let mut overlay_layout = fret_ui::element::LayoutStyle::default();
                                overlay_layout.position = fret_ui::element::PositionStyle::Absolute;
                                overlay_layout.inset.top = Some(Px(0.0)).into();
                                overlay_layout.inset.right = Some(Px(0.0)).into();
                                overlay_layout.inset.bottom = Some(Px(0.0)).into();
                                overlay_layout.inset.left = Some(Px(0.0)).into();

                                let mut overlay = ContainerProps::default();
                                overlay.layout = overlay_layout;
                                overlay.background = state_layer;
                                overlay.corner_radii = container.corner_radii;
                                cx.container(overlay, |_cx| Vec::new())
                            };

                            let leading_icon_el = leading_icon.map(|icon| {
                                let (hit_width, size, color, opacity) = {
                                    let theme = Theme::global(&*cx.app);
                                    let hit_width = minimum_interactive_size(theme);
                                    let size = match token_namespace {
                                        TextFieldTokenNamespace::TextField => {
                                            text_field_tokens::leading_icon_size(
                                                theme,
                                                variant_for_children,
                                            )
                                        }
                                        TextFieldTokenNamespace::Autocomplete => {
                                            autocomplete_tokens::leading_icon_size(
                                                theme,
                                                variant_for_children,
                                            )
                                        }
                                    };
                                    let (color, opacity) = match token_namespace {
                                        TextFieldTokenNamespace::TextField => {
                                            text_field_tokens::leading_icon_color(
                                                theme,
                                                variant_for_children,
                                                hovered,
                                                disabled,
                                                error,
                                                focused,
                                            )
                                        }
                                        TextFieldTokenNamespace::Autocomplete => {
                                            autocomplete_tokens::leading_icon_color(
                                                theme,
                                                variant_for_children,
                                                hovered,
                                                disabled,
                                                error,
                                                focused,
                                            )
                                        }
                                    };
                                    (hit_width, size, color, opacity)
                                };

                                let svg = svg_source_for_icon(cx, &icon);
                                let mut icon_props = SvgIconProps::new(svg);
                                icon_props.fit = SvgFit::Contain;
                                icon_props.color = color;
                                icon_props.opacity = opacity;
                                icon_props.layout.size.width = Length::Px(size);
                                icon_props.layout.size.height = Length::Px(size);
                                let icon_el = cx.svg_icon_props(icon_props);

                                let icon_test_id = leading_icon_test_id.clone();
                                let icon_a11y_label = leading_icon_a11y_label.clone();

                                let input_id_for_focus = input_id;
                                let handler = on_leading_icon_pointer_down.clone();
                                let enabled = !disabled;

                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout.inset.top = Some(Px(0.0)).into();
                                layout.inset.left = Some(Px(0.0)).into();
                                layout.inset.bottom = Some(Px(0.0)).into();
                                layout.size.width = Length::Px(hit_width);
                                layout.size.height = Length::Fill;

                                let has_action = handler.is_some() || icon_a11y_label.is_some();
                                let role = has_action.then_some(SemanticsRole::Button);

                                cx.pressable(
                                    PressableProps {
                                        layout,
                                        enabled,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            role,
                                            label: icon_a11y_label,
                                            test_id: icon_test_id,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, _state| {
                                        if enabled {
                                            let handler = handler.clone();
                                            cx.pressable_on_pointer_down(Arc::new(
                                                move |host, action_cx, down: PointerDownCx| {
                                                    host.request_focus(input_id_for_focus);
                                                    if let Some(ref h) = handler {
                                                        return h(host, action_cx, down);
                                                    }
                                                    PressablePointerDownResult::Continue
                                                },
                                            ));
                                        }

                                        let mut row = FlexProps::default();
                                        row.direction = Axis::Horizontal;
                                        row.justify = MainAlign::Center;
                                        row.align = CrossAlign::Center;
                                        row.layout.size.width = Length::Fill;
                                        row.layout.size.height = Length::Fill;
                                        vec![cx.flex(row, move |_cx| vec![icon_el])]
                                    },
                                )
                            });

                            let trailing_icon_el = trailing_icon.map(|icon| {
                                let (
                                    hit_width,
                                    size,
                                    color,
                                    opacity,
                                    hover_opacity,
                                    pressed_opacity,
                                    config,
                                ) = {
                                    let theme = Theme::global(&*cx.app);
                                    let hit_width = minimum_interactive_size(theme);
                                    let size = match token_namespace {
                                        TextFieldTokenNamespace::TextField => {
                                            text_field_tokens::trailing_icon_size(
                                                theme,
                                                variant_for_children,
                                            )
                                        }
                                        TextFieldTokenNamespace::Autocomplete => {
                                            autocomplete_tokens::trailing_icon_size(
                                                theme,
                                                variant_for_children,
                                            )
                                        }
                                    };
                                    let (color, opacity) = match token_namespace {
                                        TextFieldTokenNamespace::TextField => {
                                            text_field_tokens::trailing_icon_color(
                                                theme,
                                                variant_for_children,
                                                hovered,
                                                disabled,
                                                error,
                                                focused,
                                            )
                                        }
                                        TextFieldTokenNamespace::Autocomplete => {
                                            autocomplete_tokens::trailing_icon_color(
                                                theme,
                                                variant_for_children,
                                                hovered,
                                                disabled,
                                                error,
                                                focused,
                                            )
                                        }
                                    };

                                    let hover_opacity = theme
                                        .number_by_key("md.sys.state.hover.state-layer-opacity")
                                        .unwrap_or(0.08);
                                    let pressed_opacity = theme
                                        .number_by_key("md.sys.state.pressed.state-layer-opacity")
                                        .unwrap_or(0.1);
                                    let config = material_pressable_indication_config(theme, None);

                                    (
                                        hit_width,
                                        size,
                                        color,
                                        opacity,
                                        hover_opacity,
                                        pressed_opacity,
                                        config,
                                    )
                                };

                                let svg = svg_source_for_icon(cx, &icon);
                                let mut icon_props = SvgIconProps::new(svg);
                                icon_props.fit = SvgFit::Contain;
                                icon_props.color = color;
                                icon_props.opacity = opacity;
                                icon_props.layout.size.width = Length::Px(size);
                                icon_props.layout.size.height = Length::Px(size);
                                let icon_el = cx.svg_icon_props(icon_props);

                                let icon_el =
                                    if let Some(progress) = trailing_icon_rotation_progress {
                                        let degrees = 180.0 * progress.clamp(0.0, 1.0);
                                        let mut layout = fret_ui::element::LayoutStyle::default();
                                        layout.size.width = Length::Px(size);
                                        layout.size.height = Length::Px(size);
                                        cx.visual_transform_props(
                                            VisualTransformProps {
                                                layout,
                                                transform: Transform2D::rotation_about_degrees(
                                                    degrees,
                                                    Point::new(Px(size.0 * 0.5), Px(size.0 * 0.5)),
                                                ),
                                            },
                                            move |_cx| vec![icon_el],
                                        )
                                    } else {
                                        icon_el
                                    };

                                let icon_test_id = trailing_icon_test_id.clone();
                                let icon_a11y_label = trailing_icon_a11y_label.clone();

                                let input_id_for_focus = input_id;
                                let handler = on_trailing_icon_pointer_down.clone();
                                let enabled = !disabled;
                                let ripple_base_opacity = pressed_opacity;
                                let corner_radii = Corners::all(Px(hit_width.0 * 0.5));
                                let state_layer_color = alpha_mul(color, opacity);

                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout.inset.top = Some(Px(0.0)).into();
                                layout.inset.right = Some(Px(0.0)).into();
                                layout.inset.bottom = Some(Px(0.0)).into();
                                layout.size.width = Length::Px(hit_width);
                                layout.size.height = Length::Fill;

                                cx.pressable(
                                    PressableProps {
                                        layout,
                                        enabled,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::Button),
                                            label: icon_a11y_label,
                                            test_id: icon_test_id,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, state| {
                                        if enabled {
                                            let handler = handler.clone();
                                            cx.pressable_on_pointer_down(Arc::new(
                                                move |host, action_cx, down: PointerDownCx| {
                                                    host.request_focus(input_id_for_focus);
                                                    if let Some(ref h) = handler {
                                                        return h(host, action_cx, down);
                                                    }
                                                    PressablePointerDownResult::Continue
                                                },
                                            ));
                                        }

                                        let pressable_id = cx.root_id();
                                        let now_frame = cx.frame_id.0;

                                        let mut props = PointerRegionProps::default();
                                        props.enabled = enabled;
                                        props.layout.size.width = Length::Fill;
                                        props.layout.size.height = Length::Fill;

                                        vec![cx.pointer_region(props, move |cx| {
                                            cx.pointer_region_on_pointer_down(Arc::new(
                                                |_host, _cx, _down| false,
                                            ));

                                            let pressed = enabled && state.pressed;
                                            let hovered = enabled && state.hovered;
                                            let state_layer_target = if pressed {
                                                pressed_opacity
                                            } else if hovered {
                                                hover_opacity
                                            } else {
                                                0.0
                                            };

                                            let overlay = material_ink_layer_for_pressable(
                                                cx,
                                                pressable_id,
                                                now_frame,
                                                corner_radii,
                                                RippleClip::Bounded,
                                                state_layer_color,
                                                pressed,
                                                state_layer_target,
                                                ripple_base_opacity,
                                                config,
                                                false,
                                            );

                                            let mut row = FlexProps::default();
                                            row.direction = Axis::Horizontal;
                                            row.justify = MainAlign::Center;
                                            row.align = CrossAlign::Center;
                                            row.layout.size.width = Length::Fill;
                                            row.layout.size.height = Length::Fill;
                                            vec![overlay, cx.flex(row, move |_cx| vec![icon_el])]
                                        })]
                                    },
                                )
                            });

                            let mut chrome = cx.container(container, move |cx| {
                                if let Some(out) = field_id_out.as_ref() {
                                    out.set(Some(cx.root_id()));
                                }
                                let mut out = vec![overlay, text_input];
                                if let Some(icon) = leading_icon_el {
                                    out.push(icon);
                                }
                                if let Some(icon) = trailing_icon_el {
                                    out.push(icon);
                                }
                                if let Some(indicator) = active_indicator_el {
                                    out.push(indicator);
                                }
                                out
                            });
                            if let Some(test_id) = chrome_test_id.clone() {
                                chrome = chrome.test_id(test_id);
                            }
                            chrome
                        });

                        children.push(input);

                        if let Some(label) = label.as_ref() {
                            children.push(text_field_label(
                                cx,
                                variant_for_children,
                                label.clone(),
                                float_progress,
                                states,
                                &style_override,
                                hovered,
                                disabled,
                                error,
                                focused,
                                input_id,
                                input_bg,
                                outline_width_for_notch,
                                label_test_id.clone(),
                                leading_icon_content_size,
                                label_element_id_out.clone(),
                            ));
                        }

                        if let Some(text) = supporting_text.as_ref() {
                            children.push(text_field_supporting_text(
                                cx,
                                variant_for_children,
                                text.clone(),
                                states,
                                &style_override,
                                hovered,
                                disabled,
                                error,
                                focused,
                                supporting_text_test_id.clone(),
                                leading_icon_content_size,
                                supporting_text_element_id_out.clone(),
                            ));
                        }

                        children
                    },
                )]
            })
        })
    }
}

fn text_field_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: TextFieldVariant,
    text: Arc<str>,
    progress: f32,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    input_id: GlobalElementId,
    input_bg: Color,
    outline_width: Px,
    test_id: Option<Arc<str>>,
    leading_icon_size: Option<Px>,
    label_element_id_out: Rc<Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = floating_label::material_floating_label_text_style(theme, progress)
            .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
            .map(|style| typography::with_intent(style, TextIntent::Control));

        let color = resolve_override_slot_with(
            style_override.label_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || text_field_tokens::label_color(theme, variant, hovered, disabled, error, focused),
        );

        (style, color)
    };

    let (x, y) = floating_label::material_floating_label_offsets(progress);
    let x = material_field_text_start_inset_x(x, leading_icon_size);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y).into();
    layout.inset.left = Some(x).into();
    layout.inset.right = Some(Px(16.0)).into();
    layout.overflow = Overflow::Visible;

    let floated = floating_label::is_floated(progress);

    let mut patch = ContainerProps::default();
    if variant == TextFieldVariant::Outlined {
        let patch_padding_x = Px(4.0);
        let patch_padding_y = Px((outline_width.0 + 1.0).max(0.0));
        patch.padding = (if floated {
            Edges {
                top: patch_padding_y,
                right: patch_padding_x,
                bottom: patch_padding_y,
                left: patch_padding_x,
            }
        } else {
            Edges::all(Px(0.0))
        })
        .into();
        patch.background = floated.then_some(input_bg);
    }

    let mut label = cx.pointer_region(
        PointerRegionProps {
            layout,
            enabled: !disabled,
            ..Default::default()
        },
        move |cx| {
            let input_for_focus = input_id;
            cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                host.request_focus(input_for_focus);
                true
            }));

            vec![cx.container(patch, move |cx| {
                vec![cx.text_props(TextProps {
                    layout: fret_ui::element::LayoutStyle::default(),
                    text: text.clone(),
                    style,
                    color: Some(color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                })]
            })]
        },
    );
    if let Some(test_id) = test_id {
        label = label.test_id(test_id);
    }
    label_element_id_out.set(Some(label.id));
    label
}

fn text_field_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: TextFieldVariant,
    text: Arc<str>,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    test_id: Option<Arc<str>>,
    leading_icon_size: Option<Px>,
    supporting_text_element_id_out: Rc<Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = theme
            .text_style_by_key("md.sys.typescale.body-small")
            .map(|style| typography::with_intent(style, TextIntent::Content));
        let color = resolve_override_slot_with(
            style_override.supporting_text_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || {
                text_field_tokens::supporting_text_color(
                    theme, variant, hovered, disabled, error, focused,
                )
            },
        );

        (style, color)
    };

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.margin.left = fret_ui::element::MarginEdge::Px(material_field_text_start_inset_x(
        Px(16.0),
        leading_icon_size,
    ));
    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(16.0));

    let mut supporting_text = cx.text_props(TextProps {
        layout,
        text,
        style,
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: Default::default(),
    });
    if let Some(test_id) = test_id {
        supporting_text = supporting_text.test_id(test_id);
    }
    supporting_text_element_id_out.set(Some(supporting_text.id));
    supporting_text
}

fn text_field_widget_states<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hovered: bool,
    focused: bool,
    disabled: bool,
) -> WidgetStates {
    let mut states = WidgetStates::empty();
    states.set(WidgetState::Disabled, disabled);
    states.set(WidgetState::Hovered, hovered && !disabled);
    states.set(WidgetState::Focused, focused && !disabled);
    states.set(
        WidgetState::FocusVisible,
        focused && !disabled && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    states
}

fn alpha_mul(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn apply_text_field_input_overrides(
    theme: &Theme,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    chrome: &mut fret_ui::TextInputStyle,
) {
    if let Some(background) = style_override
        .container_background
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.background = background.resolve(theme);
    }

    if let Some(outline) = style_override
        .outline_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        let outline = outline.resolve(theme);
        chrome.border_color = outline;
        chrome.border_color_focused = outline;
    }

    if let Some(text_color) = style_override
        .text_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.text_color = text_color.resolve(theme);
    }

    if let Some(placeholder) = style_override
        .placeholder_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.placeholder_color = placeholder.resolve(theme);
    }

    if let Some(caret_color) = style_override
        .caret_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.caret_color = caret_color.resolve(theme);
    }
}

#[cfg(test)]
mod controllable_state_tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::elements::with_element_cx;
    use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    #[test]
    fn text_field_new_controllable_uses_controlled_value_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(String::from("alpha"));

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-text-field-controlled",
            |cx| {
                let field = TextField::new_controllable(cx, Some(controlled.clone()), "default");
                assert_eq!(field.value_model(), controlled);
            },
        );
    }

    #[test]
    fn text_field_new_controllable_applies_default_value() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(&mut app, window, bounds(), "m3-text-field-default", |cx| {
            let field = TextField::new_controllable(cx, None, "hello");
            let value = cx
                .watch_model(&field.value_model())
                .layout()
                .cloned()
                .unwrap_or_default();
            assert_eq!(value, "hello");
        });
    }

    #[test]
    fn text_field_uncontrolled_multiple_instances_do_not_share_models() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-text-field-uncontrolled",
            |cx| {
                let a = TextField::uncontrolled(cx);
                let b = TextField::uncontrolled(cx);
                assert_ne!(a.value_model(), b.value_model());
            },
        );
    }
}
