use fret_core::{Color, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length};
use fret_ui::{ElementContext, Theme, UiHost};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn row_layout(height: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);
    layout.flex.shrink = 0.0;
    layout
}

fn flex_grow_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.min_width = Some(Px(0.0));
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout
}

fn fill_grow_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.flex.grow = 1.0;
    layout
}

#[derive(Debug, Clone)]
pub struct WorkspaceFrame {
    top: Option<AnyElement>,
    left: Option<AnyElement>,
    center: AnyElement,
    right: Option<AnyElement>,
    bottom: Option<AnyElement>,
    background: Option<Color>,
}

impl WorkspaceFrame {
    pub fn new(center: AnyElement) -> Self {
        Self {
            top: None,
            left: None,
            center,
            right: None,
            bottom: None,
            background: None,
        }
    }

    pub fn top(mut self, top: AnyElement) -> Self {
        self.top = Some(top);
        self
    }

    pub fn left(mut self, left: AnyElement) -> Self {
        self.left = Some(left);
        self
    }

    pub fn right(mut self, right: AnyElement) -> Self {
        self.right = Some(right);
        self
    }

    pub fn bottom(mut self, bottom: AnyElement) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn background(mut self, background: Option<Color>) -> Self {
        self.background = background;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);
        let background = self.background.or_else(|| theme.color_by_key("background"));

        let top = self.top;
        let left = self.left;
        let center = self.center;
        let right = self.right;
        let bottom = self.bottom;

        cx.container(
            ContainerProps {
                layout: fill_layout(),
                background,
                ..Default::default()
            },
            move |cx| {
                let center_row = cx.flex(
                    FlexProps {
                        layout: fill_grow_layout(),
                        direction: fret_core::Axis::Horizontal,
                        ..Default::default()
                    },
                    move |cx| {
                        let mut children = Vec::new();
                        if let Some(left) = left {
                            children.push(left);
                        }
                        children.push(cx.container(
                            ContainerProps {
                                layout: fill_grow_layout(),
                                ..Default::default()
                            },
                            move |_cx| vec![center],
                        ));
                        if let Some(right) = right {
                            children.push(right);
                        }
                        children
                    },
                );

                vec![cx.flex(
                    FlexProps {
                        layout: fill_layout(),
                        direction: fret_core::Axis::Vertical,
                        ..Default::default()
                    },
                    move |_cx| {
                        let mut children = Vec::new();
                        if let Some(top) = top {
                            children.push(top);
                        }
                        children.push(center_row);
                        if let Some(bottom) = bottom {
                            children.push(bottom);
                        }
                        children
                    },
                )]
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceTopBar {
    height: Px,
    padding: Edges,
    left: Vec<AnyElement>,
    center: Vec<AnyElement>,
    right: Vec<AnyElement>,
}

impl WorkspaceTopBar {
    pub fn new() -> Self {
        Self {
            height: Px(32.0),
            padding: Edges::all(Px(6.0)),
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }

    pub fn height(mut self, height: Px) -> Self {
        self.height = height;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn left(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.left.extend(children);
        self
    }

    pub fn center(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.center.extend(children);
        self
    }

    pub fn right(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.right.extend(children);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);
        let bg = theme
            .color_by_key("muted")
            .or_else(|| theme.color_by_key("background"));
        let border = theme.color_by_key("border");

        cx.container(
            ContainerProps {
                layout: row_layout(self.height),
                padding: self.padding,
                background: bg,
                border: Edges {
                    bottom: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                border_color: border,
                ..Default::default()
            },
            |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: fill_layout(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        align: CrossAlign::Center,
                        ..Default::default()
                    },
                    |cx| {
                        let mut children = Vec::new();
                        children.extend(self.left);

                        if !self.center.is_empty() {
                            children.push(cx.flex(
                                FlexProps {
                                    layout: flex_grow_layout(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(8.0),
                                    align: CrossAlign::Center,
                                    ..Default::default()
                                },
                                |_cx| self.center,
                            ));
                        } else {
                            children.push(cx.spacer(Default::default()));
                        }

                        children.extend(self.right);
                        children
                    },
                )]
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceStatusBar {
    height: Px,
    padding: Edges,
    left: Vec<AnyElement>,
    right: Vec<AnyElement>,
}

impl WorkspaceStatusBar {
    pub fn new() -> Self {
        Self {
            height: Px(24.0),
            padding: Edges::all(Px(6.0)),
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    pub fn height(mut self, height: Px) -> Self {
        self.height = height;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn left(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.left.extend(children);
        self
    }

    pub fn right(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.right.extend(children);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);
        let bg = theme
            .color_by_key("muted")
            .or_else(|| theme.color_by_key("background"));
        let border = theme.color_by_key("border");

        cx.container(
            ContainerProps {
                layout: row_layout(self.height),
                padding: self.padding,
                background: bg,
                border: Edges {
                    top: Px(1.0),
                    ..Edges::all(Px(0.0))
                },
                border_color: border,
                ..Default::default()
            },
            |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: fill_layout(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        align: CrossAlign::Center,
                        ..Default::default()
                    },
                    |cx| {
                        let mut children = Vec::new();
                        children.extend(self.left);
                        children.push(cx.spacer(Default::default()));
                        children.extend(self.right);
                        children
                    },
                )]
            },
        )
    }
}
