use std::sync::Arc;

use fret_components_ui::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_core::{Corners, FontId, NodeId, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, Length, Overflow, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};

#[derive(Clone)]
pub struct Input {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
}

impl Input {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            placeholder: None,
            active_descendant: None,
            submit_command: None,
            cancel_command: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        input(
            cx,
            self.model,
            self.a11y_label,
            self.placeholder,
            self.active_descendant,
            self.submit_command,
            self.cancel_command,
        )
    }
}

pub fn input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let resolved = resolve_input_chrome(
        &theme,
        fret_components_ui::Size::default(),
        &Default::default(),
        InputTokenKeys::none(),
    );

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = fret_core::Edges::all(resolved.border_width);
    chrome.background = resolved.background;
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.text_color = resolved.text_color;
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or(theme.metrics.font_line_height);
    let text_style = TextStyle {
        font: FontId::default(),
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    };

    let mut props = TextInputProps::new(model);
    props.a11y_label = a11y_label;
    props.placeholder = placeholder;
    props.active_descendant = active_descendant;
    props.submit_command = submit_command;
    props.cancel_command = cancel_command;
    props.chrome = chrome;
    props.text_style = text_style;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        min_height: Some(resolved.min_height),
        ..Default::default()
    };
    props.layout.overflow = Overflow::Clip;

    cx.text_input(props)
}
