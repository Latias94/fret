use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize};

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

    pub fn leading(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.leading = children.into_iter().collect();
        self
    }

    pub fn trailing(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.trailing = children.into_iter().collect();
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

        let base_px = resolved.padding.left;
        let base_py = resolved.padding.top;

        let mut chrome = TextInputStyle::from_theme(theme.snapshot());
        chrome.padding = Edges {
            top: base_py,
            right: if self.trailing.is_empty() {
                base_px
            } else {
                // Tailwind/shadcn input-group uses `pr-2` when a trailing slot exists.
                Px((base_px.0 - 4.0).max(0.0))
            },
            bottom: base_py,
            left: if self.leading.is_empty() {
                base_px
            } else {
                // Tailwind/shadcn input-group uses `pl-2` when a leading slot exists.
                Px((base_px.0 - 4.0).max(0.0))
            },
        };
        chrome.corner_radii = Corners::all(Px(0.0));
        chrome.border = Edges::all(Px(0.0));
        chrome.background = fret_core::Color {
            a: 0.0,
            ..resolved.background
        };
        chrome.border_color = fret_core::Color {
            a: 0.0,
            ..resolved.border_color
        };
        chrome.border_color_focused = fret_core::Color {
            a: 0.0,
            ..resolved.border_color_focused
        };
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
            width: Length::Auto,
            min_height: Some(resolved.min_height),
            min_width: Some(Px(0.0)),
            ..Default::default()
        };
        input.layout.flex.grow = 1.0;
        input.layout.flex.shrink = 1.0;
        input.layout.flex.basis = Length::Px(Px(0.0));

        let leading = self.leading;
        let trailing = self.trailing;

        let mut root_layout = decl_style::layout_style(&theme, self.layout.relative().w_full());
        root_layout.size.height = Length::Px(resolved.min_height);

        cx.container(
            ContainerProps {
                layout: root_layout,
                padding: Edges::all(Px(0.0)),
                background: Some(resolved.background),
                shadow: None,
                border: Edges::all(resolved.border_width),
                border_color: Some(resolved.border_color),
                corner_radii: Corners::all(resolved.radius),
            },
            move |cx| {
                let mut children = Vec::new();

                let mut row_layout = LayoutStyle::default();
                row_layout.size.width = Length::Fill;
                row_layout.size.height = Length::Fill;
                row_layout.flex.grow = 1.0;
                row_layout.flex.shrink = 1.0;
                row_layout.flex.basis = Length::Px(Px(0.0));

                let row = cx.flex(
                    FlexProps {
                        layout: row_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        let mut out = Vec::new();

                        if !leading.is_empty() {
                            let slot_py = Px((base_py.0 + 2.0).max(0.0));
                            out.push(cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: Axis::Horizontal,
                                    gap: Px(8.0),
                                    padding: Edges {
                                        top: slot_py,
                                        right: Px(0.0),
                                        bottom: slot_py,
                                        left: base_px,
                                    },
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                |_cx| leading,
                            ));
                        }

                        out.push(cx.text_input(input));

                        if !trailing.is_empty() {
                            let slot_py = Px((base_py.0 + 2.0).max(0.0));
                            out.push(cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: Axis::Horizontal,
                                    gap: Px(8.0),
                                    padding: Edges {
                                        top: slot_py,
                                        right: base_px,
                                        bottom: slot_py,
                                        left: Px(0.0),
                                    },
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                |_cx| trailing,
                            ));
                        }

                        out
                    },
                );

                children.push(row);
                children
            },
        )
    }
}

pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement {
    group.into_element(cx)
}
