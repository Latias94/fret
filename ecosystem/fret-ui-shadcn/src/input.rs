use std::sync::Arc;

use fret_core::{Corners, FontId, NodeId, Px, SemanticsRole, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, Length, Overflow, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size};

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Clone)]
pub struct Input {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    size: Size,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
}

impl Input {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            a11y_role: None,
            placeholder: None,
            aria_invalid: false,
            disabled: false,
            active_descendant: None,
            expanded: None,
            submit_command: None,
            cancel_command: None,
            size: Size::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn a11y_role(mut self, role: SemanticsRole) -> Self {
        self.a11y_role = Some(role);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn active_descendant(mut self, node: NodeId) -> Self {
        self.active_descendant = Some(node);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    pub fn size(mut self, size: Size) -> Self {
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

    /// Overrides per-edge border widths (in px) for this input's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge borders (e.g. input groups).
    pub fn border_left_width_override(mut self, border: Px) -> Self {
        self.border_width_override.left = Some(border);
        self
    }

    pub fn border_right_width_override(mut self, border: Px) -> Self {
        self.border_width_override.right = Some(border);
        self
    }

    pub fn border_top_width_override(mut self, border: Px) -> Self {
        self.border_width_override.top = Some(border);
        self
    }

    pub fn border_bottom_width_override(mut self, border: Px) -> Self {
        self.border_width_override.bottom = Some(border);
        self
    }

    /// Overrides per-corner radii (in px) for this input's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge corner radii
    /// (`rounded-l-none`, `rounded-r-none`).
    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.corner_radii_override = Some(corners);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        input_with_style(
            cx,
            self.model,
            self.a11y_label,
            self.a11y_role,
            self.placeholder,
            self.aria_invalid,
            self.disabled,
            self.active_descendant,
            self.expanded,
            self.submit_command,
            self.cancel_command,
            self.size,
            self.chrome,
            self.layout,
            self.border_width_override,
            self.corner_radii_override,
        )
    }
}

pub fn input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
) -> AnyElement {
    input_with_style(
        cx,
        model,
        a11y_label,
        a11y_role,
        placeholder,
        false,
        false,
        active_descendant,
        expanded,
        submit_command,
        cancel_command,
        Size::default(),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
        BorderWidthOverride::default(),
        None,
    )
}

fn input_with_style<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    size: Size,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let resolved = resolve_input_chrome(
        &theme,
        size,
        &chrome_override,
        InputTokenKeys {
            bg: Some("component.input.bg"),
            border: Some("input"),
            border_focus: Some("ring"),
            fg: Some("foreground"),
            selection: Some("primary"),
            ..InputTokenKeys::none()
        },
    );

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = fret_core::Edges::all(resolved.border_width);
    chrome.background = resolved.background;
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.focus_ring = Some(decl_style::focus_ring(&theme, resolved.radius));
    chrome.text_color = resolved.text_color;
    chrome.placeholder_color = theme
        .color_by_key("muted-foreground")
        .unwrap_or(chrome.placeholder_color);
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    if aria_invalid {
        let border_color = theme.color_required("destructive");
        chrome.border_color = border_color;
        chrome.border_color_focused = border_color;
        if let Some(mut ring) = chrome.focus_ring.take() {
            let ring_key = if theme.name.contains("/dark") {
                "destructive/40"
            } else {
                "destructive/20"
            };
            ring.color = theme
                .color_by_key(ring_key)
                .or_else(|| theme.color_by_key("destructive/20"))
                .unwrap_or(border_color);
            chrome.focus_ring = Some(ring);
        }
    }

    if let Some(corners) = corner_radii_override {
        chrome.corner_radii = corners;
    }
    if let Some(border) = border_width_override.top {
        chrome.border.top = border;
    }
    if let Some(border) = border_width_override.right {
        chrome.border.right = border;
    }
    if let Some(border) = border_width_override.bottom {
        chrome.border.bottom = border;
    }
    if let Some(border) = border_width_override.left {
        chrome.border.left = border;
    }

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    let text_style = TextStyle {
        font: FontId::default(),
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    };

    let mut props = TextInputProps::new(model);
    props.enabled = !disabled;
    props.focusable = !disabled;
    props.a11y_label = a11y_label;
    props.a11y_role = a11y_role;
    props.placeholder = placeholder;
    props.active_descendant = active_descendant;
    props.expanded = expanded;
    props.submit_command = submit_command;
    props.cancel_command = cancel_command;
    props.chrome = chrome;
    props.text_style = text_style;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        min_width: Some(fret_core::Px(0.0)),
        min_height: Some(resolved.min_height),
        ..Default::default()
    };
    props.layout.overflow = Overflow::Clip;
    decl_style::apply_layout_refinement(&theme, layout_override, &mut props.layout);

    if disabled {
        cx.opacity(0.5, move |cx| vec![cx.text_input(props)])
    } else {
        cx.text_input(props)
    }
}
