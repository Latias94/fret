use std::sync::Arc;

use fret_core::{Corners, FontId, NodeId, SemanticsRole, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, Length, Overflow, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size};

#[derive(Clone)]
pub struct Input {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    size: Size,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Input {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            a11y_role: None,
            placeholder: None,
            active_descendant: None,
            expanded: None,
            submit_command: None,
            cancel_command: None,
            size: Size::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        input_with_style(
            cx,
            self.model,
            self.a11y_label,
            self.a11y_role,
            self.placeholder,
            self.active_descendant,
            self.expanded,
            self.submit_command,
            self.cancel_command,
            self.size,
            self.chrome,
            self.layout,
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
        active_descendant,
        expanded,
        submit_command,
        cancel_command,
        Size::default(),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn input_with_style<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    size: Size,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
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

    cx.text_input(props)
}
