use fret_core::{Axis, Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    PositionStyle, SemanticsDecoration, ShadowLayerStyle, ShadowStyle, SizeStyle, TextInputProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, TextInputStyle, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Size as ComponentSize,
    Space, ui,
};
use std::sync::Arc;
use std::time::Duration;

use crate::overlay_motion;
use crate::text_value_model::IntoTextValueModel;

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default transition timing function: cubic-bezier(0.4, 0, 0.2, 1).
    // (Often described as `ease-in-out`-ish.)
    fret_ui_headless::easing::SHADCN_EASE.sample(t)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputOtpPattern {
    Digits,
    DigitsAndChars,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputOtpSeparatorMode {
    /// Legacy behavior: when `group_size` yields multiple groups, render a separator between every
    /// pair of adjacent groups.
    AutoBetweenGroups,
    /// Render separators only after the group indices listed here.
    ExplicitAfterGroups(Vec<usize>),
}

fn otp_gap(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.input_otp.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn otp_separator_color(theme: &ThemeSnapshot) -> Color {
    theme.color_token("muted-foreground")
}

fn otp_active_ring_width(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.ring.width")
        .unwrap_or(Px(3.0))
}

fn otp_active_ring_color(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("ring/50")
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("ring"))
}

fn otp_invalid_ring_color(theme: &ThemeSnapshot) -> Color {
    let border_color = theme.color_token("destructive");
    crate::theme_variants::invalid_control_ring_color(theme, border_color)
}

fn otp_slot_border_color(
    theme: &ThemeSnapshot,
    border_color: Color,
    border_color_focused: Color,
    is_active: bool,
    aria_invalid: bool,
) -> Color {
    if aria_invalid {
        return theme.color_token("destructive");
    }

    if is_active {
        border_color_focused
    } else {
        border_color
    }
}

fn otp_slot_ring_color(theme: &ThemeSnapshot, aria_invalid: bool) -> Color {
    if aria_invalid {
        otp_invalid_ring_color(theme)
    } else {
        otp_active_ring_color(theme)
    }
}

fn sanitize_otp(input: &str, length: usize, pattern: InputOtpPattern) -> String {
    let mut out = String::new();
    out.reserve(length.min(input.len()));
    for ch in input.chars() {
        if out.chars().count() >= length {
            break;
        }
        if ch.is_whitespace() {
            continue;
        }
        if ch.is_control() {
            continue;
        }
        let allowed = match pattern {
            InputOtpPattern::Digits => ch.is_ascii_digit(),
            InputOtpPattern::DigitsAndChars => ch.is_ascii_alphanumeric(),
        };
        if !allowed {
            continue;
        }
        out.push(ch);
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputOtpSlotCornerMode {
    #[default]
    Merge,
    All,
}

#[derive(Clone)]
pub struct InputOtp {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    labelled_by_element: Option<GlobalElementId>,
    control_id: Option<ControlId>,
    length: usize,
    pattern: InputOtpPattern,
    group_size: Option<usize>,
    explicit_groups: Option<Vec<InputOtpGroup>>,
    separator_mode: InputOtpSeparatorMode,
    aria_invalid: bool,
    required: bool,
    disabled: bool,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id_prefix: Option<Arc<str>>,
    container_gap_override: Option<Px>,
    slot_gap_override: Option<Px>,
    slot_size_override: Option<(Px, Px)>,
    slot_text_px_override: Option<Px>,
    slot_line_height_px_override: Option<Px>,
    slot_corner_mode: InputOtpSlotCornerMode,
}

impl std::fmt::Debug for InputOtp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputOtp")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("labelled_by_element", &self.labelled_by_element)
            .field("control_id", &self.control_id)
            .field("length", &self.length)
            .field("pattern", &self.pattern)
            .field("group_size", &self.group_size)
            .field(
                "explicit_groups",
                &self.explicit_groups.as_ref().map(|g| g.len()),
            )
            .field("separator_mode", &self.separator_mode)
            .field("aria_invalid", &self.aria_invalid)
            .field("required", &self.required)
            .field("disabled", &self.disabled)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field(
                "test_id_prefix",
                &self.test_id_prefix.as_ref().map(|s| s.as_ref()),
            )
            .field("container_gap_override", &self.container_gap_override)
            .field("slot_gap_override", &self.slot_gap_override)
            .field("slot_size_override", &self.slot_size_override)
            .field("slot_text_px_override", &self.slot_text_px_override)
            .field(
                "slot_line_height_px_override",
                &self.slot_line_height_px_override,
            )
            .field("slot_corner_mode", &self.slot_corner_mode)
            .finish()
    }
}

impl InputOtp {
    pub fn new(model: impl IntoTextValueModel) -> Self {
        Self {
            model: model.into_text_value_model(),
            a11y_label: None,
            labelled_by_element: None,
            control_id: None,
            length: 6,
            pattern: InputOtpPattern::Digits,
            group_size: None,
            explicit_groups: None,
            separator_mode: InputOtpSeparatorMode::AutoBetweenGroups,
            aria_invalid: false,
            required: false,
            disabled: false,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            test_id_prefix: None,
            container_gap_override: None,
            slot_gap_override: None,
            slot_size_override: None,
            slot_text_px_override: None,
            slot_line_height_px_override: None,
            slot_corner_mode: InputOtpSlotCornerMode::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn labelled_by_element(mut self, label: GlobalElementId) -> Self {
        self.labelled_by_element = Some(label);
        self
    }

    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(1);
        self
    }

    pub fn numeric_only(mut self, numeric_only: bool) -> Self {
        self.pattern = if numeric_only {
            InputOtpPattern::Digits
        } else {
            InputOtpPattern::DigitsAndChars
        };
        self
    }

    pub fn pattern(mut self, pattern: InputOtpPattern) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn group_size(mut self, group_size: Option<usize>) -> Self {
        self.group_size = group_size.and_then(|v| if v >= 1 { Some(v) } else { None });
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Apply the upstream `disabled` interaction + chrome outcome.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn container_gap_px(mut self, gap: Px) -> Self {
        self.container_gap_override = Some(gap);
        self
    }

    pub fn slot_gap_px(mut self, gap: Px) -> Self {
        self.slot_gap_override = Some(gap);
        self
    }

    pub fn slot_size_px(mut self, w: Px, h: Px) -> Self {
        self.slot_size_override = Some((w, h));
        self
    }

    pub fn slot_text_px(mut self, px: Px) -> Self {
        self.slot_text_px_override = Some(px);
        self
    }

    pub fn slot_line_height_px(mut self, px: Px) -> Self {
        self.slot_line_height_px_override = Some(px);
        self
    }

    pub fn slot_corner_mode(mut self, mode: InputOtpSlotCornerMode) -> Self {
        self.slot_corner_mode = mode;
        self
    }

    /// Render the OTP input using shadcn/ui v4 part-based composition.
    ///
    /// This is a compatibility adapter that maps upstream-like `InputOTPGroup` / `InputOTPSlot` /
    /// `InputOTPSeparator` compositions onto Fret's single `InputOtp` recipe.
    ///
    /// Notes:
    /// - Slot indices are treated as the source of truth.
    /// - The inferred length becomes `max(self.length, max_slot_index + 1)` to reduce copy/paste
    ///   friction when porting examples.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
        parts: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<InputOtpPart>,
    ) -> AnyElement {
        let parsed = parse_input_otp_parts(parts(cx));
        if let Some(inferred_length) = parsed.inferred_length {
            self.length = self.length.max(inferred_length);
        }

        if !parsed.groups.is_empty() {
            self.explicit_groups = Some(parsed.groups);
            self.separator_mode =
                InputOtpSeparatorMode::ExplicitAfterGroups(parsed.separators_after_groups);
        }

        self.into_element(cx)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let length = self.length;
        let mut value = cx.watch_model(&self.model).cloned().unwrap_or_default();
        let sanitized = sanitize_otp(&value, length, self.pattern);
        if sanitized != value {
            let next = sanitized.clone();
            let _ = cx.app.models_mut().update(&self.model, |v| *v = next);
            value = sanitized;
        }

        let chars: Vec<char> = value.chars().collect();
        let has_invalid_part_slots = self.explicit_groups.as_ref().is_some_and(|groups| {
            groups
                .iter()
                .flat_map(|group| group.slots.iter())
                .any(|slot| slot.aria_invalid)
        });
        let any_aria_invalid = self.aria_invalid || has_invalid_part_slots;
        let resolved = resolve_input_chrome(
            Theme::global(&*cx.app),
            self.size,
            &self.chrome,
            InputTokenKeys::none(),
        );

        let default_slot_w = Px(resolved.min_height.0.max(0.0));
        let default_slot_h = Px(resolved.min_height.0.max(0.0));

        let container_gap = self
            .container_gap_override
            .unwrap_or_else(|| otp_gap(&theme));

        let font_line_height = theme.metric_token("font.line_height");

        let root_layout = decl_style::layout_style(&theme, self.layout.relative());
        let separator_color = otp_separator_color(&theme);

        let el = cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out: Vec<AnyElement> = Vec::new();

                let input_test_id: Option<Arc<str>> = self
                    .test_id_prefix
                    .as_ref()
                    .map(|prefix| Arc::from(format!("{prefix}.input")));

                let groups: Vec<InputOtpGroup> =
                    if let Some(explicit_groups) = self.explicit_groups.as_ref() {
                        explicit_groups.clone()
                    } else {
                        let group_size = self.group_size.unwrap_or(length).max(1);
                        let mut out: Vec<InputOtpGroup> = Vec::new();
                        let mut start = 0;
                        while start < length {
                            let end = (start + group_size).min(length);
                            out.push(InputOtpGroup::new((start..end).map(InputOtpSlot::new)));
                            start = end;
                        }
                        out
                    };

                let mut chrome = TextInputStyle::from_theme(theme.clone());
                chrome.padding = Edges::all(Px(0.0));
                chrome.corner_radii = Corners::all(Px(0.0));
                chrome.border = Edges::all(Px(0.0));
                chrome.background = Color::TRANSPARENT;
                chrome.border_color = Color::TRANSPARENT;
                chrome.border_color_focused = Color::TRANSPARENT;
                chrome.text_color = Color::TRANSPARENT;
                chrome.selection_color = Color::TRANSPARENT;
                chrome.caret_color = Color::TRANSPARENT;
                chrome.preedit_color = Color::TRANSPARENT;
                chrome.preedit_underline_color = Color::TRANSPARENT;

                let has_a11y_label = self.a11y_label.is_some();
                let control_id = self.control_id.clone();
                let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));

                let mut input = TextInputProps::new(self.model);
                input.chrome = chrome;
                let default_slot_line_height = self
                    .slot_line_height_px_override
                    .unwrap_or(font_line_height);
                let default_slot_text_px = self.slot_text_px_override.unwrap_or(resolved.text_px);
                let mut input_text_style = typography::fixed_line_box_style(
                    FontId::ui(),
                    default_slot_text_px,
                    default_slot_line_height,
                );
                input_text_style.weight = FontWeight::MEDIUM;
                input.text_style = input_text_style;
                input.test_id = input_test_id.clone();
                input.a11y_label = self.a11y_label.clone();
                input.a11y_invalid = any_aria_invalid.then_some(fret_core::SemanticsInvalid::True);
                input.a11y_required = self.required;
                input.enabled = !self.disabled;
                input.focusable = !self.disabled;
                input.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(Px(0.0)).into(),
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                    },
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    overflow: fret_ui::element::Overflow::Clip,
                    ..Default::default()
                };
                let mut input_el = cx.text_input(input);

                let input_id = input_el.id;
                if let (Some(control_id), Some(control_registry)) =
                    (control_id.clone(), control_registry.clone())
                {
                    let entry = ControlEntry {
                        element: input_id,
                        enabled: !self.disabled,
                        action: ControlAction::FocusOnly,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }

                let labelled_by_element = if self.labelled_by_element.is_some() {
                    self.labelled_by_element
                } else if has_a11y_label {
                    None
                } else if let (Some(control_id), Some(control_registry)) =
                    (control_id.as_ref(), control_registry.as_ref())
                {
                    cx.app
                        .models()
                        .read(control_registry, |reg| {
                            reg.label_for(cx.window, control_id).map(|l| l.element)
                        })
                        .ok()
                        .flatten()
                } else {
                    None
                };

                let described_by_element = if let (Some(control_id), Some(control_registry)) =
                    (control_id.as_ref(), control_registry.as_ref())
                {
                    cx.app
                        .models()
                        .read(control_registry, |reg| {
                            reg.described_by_for(cx.window, control_id)
                        })
                        .ok()
                        .flatten()
                } else {
                    None
                };

                if labelled_by_element.is_some() || described_by_element.is_some() {
                    let mut decoration = SemanticsDecoration::default();
                    if let Some(label) = labelled_by_element {
                        decoration = decoration.labelled_by_element(label.0);
                    }
                    if let Some(desc) = described_by_element {
                        decoration = decoration.described_by_element(desc.0);
                    }
                    input_el = input_el.attach_semantics(decoration);
                }
                let focused = cx.is_focused_element(input_id) && !self.disabled;
                let active_slot_idx = focused.then(|| chars.len().min(length.saturating_sub(1)));
                let fake_caret_idx = (focused && chars.len() < length).then_some(chars.len());

                let blink = motion::drive_loop_progress_keyed(
                    cx,
                    ("shadcn.input_otp.caret_blink", cx.root_id()),
                    focused && fake_caret_idx.is_some(),
                    Duration::from_millis(1000),
                );
                let caret_blink_visible =
                    focused && fake_caret_idx.is_some() && blink.progress < 0.5;

                let mut slot_ids: Vec<Option<fret_ui::elements::GlobalElementId>> =
                    vec![None; length];
                let mut slot_corners: Vec<Option<Corners>> = vec![None; length];
                let mut slot_invalids: Vec<bool> = vec![self.aria_invalid; length];
                let mut pieces: Vec<AnyElement> = Vec::new();
                for (group_idx, group) in groups.iter().enumerate() {
                    let slot_gap = group
                        .slot_gap_override
                        .or(self.slot_gap_override)
                        .unwrap_or(Px(0.0));
                    let slot_gap_is_nonzero = slot_gap.0.abs() >= 0.5;
                    let (slot_w, slot_h) = group
                        .slot_size_override
                        .or(self.slot_size_override)
                        .unwrap_or((default_slot_w, default_slot_h));
                    let slot_line_height = group
                        .slot_line_height_px_override
                        .or(self.slot_line_height_px_override)
                        .unwrap_or(font_line_height);
                    let slot_text_px = group
                        .slot_text_px_override
                        .or(self.slot_text_px_override)
                        .unwrap_or(resolved.text_px);
                    let mut slot_text_style = typography::fixed_line_box_style(
                        FontId::ui(),
                        slot_text_px,
                        slot_line_height,
                    );
                    slot_text_style.weight = FontWeight::MEDIUM;

                    let group_slots = &group.slots;
                    let mut slots: Vec<AnyElement> = Vec::new();
                    for (slot_pos, slot) in group_slots.iter().enumerate() {
                        let idx = slot.index;
                        if idx >= length {
                            continue;
                        }

                        let is_first = slot_pos == 0;
                        let is_last = slot_pos + 1 == group_slots.len();
                        let is_active = active_slot_idx == Some(idx);
                        let has_fake_caret = fake_caret_idx == Some(idx);

                        let ch = chars.get(idx).copied();
                        let text: Arc<str> = ch
                            .map(|c| Arc::from(c.to_string()))
                            .unwrap_or_else(|| Arc::from(""));

                        let bg = resolved.background;
                        let border = resolved.border_width;
                        let slot_invalid = self.aria_invalid || slot.aria_invalid;
                        let border_color = otp_slot_border_color(
                            &theme,
                            resolved.border_color,
                            resolved.border_color_focused,
                            is_active,
                            slot_invalid,
                        );
                        let radius = resolved.radius;
                        let fg = resolved.text_color;

                        let all_corners = slot_gap_is_nonzero
                            || matches!(
                                group.slot_corner_mode.unwrap_or(self.slot_corner_mode),
                                InputOtpSlotCornerMode::All
                            );
                        let corner_radii = if all_corners {
                            Corners::all(radius)
                        } else {
                            Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                bottom_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_last { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                            }
                        };

                        let slot_border = if slot_gap_is_nonzero {
                            Edges::all(border)
                        } else {
                            Edges {
                                top: border,
                                right: border,
                                bottom: border,
                                left: if is_first { border } else { Px(0.0) },
                            }
                        };

                        let slot_text_style_for_slot = slot_text_style.clone();
                        let slot_test_id: Option<Arc<str>> = self
                            .test_id_prefix
                            .as_ref()
                            .map(|prefix| Arc::from(format!("{prefix}.slot.{idx}")));

                        let mut slot_el = cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Px(slot_w),
                                        height: Length::Px(slot_h),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                padding: Edges::all(Px(0.0)).into(),
                                background: Some(bg),
                                shadow: Some(decl_style::shadow_xs(&theme, radius)),
                                border: slot_border,
                                border_color: Some(border_color),
                                corner_radii,
                                ..Default::default()
                            },
                            move |cx| {
                                let mut inner: Vec<AnyElement> = Vec::new();

                                inner.push(cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Fill,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        direction: Axis::Horizontal,
                                        gap: Px(0.0).into(),
                                        padding: Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::Center,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        let style = slot_text_style_for_slot.clone();
                                        let mut label = ui::label(text)
                                            .text_size_px(style.size)
                                            .font_weight(style.weight)
                                            .text_color(ColorRef::Color(fg))
                                            .nowrap();
                                        if let Some(line_height) = style.line_height {
                                            label = label
                                                .line_height_px(line_height)
                                                .line_height_policy(
                                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                );
                                        }
                                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                                            label = label.letter_spacing_em(letter_spacing_em);
                                        }
                                        vec![label.into_element(cx)]
                                    },
                                ));

                                if has_fake_caret && caret_blink_visible {
                                    let caret_color = fg;
                                    let caret_h = Px(16.0).min(Px((slot_h.0 - 8.0).max(0.0)));
                                    inner.push(cx.container(
                                        ContainerProps {
                                            layout: LayoutStyle {
                                                position: PositionStyle::Absolute,
                                                inset: InsetStyle {
                                                    left: Some(Px(0.0)).into(),
                                                    top: Some(Px(0.0)).into(),
                                                    right: Some(Px(0.0)).into(),
                                                    bottom: Some(Px(0.0)).into(),
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            vec![cx.flex(
                                                FlexProps {
                                                    layout: LayoutStyle {
                                                        size: SizeStyle {
                                                            width: Length::Fill,
                                                            height: Length::Fill,
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    direction: Axis::Horizontal,
                                                    gap: Px(0.0).into(),
                                                    padding: Edges::all(Px(0.0)).into(),
                                                    justify: MainAlign::Center,
                                                    align: CrossAlign::Center,
                                                    wrap: false,
                                                },
                                                move |cx| {
                                                    vec![cx.container(
                                                        ContainerProps {
                                                            layout: LayoutStyle {
                                                                size: SizeStyle {
                                                                    width: Length::Px(Px(1.0)),
                                                                    height: Length::Px(caret_h),
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            background: Some(caret_color),
                                                            ..Default::default()
                                                        },
                                                        |_cx| Vec::<AnyElement>::new(),
                                                    )]
                                                },
                                            )]
                                        },
                                    ));
                                }

                                inner
                            },
                        );

                        // Upstream shadcn: `transition-all` on slot chrome, so border should ease
                        // instead of snapping when active slot changes.
                        let duration = overlay_motion::shadcn_motion_duration_150(cx);
                        let border_motion = motion::drive_tween_color_for_element(
                            cx,
                            slot_el.id,
                            "input_otp.slot.border",
                            border_color,
                            duration,
                            tailwind_transition_ease_in_out,
                        );
                        if let fret_ui::element::ElementKind::Container(props) = &mut slot_el.kind {
                            props.border_color = Some(border_motion.value);
                        }

                        slot_ids[idx] = Some(slot_el.id);
                        slot_corners[idx] = Some(corner_radii);
                        slot_invalids[idx] = slot_invalid;

                        let mut decoration = SemanticsDecoration::default().selected(is_active);
                        if self.disabled {
                            decoration = decoration.disabled(true);
                        }
                        if let Some(test_id) = slot_test_id {
                            decoration = decoration.test_id(test_id);
                        }
                        slot_el = slot_el.attach_semantics(decoration);

                        slots.push(slot_el);
                    }

                    pieces.push(cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: Axis::Horizontal,
                            gap: slot_gap.into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| slots,
                    ));

                    let show_separator = match &self.separator_mode {
                        InputOtpSeparatorMode::AutoBetweenGroups => group_idx + 1 < groups.len(),
                        InputOtpSeparatorMode::ExplicitAfterGroups(indices) => {
                            indices.contains(&group_idx)
                        }
                    };

                    if show_separator {
                        pieces.push(
                            cx.flex(
                                FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(24.0)),
                                            height: Length::Px(Px(24.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    direction: Axis::Horizontal,
                                    gap: Px(0.0).into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |cx| {
                                    vec![decl_icon::icon_with(
                                        cx,
                                        ids::ui::MINUS,
                                        Some(Px(24.0)),
                                        Some(ColorRef::Color(separator_color)),
                                    )]
                                },
                            )
                            .attach_semantics(
                                SemanticsDecoration::default().role(SemanticsRole::Separator),
                            ),
                        );
                    }
                }

                out.push(cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: Axis::Horizontal,
                        gap: container_gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| pieces,
                ));

                out.push(input_el);

                #[derive(Debug, Clone, Copy, Default)]
                struct ActiveSlotRingGeometry {
                    left: Option<Px>,
                    top: Option<Px>,
                    width: Option<Px>,
                    height: Option<Px>,
                    corners: Option<Corners>,
                }

                // Upstream shadcn: `transition-all` includes the ring (box-shadow), so the ring
                // should ease in/out instead of snapping. Keep it mounted while animating out.
                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let ring_alpha = motion::drive_tween_f32_for_element(
                    cx,
                    input_id,
                    "input_otp.slot.ring.alpha",
                    if focused && active_slot_idx.is_some() {
                        1.0
                    } else {
                        0.0
                    },
                    duration,
                    tailwind_transition_ease_in_out,
                );
                let wants_ring = ring_alpha.animating || ring_alpha.value > 0.0;

                let active_slot_geom = (focused && wants_ring)
                    .then_some(())
                    .and(active_slot_idx)
                    .and_then(|active_idx| {
                        let root_bounds = cx.last_bounds_for_element(cx.root_id())?;
                        let slot_bounds = slot_ids
                            .get(active_idx)
                            .copied()
                            .flatten()
                            .and_then(|id| cx.last_bounds_for_element(id))?;
                        let corners = slot_corners
                            .get(active_idx)
                            .copied()
                            .flatten()
                            .unwrap_or_else(|| Corners::all(resolved.radius));
                        let left = Px((slot_bounds.origin.x.0 - root_bounds.origin.x.0).max(0.0));
                        let top = Px((slot_bounds.origin.y.0 - root_bounds.origin.y.0).max(0.0));
                        Some(ActiveSlotRingGeometry {
                            left: Some(left),
                            top: Some(top),
                            width: Some(slot_bounds.size.width),
                            height: Some(slot_bounds.size.height),
                            corners: Some(corners),
                        })
                    });

                let geom = cx.state_for(input_id, ActiveSlotRingGeometry::default, |st| {
                    if let Some(active_slot_geom) = active_slot_geom {
                        *st = active_slot_geom;
                    }
                    *st
                });

                let active_slot_invalid = active_slot_idx
                    .and_then(|idx| slot_invalids.get(idx).copied())
                    .unwrap_or(self.aria_invalid);

                if wants_ring
                    && let (Some(left), Some(top), Some(width), Some(height), Some(corners)) =
                        (geom.left, geom.top, geom.width, geom.height, geom.corners)
                {
                    let ring_w = otp_active_ring_width(&theme);
                    let mut ring_color = otp_slot_ring_color(&theme, active_slot_invalid);
                    ring_color.a = (ring_color.a * ring_alpha.value).clamp(0.0, 1.0);
                    out.push(cx.hit_test_gate(false, move |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(left).into(),
                                        top: Some(top).into(),
                                        right: None.into(),
                                        bottom: None.into(),
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(width),
                                        height: Length::Px(height),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                shadow: Some(ShadowStyle {
                                    primary: ShadowLayerStyle {
                                        color: ring_color,
                                        offset_x: Px(0.0),
                                        offset_y: Px(0.0),
                                        blur: Px(0.0),
                                        spread: ring_w,
                                    },
                                    secondary: None,
                                    corner_radii: corners,
                                }),
                                ..Default::default()
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        )]
                    }));
                }

                out
            },
        );

        if self.disabled {
            cx.opacity(0.5, move |_cx| vec![el])
        } else {
            el
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedInputOtpParts {
    groups: Vec<InputOtpGroup>,
    separators_after_groups: Vec<usize>,
    inferred_length: Option<usize>,
}

fn parse_input_otp_parts(parts: Vec<InputOtpPart>) -> ParsedInputOtpParts {
    let mut groups: Vec<InputOtpGroup> = Vec::new();
    let mut current_group: Vec<InputOtpSlot> = Vec::new();
    let mut separators_after_groups: Vec<usize> = Vec::new();
    let mut max_index: Option<usize> = None;

    for part in parts {
        match part {
            InputOtpPart::Group(group) => {
                if !current_group.is_empty() {
                    groups.push(InputOtpGroup::new(std::mem::take(&mut current_group)));
                }

                if !group.slots.is_empty() {
                    for slot in &group.slots {
                        max_index = Some(max_index.map_or(slot.index, |m| m.max(slot.index)));
                    }
                    groups.push(group);
                }
            }
            InputOtpPart::Slot(slot) => {
                max_index = Some(max_index.map_or(slot.index, |m| m.max(slot.index)));
                current_group.push(slot);
            }
            InputOtpPart::Separator(_) => {
                if !current_group.is_empty() {
                    groups.push(InputOtpGroup::new(std::mem::take(&mut current_group)));
                }

                if let Some(idx) = groups.len().checked_sub(1)
                    && !separators_after_groups.contains(&idx)
                {
                    separators_after_groups.push(idx);
                }
            }
        }
    }

    if !current_group.is_empty() {
        groups.push(InputOtpGroup::new(current_group));
    }

    ParsedInputOtpParts {
        groups,
        separators_after_groups,
        inferred_length: max_index.map(|idx| idx.saturating_add(1)),
    }
}

/// shadcn/ui `InputOTPGroup` (v4).
#[derive(Debug, Clone)]
pub struct InputOtpGroup {
    slots: Vec<InputOtpSlot>,
    slot_gap_override: Option<Px>,
    slot_size_override: Option<(Px, Px)>,
    slot_text_px_override: Option<Px>,
    slot_line_height_px_override: Option<Px>,
    slot_corner_mode: Option<InputOtpSlotCornerMode>,
}

impl InputOtpGroup {
    pub fn new(slots: impl IntoIterator<Item = InputOtpSlot>) -> Self {
        Self {
            slots: slots.into_iter().collect(),
            slot_gap_override: None,
            slot_size_override: None,
            slot_text_px_override: None,
            slot_line_height_px_override: None,
            slot_corner_mode: None,
        }
    }

    pub fn slot_gap_px(mut self, gap: Px) -> Self {
        self.slot_gap_override = Some(gap);
        self
    }

    pub fn slot_size_px(mut self, w: Px, h: Px) -> Self {
        self.slot_size_override = Some((w, h));
        self
    }

    pub fn slot_text_px(mut self, px: Px) -> Self {
        self.slot_text_px_override = Some(px);
        self
    }

    pub fn slot_line_height_px(mut self, px: Px) -> Self {
        self.slot_line_height_px_override = Some(px);
        self
    }

    pub fn slot_corner_mode(mut self, mode: InputOtpSlotCornerMode) -> Self {
        self.slot_corner_mode = Some(mode);
        self
    }
}

/// shadcn/ui `InputOTPSlot` (v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputOtpSlot {
    index: usize,
    aria_invalid: bool,
}

impl InputOtpSlot {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            aria_invalid: false,
        }
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }
}

/// shadcn/ui `InputOTPSeparator` (v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputOtpSeparator;

/// shadcn/ui v4 part surface for `InputOtp`.
#[derive(Debug, Clone)]
pub enum InputOtpPart {
    Group(InputOtpGroup),
    Slot(InputOtpSlot),
    Separator(InputOtpSeparator),
}

impl From<InputOtpGroup> for InputOtpPart {
    fn from(value: InputOtpGroup) -> Self {
        Self::Group(value)
    }
}

impl From<InputOtpSlot> for InputOtpPart {
    fn from(value: InputOtpSlot) -> Self {
        Self::Slot(value)
    }
}

impl From<InputOtpSeparator> for InputOtpPart {
    fn from(value: InputOtpSeparator) -> Self {
        Self::Separator(value)
    }
}

impl InputOtpPart {
    pub fn group(group: InputOtpGroup) -> Self {
        Self::Group(group)
    }

    pub fn slot(slot: InputOtpSlot) -> Self {
        Self::Slot(slot)
    }

    pub fn separator(separator: InputOtpSeparator) -> Self {
        Self::Separator(separator)
    }
}

pub fn input_otp<H: UiHost>(otp: InputOtp) -> impl IntoUiElement<H> + use<H> {
    otp
}

/// shadcn/ui `InputOTP` (v4).
///
/// Upstream exports this type as `InputOTP`. Fret's canonical Rust name is [`InputOtp`]; this
/// alias exists to improve copy/paste parity with shadcn docs/examples.
pub type InputOTP = InputOtp;

/// shadcn/ui `InputOTPGroup` (v4).
pub type InputOTPGroup = InputOtpGroup;

/// shadcn/ui `InputOTPSlot` (v4).
pub type InputOTPSlot = InputOtpSlot;

/// shadcn/ui `InputOTPSeparator` (v4).
pub type InputOTPSeparator = InputOtpSeparator;

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::window::ColorScheme;
    use fret_core::{
        AppWindowId, NodeId, Point, Rect, TextBlobId, TextConstraints, TextMetrics, TextService,
        WindowFrameClockService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::ThemeConfig;
    use fret_ui::elements::ElementRuntime;
    use fret_ui::tree::UiTree;
    use std::time::Duration;

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

    impl PathService for FakeServices {
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

    #[test]
    fn otp_invalid_ring_color_tracks_theme_color_scheme() {
        let mut app = fret_app::App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                color_scheme: Some(ColorScheme::Dark),
                colors: std::collections::HashMap::from([
                    ("destructive".to_string(), "#ff0000".to_string()),
                    ("destructive/40".to_string(), "#00ff00".to_string()),
                    ("destructive/20".to_string(), "#0000ff".to_string()),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            otp_invalid_ring_color(&theme),
            theme.color_by_key("destructive/40").unwrap()
        );

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config_patch(&ThemeConfig {
                color_scheme: Some(ColorScheme::Light),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            otp_invalid_ring_color(&theme),
            theme.color_by_key("destructive/20").unwrap()
        );
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

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        model: Model<String>,
    ) {
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "otp", |cx| {
                vec![
                    InputOtp::new(model)
                        .length(6)
                        .test_id_prefix("otp")
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn node_id_by_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) -> NodeId {
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("expected semantics node with test_id={id:?}"))
            .id
    }

    fn slot_descendant_has_caret_like_1px_bar(ui: &UiTree<App>, slot: NodeId) -> bool {
        let mut stack = ui.children(slot);
        while let Some(node) = stack.pop() {
            if let Some(bounds) = ui.debug_node_bounds(node)
                && (bounds.size.width.0 - 1.0).abs() <= 0.6
                && bounds.size.height.0 >= 8.0
            {
                return true;
            }
            stack.extend(ui.children(node));
        }
        false
    }

    #[test]
    fn input_otp_sanitizes_and_clamps_value() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;

        let model = app.models_mut().insert("12a 34-5678".to_string());
        render(&mut ui, &mut app, &mut services, model.clone());

        assert_eq!(app.models().get_cloned(&model).as_deref(), Some("123456"));
    }

    #[test]
    fn input_otp_pattern_digits_and_chars_filters_punctuation() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;

        let model = app.models_mut().insert("12a 34-5678".to_string());

        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "otp",
            |cx| {
                vec![
                    InputOtp::new(model.clone())
                        .length(6)
                        .pattern(InputOtpPattern::DigitsAndChars)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(app.models().get_cloned(&model).as_deref(), Some("12a345"));
    }

    #[test]
    fn input_otp_fake_caret_blinks_on_a_1000ms_cycle_under_fixed_delta() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let model = app.models_mut().insert(String::new());

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        ui.request_semantics_snapshot();
        render(&mut ui, &mut app, &mut services, model.clone());
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = node_id_by_test_id(snap, "otp.input");
        ui.set_focus(Some(input));

        // Focused: caret should be visible near t=0.
        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        ui.request_semantics_snapshot();
        render(&mut ui, &mut app, &mut services, model.clone());
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let slot0 = node_id_by_test_id(snap, "otp.slot.0");
        assert!(
            slot_descendant_has_caret_like_1px_bar(&ui, slot0),
            "expected focused input otp slot 0 to render the fake caret near the start of the blink cycle"
        );

        // After ~640ms (40 frames @ 16ms), caret should be hidden (progress >= 0.5).
        for n in 3..=42 {
            app.set_tick_id(TickId(n));
            app.set_frame_id(FrameId(n));
            ui.request_semantics_snapshot();
            render(&mut ui, &mut app, &mut services, model.clone());
        }
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let slot0 = node_id_by_test_id(snap, "otp.slot.0");
        assert!(
            !slot_descendant_has_caret_like_1px_bar(&ui, slot0),
            "expected caret to blink off mid-cycle under fixed delta"
        );

        // Reduced motion: caret should stop blinking (remain visible).
        app.with_global_mut(ElementRuntime::new, |rt, _app| {
            rt.set_window_prefers_reduced_motion(window, Some(true));
        });
        for n in 43..=120 {
            app.set_tick_id(TickId(n));
            app.set_frame_id(FrameId(n));
            ui.request_semantics_snapshot();
            render(&mut ui, &mut app, &mut services, model.clone());
        }
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let slot0 = node_id_by_test_id(snap, "otp.slot.0");
        assert!(
            slot_descendant_has_caret_like_1px_bar(&ui, slot0),
            "expected reduced motion to disable the blink (caret remains visible)"
        );
    }

    fn find_element_by_test_id<'a>(el: &'a AnyElement, test_id: &str) -> Option<&'a AnyElement> {
        if el
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return Some(el);
        }
        for child in &el.children {
            if let Some(found) = find_element_by_test_id(child, test_id) {
                return Some(found);
            }
        }
        None
    }

    fn find_slot_border_color(el: &AnyElement, test_id: &str) -> Option<Color> {
        let slot = find_element_by_test_id(el, test_id)?;
        match &slot.kind {
            fret_ui::element::ElementKind::Container(props) => props.border_color,
            _ => None,
        }
    }

    fn find_active_ring_shadow_alpha(el: &AnyElement, ring_spread: Px) -> Option<f32> {
        let mut best: Option<f32> = None;
        let mut stack = vec![el];
        while let Some(node) = stack.pop() {
            if let fret_ui::element::ElementKind::Container(props) = &node.kind
                && let Some(shadow) = props.shadow
                && (shadow.primary.offset_x.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.offset_y.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.blur.0 - 0.0).abs() <= 1e-6
                && (shadow.primary.spread.0 - ring_spread.0).abs() <= 1e-6
            {
                best = Some(best.map_or(shadow.primary.color.a, |cur| {
                    cur.max(shadow.primary.color.a)
                }));
            }
            for child in &node.children {
                stack.push(child);
            }
        }
        best
    }

    #[test]
    fn input_otp_slot_border_and_ring_tween_like_transition_all() {
        fn color_eq_eps(a: Color, b: Color, eps: f32) -> bool {
            (a.r - b.r).abs() <= eps
                && (a.g - b.g).abs() <= eps
                && (a.b - b.b).abs() <= eps
                && (a.a - b.a).abs() <= eps
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let model = app.models_mut().insert(String::new());

        let theme_full = Theme::global(&app);
        let theme = theme_full.snapshot();
        let resolved = resolve_input_chrome(
            theme_full,
            ComponentSize::default(),
            &ChromeRefinement::default(),
            InputTokenKeys::none(),
        );
        let ring_spread = otp_active_ring_width(&theme);
        let ring_base_alpha = otp_slot_ring_color(&theme, false).a;
        let border_base = otp_slot_border_color(
            &theme,
            resolved.border_color,
            resolved.border_color_focused,
            false,
            false,
        );
        let border_active = otp_slot_border_color(
            &theme,
            resolved.border_color,
            resolved.border_color_focused,
            true,
            false,
        );

        fn render_capture(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            model: Model<String>,
            slot_border_out: &mut Option<Color>,
            ring_alpha_out: &mut Option<f32>,
            ring_spread: Px,
        ) {
            let window = AppWindowId::default();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(480.0), Px(120.0)),
            );

            let root =
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "otp", |cx| {
                    let el = InputOtp::new(model)
                        .length(6)
                        .test_id_prefix("otp")
                        .into_element(cx);
                    *slot_border_out = find_slot_border_color(&el, "otp.slot.0");
                    *ring_alpha_out = find_active_ring_shadow_alpha(&el, ring_spread);
                    vec![el]
                });
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        }

        // Frame 1: unfocused, border should be base and ring should be absent.
        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let mut border_out: Option<Color> = None;
        let mut ring_alpha_out: Option<f32> = None;
        render_capture(
            &mut ui,
            &mut app,
            &mut services,
            model.clone(),
            &mut border_out,
            &mut ring_alpha_out,
            ring_spread,
        );
        let border0 = border_out.expect("border0");
        assert!(
            (border0.r - border_base.r).abs() <= 1e-6
                && (border0.g - border_base.g).abs() <= 1e-6
                && (border0.b - border_base.b).abs() <= 1e-6
                && (border0.a - border_base.a).abs() <= 1e-6,
            "expected base border color before focus; got border0={border0:?} base={border_base:?}"
        );
        assert!(
            ring_alpha_out.unwrap_or(0.0) <= 1e-6,
            "expected no ring before focus; got {:?}",
            ring_alpha_out
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = node_id_by_test_id(snap, "otp.input");
        ui.set_focus(Some(input));

        // Frame 2: focused, border + ring should tween in (intermediate).
        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        render_capture(
            &mut ui,
            &mut app,
            &mut services,
            model.clone(),
            &mut border_out,
            &mut ring_alpha_out,
            ring_spread,
        );
        let border1 = border_out.expect("border1");
        assert!(
            !color_eq_eps(border1, border_base, 1e-6)
                && !color_eq_eps(border1, border_active, 1e-6),
            "expected focused border to tween (intermediate); got border1={border1:?} base={border_base:?} active={border_active:?}"
        );
        let ring1 = ring_alpha_out.expect("ring1");
        assert!(
            ring1 > 1e-6 && ring1 < ring_base_alpha - 1e-6,
            "expected ring alpha to tween in (intermediate); got ring1={ring1} base_alpha={ring_base_alpha}"
        );

        // Settle.
        let settle = fret_ui_kit::declarative::transition::ticks_60hz_for_duration(
            Duration::from_millis(150),
        ) + 2;
        for n in 0..settle {
            app.set_tick_id(TickId(3 + n));
            app.set_frame_id(FrameId(3 + n));
            render_capture(
                &mut ui,
                &mut app,
                &mut services,
                model.clone(),
                &mut border_out,
                &mut ring_alpha_out,
                ring_spread,
            );
        }
        let border_final = border_out.expect("border_final");
        assert!(
            color_eq_eps(border_final, border_active, 1e-4),
            "expected border to settle; got border_final={border_final:?} active={border_active:?}"
        );
        let ring_final = ring_alpha_out.expect("ring_final");
        assert!(
            (ring_final - ring_base_alpha).abs() <= 1e-4,
            "expected ring to settle; got ring_final={ring_final} base_alpha={ring_base_alpha}"
        );

        // Blur: border + ring should tween out (intermediate).
        ui.set_focus(None);
        app.set_tick_id(TickId(1000));
        app.set_frame_id(FrameId(1000));
        render_capture(
            &mut ui,
            &mut app,
            &mut services,
            model.clone(),
            &mut border_out,
            &mut ring_alpha_out,
            ring_spread,
        );
        let border_out1 = border_out.expect("border_out1");
        assert!(
            !color_eq_eps(border_out1, border_base, 1e-6)
                && !color_eq_eps(border_out1, border_active, 1e-6),
            "expected blur border to tween out (intermediate); got border_out1={border_out1:?} base={border_base:?} active={border_active:?}"
        );
        let ring_out1 = ring_alpha_out.expect("ring_out1");
        assert!(
            ring_out1 > 1e-6 && ring_out1 < ring_base_alpha - 1e-6,
            "expected ring alpha to tween out (intermediate); got ring_out1={ring_out1} base_alpha={ring_base_alpha}"
        );
    }

    #[test]
    fn input_otp_aria_invalid_uses_destructive_border_color() {
        let app = App::new();
        let theme_full = Theme::global(&app);
        let theme = theme_full.snapshot();
        let resolved = resolve_input_chrome(
            theme_full,
            ComponentSize::default(),
            &ChromeRefinement::default(),
            InputTokenKeys::none(),
        );

        let invalid = otp_slot_border_color(
            &theme,
            resolved.border_color,
            resolved.border_color_focused,
            false,
            true,
        );

        assert_eq!(invalid, theme.color_token("destructive"));
    }

    #[test]
    fn input_otp_aria_invalid_sets_semantics_invalid() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;

        let model = app.models_mut().insert("000000".to_string());

        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "otp",
            |cx| {
                vec![
                    InputOtp::new(model)
                        .length(6)
                        .aria_invalid(true)
                        .test_id_prefix("otp")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp.input"))
            .expect("expected semantics node otp.input");
        assert_eq!(node.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }

    #[test]
    fn input_otp_slot_part_aria_invalid_sets_hidden_input_semantics_invalid() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;

        let model = app.models_mut().insert("000000".to_string());

        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "otp-slot-invalid",
            |cx| {
                vec![
                    InputOtp::new(model)
                        .length(6)
                        .test_id_prefix("otp-slot-invalid")
                        .into_element_parts(cx, |_cx| {
                            vec![
                                InputOtpGroup::new([
                                    InputOtpSlot::new(0).aria_invalid(true),
                                    InputOtpSlot::new(1),
                                    InputOtpSlot::new(2),
                                    InputOtpSlot::new(3),
                                    InputOtpSlot::new(4),
                                    InputOtpSlot::new(5),
                                ])
                                .into(),
                            ]
                        }),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp-slot-invalid.input"))
            .expect("expected semantics node otp-slot-invalid.input");
        assert_eq!(node.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }

    #[test]
    fn input_otp_required_builder_exposes_required_semantics() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;

        let model = app.models_mut().insert("000000".to_string());

        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "otp-required",
            |cx| {
                vec![
                    InputOtp::new(model)
                        .length(6)
                        .required(true)
                        .test_id_prefix("otp-required")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp-required.input"))
            .expect("expected semantics node otp-required.input");
        assert!(node.flags.required);
    }

    #[test]
    fn input_otp_group_part_slot_size_override_affects_only_that_group() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices;
        let model = app.models_mut().insert(String::new());

        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "otp-group-slot-size",
            |cx| {
                vec![
                    InputOtp::new(model)
                        .length(6)
                        .test_id_prefix("otp-group-slot-size")
                        .into_element_parts(cx, |_cx| {
                            vec![
                                InputOtpGroup::new([
                                    InputOtpSlot::new(0),
                                    InputOtpSlot::new(1),
                                    InputOtpSlot::new(2),
                                ])
                                .slot_size_px(Px(44.0), Px(48.0))
                                .into(),
                                InputOtpSeparator.into(),
                                InputOtpGroup::new([
                                    InputOtpSlot::new(3),
                                    InputOtpSlot::new(4),
                                    InputOtpSlot::new(5),
                                ])
                                .into(),
                            ]
                        }),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let slot0 = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp-group-slot-size.slot.0"))
            .expect("expected semantics node for slot 0");
        let slot3 = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("otp-group-slot-size.slot.3"))
            .expect("expected semantics node for slot 3");

        assert!((slot0.bounds.size.width.0 - 44.0).abs() <= 1.0);
        assert!((slot0.bounds.size.height.0 - 48.0).abs() <= 1.0);
        assert!(slot0.bounds.size.width.0 > slot3.bounds.size.width.0 + 1.0);
        assert!(slot0.bounds.size.height.0 > slot3.bounds.size.height.0 + 1.0);
    }

    #[test]
    fn input_otp_control_id_uses_registry_labelled_by_and_described_by_elements() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());
        let root = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "control-id-input-otp",
            |cx| {
                let id =
                    fret_ui_kit::primitives::control_registry::ControlId::from("otp-verification");

                cx.column(fret_ui::element::ColumnProps::default(), move |cx| {
                    vec![
                        crate::field::FieldLabel::new("Verification code")
                            .for_control(id.clone())
                            .into_element(cx),
                        crate::field::FieldDescription::new(
                            "Click the label to focus the OTP control.",
                        )
                        .for_control(id.clone())
                        .into_element(cx),
                        InputOtp::new(model.clone()).control_id(id).into_element(cx),
                    ]
                })
            },
        );

        let label_id = root.children[0].id;
        let desc_id = root.children[1].id;

        fn find_text_input(el: &AnyElement) -> Option<&AnyElement> {
            if matches!(el.kind, fret_ui::element::ElementKind::TextInput(_)) {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text_input(child) {
                    return Some(found);
                }
            }
            None
        }

        let input = find_text_input(&root).expect("expected a TextInput node");
        let decoration = input
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(label_id.0));
        assert_eq!(decoration.described_by_element, Some(desc_id.0));
    }

    fn count_svg_icons(node: &AnyElement) -> usize {
        let mut out = match &node.kind {
            fret_ui::element::ElementKind::SvgIcon(_) => 1,
            _ => 0,
        };
        for child in &node.children {
            out += count_svg_icons(child);
        }
        out
    }

    fn has_semantics_role(node: &AnyElement, role: SemanticsRole) -> bool {
        if node.semantics_decoration.as_ref().and_then(|d| d.role) == Some(role) {
            return true;
        }
        node.children
            .iter()
            .any(|child| has_semantics_role(child, role))
    }

    fn count_semantics_role(node: &AnyElement, role: SemanticsRole) -> usize {
        let mut out =
            usize::from(node.semantics_decoration.as_ref().and_then(|d| d.role) == Some(role));
        for child in &node.children {
            out += count_semantics_role(child, role);
        }
        out
    }

    #[test]
    fn input_otp_parts_infer_length_and_respect_explicit_separators() {
        use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};
        use fret_core::{AppWindowId, Point, Px, Rect, Size};

        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let model = app.models_mut().insert("12a 34-5678".to_string());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "input_otp_parts", |cx| {
                InputOtp::new(model.clone())
                    .length(4)
                    .numeric_only(true)
                    .into_element_parts(cx, |_cx| {
                        vec![
                            InputOtpPart::group(InputOtpGroup::new([
                                InputOtpSlot::new(0),
                                InputOtpSlot::new(1),
                                InputOtpSlot::new(2),
                            ])),
                            InputOtpPart::separator(InputOtpSeparator),
                            InputOtpPart::group(InputOtpGroup::new([
                                InputOtpSlot::new(3),
                                InputOtpSlot::new(4),
                                InputOtpSlot::new(5),
                            ])),
                        ]
                    })
            });

        assert_eq!(app.models().get_cloned(&model).as_deref(), Some("123456"));
        assert_eq!(count_svg_icons(&element), 1);
        assert!(
            has_semantics_role(&element, SemanticsRole::Separator),
            "expected InputOTPSeparator to emit separator semantics like upstream role=separator"
        );
    }

    #[test]
    fn input_otp_group_size_auto_separators_cover_the_default_gallery_lane() {
        use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};
        use fret_core::{AppWindowId, Point, Px, Rect, Size};

        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let model = app.models_mut().insert("123456".to_string());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "input_otp_group_size",
            |cx| {
                InputOtp::new(model.clone())
                    .length(6)
                    .group_size(Some(3))
                    .into_element(cx)
            },
        );

        assert_eq!(count_svg_icons(&element), 1);
        assert_eq!(count_semantics_role(&element, SemanticsRole::Separator), 1);
    }

    #[test]
    fn input_otp_disabled_wraps_in_opacity() {
        use fret_ui::element::ElementKind;

        let mut app = App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let model = app.models_mut().insert("123456".to_string());
        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "input_otp_disabled_opacity",
            |cx| InputOtp::new(model).disabled(true).into_element(cx),
        );

        let ElementKind::Opacity(_) = &el.kind else {
            panic!("expected disabled InputOtp to be wrapped in Opacity");
        };
    }
}
