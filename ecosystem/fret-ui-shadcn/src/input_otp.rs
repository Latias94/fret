use fret_core::{Axis, Color, Corners, Edges, FontId, FontWeight, Px};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    PositionStyle, SemanticsDecoration, ShadowLayerStyle, ShadowStyle, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Size as ComponentSize, Space, ui,
};
use std::sync::Arc;
use std::time::Duration;

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

fn sanitize_otp(input: &str, length: usize, numeric_only: bool) -> String {
    let mut out = String::new();
    out.reserve(length.min(input.len()));
    for ch in input.chars() {
        if out.chars().count() >= length {
            break;
        }
        if ch.is_whitespace() {
            continue;
        }
        if numeric_only && !ch.is_ascii_digit() {
            continue;
        }
        if ch.is_control() {
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
    length: usize,
    numeric_only: bool,
    group_size: Option<usize>,
    explicit_groups: Option<Vec<Vec<usize>>>,
    separator_mode: InputOtpSeparatorMode,
    aria_invalid: bool,
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
            .field("length", &self.length)
            .field("numeric_only", &self.numeric_only)
            .field("group_size", &self.group_size)
            .field(
                "explicit_groups",
                &self.explicit_groups.as_ref().map(|g| g.len()),
            )
            .field("separator_mode", &self.separator_mode)
            .field("aria_invalid", &self.aria_invalid)
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
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            length: 6,
            numeric_only: true,
            group_size: None,
            explicit_groups: None,
            separator_mode: InputOtpSeparatorMode::AutoBetweenGroups,
            aria_invalid: false,
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

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(1);
        self
    }

    pub fn numeric_only(mut self, numeric_only: bool) -> Self {
        self.numeric_only = numeric_only;
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
        let sanitized = sanitize_otp(&value, length, self.numeric_only);
        if sanitized != value {
            let next = sanitized.clone();
            let _ = cx.app.models_mut().update(&self.model, |v| *v = next);
            value = sanitized;
        }

        let chars: Vec<char> = value.chars().collect();
        let resolved = resolve_input_chrome(
            Theme::global(&*cx.app),
            self.size,
            &self.chrome,
            InputTokenKeys::none(),
        );

        let default_slot_w = Px(resolved.min_height.0.max(0.0));
        let default_slot_h = Px(resolved.min_height.0.max(0.0));
        let (slot_w, slot_h) = self
            .slot_size_override
            .unwrap_or((default_slot_w, default_slot_h));

        let container_gap = self
            .container_gap_override
            .unwrap_or_else(|| otp_gap(&theme));
        let slot_gap = self.slot_gap_override.unwrap_or(Px(0.0));
        let slot_gap_is_nonzero = slot_gap.0.abs() >= 0.5;

        let font_line_height = theme.metric_token("font.line_height");
        let slot_line_height = self
            .slot_line_height_px_override
            .unwrap_or(font_line_height);
        let slot_text_px = self.slot_text_px_override.unwrap_or(resolved.text_px);
        let mut slot_text_style =
            typography::fixed_line_box_style(FontId::ui(), slot_text_px, slot_line_height);
        slot_text_style.weight = FontWeight::MEDIUM;

        let root_layout = decl_style::layout_style(&theme, self.layout.relative());
        let separator_color = otp_separator_color(&theme);

        cx.container(
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

                let groups: Vec<Vec<usize>> =
                    if let Some(explicit_groups) = self.explicit_groups.as_ref() {
                        explicit_groups.clone()
                    } else {
                        let group_size = self.group_size.unwrap_or(length).max(1);
                        let mut out: Vec<Vec<usize>> = Vec::new();
                        let mut start = 0;
                        while start < length {
                            let end = (start + group_size).min(length);
                            out.push((start..end).collect());
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

                let mut input = TextInputProps::new(self.model);
                input.chrome = chrome;
                input.text_style = slot_text_style.clone();
                input.test_id = input_test_id.clone();
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
                let input_el = cx.text_input(input);

                let input_id = input_el.id;
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
                let mut pieces: Vec<AnyElement> = Vec::new();
                for (group_idx, group_slots) in groups.iter().enumerate() {
                    let mut slots: Vec<AnyElement> = Vec::new();
                    for (slot_pos, idx) in group_slots.iter().copied().enumerate() {
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
                        let border_color = otp_slot_border_color(
                            &theme,
                            resolved.border_color,
                            resolved.border_color_focused,
                            is_active,
                            self.aria_invalid,
                        );
                        let radius = resolved.radius;
                        let fg = resolved.text_color;

                        let all_corners = slot_gap_is_nonzero
                            || matches!(self.slot_corner_mode, InputOtpSlotCornerMode::All);
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
                                        let mut label = ui::label(cx, text)
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

                        slot_ids[idx] = Some(slot_el.id);
                        slot_corners[idx] = Some(corner_radii);

                        let mut decoration = SemanticsDecoration::default().selected(is_active);
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
                        pieces.push(cx.flex(
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
                        ));
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

                if focused {
                    if let Some(active_idx) = active_slot_idx {
                        let root_bounds = cx.last_bounds_for_element(cx.root_id());
                        let slot_bounds = slot_ids
                            .get(active_idx)
                            .copied()
                            .flatten()
                            .and_then(|id| cx.last_bounds_for_element(id));
                        if let (Some(root_bounds), Some(slot_bounds)) = (root_bounds, slot_bounds) {
                            let ring_w = otp_active_ring_width(&theme);
                            let ring_color = otp_slot_ring_color(&theme, self.aria_invalid);
                            let corners = slot_corners
                                .get(active_idx)
                                .copied()
                                .flatten()
                                .unwrap_or_else(|| Corners::all(resolved.radius));
                            let left =
                                Px((slot_bounds.origin.x.0 - root_bounds.origin.x.0).max(0.0));
                            let top =
                                Px((slot_bounds.origin.y.0 - root_bounds.origin.y.0).max(0.0));

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
                                                width: Length::Px(slot_bounds.size.width),
                                                height: Length::Px(slot_bounds.size.height),
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
                    }
                }

                out
            },
        )
    }
}

#[derive(Debug, Clone)]
struct ParsedInputOtpParts {
    groups: Vec<Vec<usize>>,
    separators_after_groups: Vec<usize>,
    inferred_length: Option<usize>,
}

fn parse_input_otp_parts(parts: Vec<InputOtpPart>) -> ParsedInputOtpParts {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current_group: Vec<usize> = Vec::new();
    let mut separators_after_groups: Vec<usize> = Vec::new();
    let mut max_index: Option<usize> = None;

    for part in parts {
        match part {
            InputOtpPart::Group(group) => {
                if !current_group.is_empty() {
                    groups.push(std::mem::take(&mut current_group));
                }

                let mut indices: Vec<usize> = Vec::new();
                for slot in group.slots {
                    max_index = Some(max_index.map_or(slot.index, |m| m.max(slot.index)));
                    indices.push(slot.index);
                }
                if !indices.is_empty() {
                    groups.push(indices);
                }
            }
            InputOtpPart::Slot(slot) => {
                max_index = Some(max_index.map_or(slot.index, |m| m.max(slot.index)));
                current_group.push(slot.index);
            }
            InputOtpPart::Separator(_) => {
                if !current_group.is_empty() {
                    groups.push(std::mem::take(&mut current_group));
                }

                if let Some(idx) = groups.len().checked_sub(1) {
                    if !separators_after_groups.contains(&idx) {
                        separators_after_groups.push(idx);
                    }
                }
            }
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
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
}

impl InputOtpGroup {
    pub fn new(slots: impl IntoIterator<Item = InputOtpSlot>) -> Self {
        Self {
            slots: slots.into_iter().collect(),
        }
    }
}

/// shadcn/ui `InputOTPSlot` (v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputOtpSlot {
    index: usize,
}

impl InputOtpSlot {
    pub fn new(index: usize) -> Self {
        Self { index }
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

pub fn input_otp<H: UiHost>(cx: &mut ElementContext<'_, H>, otp: InputOtp) -> AnyElement {
    otp.into_element(cx)
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
        AppWindowId, Point, Rect, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_ui::ThemeConfig;
    use fret_ui::tree::UiTree;

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
                vec![InputOtp::new(model).length(6).into_element(cx)]
            });
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn input_otp_sanitizes_and_clamps_value() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices::default();

        let model = app.models_mut().insert("12a 34-5678".to_string());
        render(&mut ui, &mut app, &mut services, model.clone());

        assert_eq!(app.models().get_cloned(&model).as_deref(), Some("123456"));
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
    }
}
