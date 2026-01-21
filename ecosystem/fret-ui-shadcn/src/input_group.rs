use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, FlexProps, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize, Space};

#[derive(Clone)]
pub struct InputGroup {
    model: Model<String>,
    leading: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    leading_has_button: bool,
    trailing_has_button: bool,
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
            .field("leading_has_button", &self.leading_has_button)
            .field("trailing_has_button", &self.trailing_has_button)
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
            leading_has_button: false,
            trailing_has_button: false,
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

    /// Upstream uses `has-[>button]:ml-[-0.45rem]` on inline-start addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn leading_has_button(mut self, value: bool) -> Self {
        self.leading_has_button = value;
        self
    }

    /// Upstream uses `has-[>button]:mr-[-0.45rem]` on inline-end addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn trailing_has_button(mut self, value: bool) -> Self {
        self.trailing_has_button = value;
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

        let addon_pl = fret_ui_kit::MetricRef::space(Space::N3).resolve(&theme);
        let addon_py = fret_ui_kit::MetricRef::space(Space::N1p5).resolve(&theme);
        let compact_px = fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme);

        let left_pad = if self.leading.is_empty() {
            resolved.padding.left
        } else {
            compact_px
        };
        let right_pad = if self.trailing.is_empty() {
            resolved.padding.right
        } else {
            compact_px
        };

        let mut chrome = TextInputStyle::from_theme(theme.snapshot());
        chrome.padding = Edges {
            top: resolved.padding.top,
            right: right_pad,
            bottom: resolved.padding.bottom,
            left: left_pad,
        };
        chrome.corner_radii = Corners::all(Px(0.0));
        chrome.border = Edges::all(Px(0.0));
        chrome.background = Color::TRANSPARENT;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.focus_ring = None;
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
        input.layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .flex_1()
                .h_full()
                .min_w_0()
                .min_h(fret_ui_kit::MetricRef::Px(resolved.min_height)),
        );

        let root_layout = decl_style::layout_style(
            &theme,
            self.layout
                .relative()
                .w_full()
                .min_w_0()
                .h_px(fret_ui_kit::MetricRef::Px(resolved.min_height)),
        );

        let leading = self.leading;
        let trailing = self.trailing;
        let leading_has_button = self.leading_has_button;
        let trailing_has_button = self.trailing_has_button;

        cx.container(
            fret_ui::element::ContainerProps {
                layout: root_layout,
                background: None,
                shadow: Some(decl_style::shadow_xs(&theme, resolved.radius)),
                border: Edges::all(resolved.border_width),
                border_color: Some(resolved.border_color),
                corner_radii: Corners::all(resolved.radius),
                ..Default::default()
            },
            |cx| {
                let flex_layout =
                    decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

                let leading = (!leading.is_empty()).then(|| {
                    let layout = if leading_has_button {
                        LayoutRefinement::default().flex_none().ml_neg(Space::N2)
                    } else {
                        LayoutRefinement::default().flex_none()
                    };
                    cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(&theme, layout),
                            direction: Axis::Horizontal,
                            gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme),
                            padding: Edges {
                                top: addon_py,
                                right: Px(0.0),
                                bottom: addon_py,
                                left: addon_pl,
                            },
                            justify: fret_ui::element::MainAlign::Center,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        |_cx| leading,
                    )
                });

                let trailing = (!trailing.is_empty()).then(|| {
                    let layout = if trailing_has_button {
                        LayoutRefinement::default().flex_none().mr_neg(Space::N2)
                    } else {
                        LayoutRefinement::default().flex_none()
                    };
                    cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(&theme, layout),
                            direction: Axis::Horizontal,
                            gap: fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme),
                            padding: Edges {
                                top: addon_py,
                                right: addon_pl,
                                bottom: addon_py,
                                left: Px(0.0),
                            },
                            justify: fret_ui::element::MainAlign::Center,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        |_cx| trailing,
                    )
                });

                let input = cx.text_input(input);

                vec![cx.flex(
                    FlexProps {
                        layout: flex_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut children = Vec::new();
                        if let Some(leading) = leading {
                            children.push(leading);
                        }
                        children.push(input);
                        if let Some(trailing) = trailing {
                            children.push(trailing);
                        }
                        children
                    },
                )]
            },
        )
    }
}

pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement {
    group.into_element(cx)
}
