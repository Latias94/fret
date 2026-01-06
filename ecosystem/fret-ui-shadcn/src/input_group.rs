use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, FlexProps, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize, Space};

#[derive(Clone)]
pub struct InputGroup {
    model: Model<String>,
    leading: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    a11y_label: Option<Arc<str>>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for InputGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputGroup")
            .field("model", &"<model>")
            .field("leading_len", &self.leading.len())
            .field("trailing_len", &self.trailing.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl InputGroup {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            leading: Vec::new(),
            trailing: Vec::new(),
            a11y_label: None,
            submit_command: None,
            cancel_command: None,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn leading(mut self, children: Vec<AnyElement>) -> Self {
        self.leading = children;
        self
    }

    pub fn trailing(mut self, children: Vec<AnyElement>) -> Self {
        self.trailing = children;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let resolved =
            resolve_input_chrome(&theme, self.size, &self.chrome, InputTokenKeys::none());

        let slot_w = Px(self.size.input_h(&theme).0.max(0.0));
        let base_px = resolved.padding.left;
        let base_py = resolved.padding.top;

        let left_pad = if self.leading.is_empty() {
            base_px
        } else {
            Px((base_px.0 + slot_w.0).max(0.0))
        };
        let right_pad = if self.trailing.is_empty() {
            base_px
        } else {
            Px((base_px.0 + slot_w.0).max(0.0))
        };

        let mut chrome = TextInputStyle::from_theme(theme.snapshot());
        chrome.padding = Edges {
            top: base_py,
            right: right_pad,
            bottom: base_py,
            left: left_pad,
        };
        chrome.corner_radii = Corners::all(resolved.radius);
        chrome.border = Edges::all(resolved.border_width);
        chrome.background = resolved.background;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.text_color = resolved.text_color;
        chrome.caret_color = resolved.text_color;
        chrome.selection_color = resolved.selection_color;

        let font_line_height = theme.metric_required("font.line_height");
        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            line_height: Some(font_line_height),
            ..Default::default()
        };

        let mut input = TextInputProps::new(self.model);
        input.a11y_label = self.a11y_label;
        input.submit_command = self.submit_command;
        input.cancel_command = self.cancel_command;
        input.chrome = chrome;
        input.text_style = text_style;
        input.layout.size = SizeStyle {
            width: Length::Fill,
            min_height: Some(resolved.min_height),
            ..Default::default()
        };

        let root_layout = decl_style::layout_style(&theme, self.layout.relative().w_full());

        let leading = self.leading;
        let trailing = self.trailing;

        cx.container(
            fret_ui::element::ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out = Vec::new();
                out.push(cx.text_input(input));

                if !leading.is_empty() {
                    let left_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .left(Space::N0)
                            .top(Space::N0)
                            .bottom(Space::N0)
                            .w_px(fret_ui_kit::MetricRef::Px(slot_w))
                            .h_full(),
                    );
                    out.push(cx.flex(
                        FlexProps {
                            layout: left_layout,
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::symmetric(base_px, Px(0.0)),
                            justify: fret_ui::element::MainAlign::Center,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        |_cx| leading,
                    ));
                }

                if !trailing.is_empty() {
                    let right_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .right(Space::N0)
                            .top(Space::N0)
                            .bottom(Space::N0)
                            .w_px(fret_ui_kit::MetricRef::Px(slot_w))
                            .h_full(),
                    );

                    out.push(cx.flex(
                        FlexProps {
                            layout: right_layout,
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::symmetric(base_px, Px(0.0)),
                            justify: fret_ui::element::MainAlign::Center,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        |_cx| trailing,
                    ));
                }

                out
            },
        )
    }
}

pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement {
    group.into_element(cx)
}
