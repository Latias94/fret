use fret_core::{Color, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, Overflow,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::IntoUiElement;

use crate::theme_tokens::{
    workspace_frame_background, workspace_status_bar_background, workspace_status_bar_border,
    workspace_top_bar_background, workspace_top_bar_border,
};

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
    layout.size.min_width = Some(Length::Px(Px(0.0)));
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

fn no_shrink_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.flex.shrink = 0.0;
    layout
}

#[derive(Debug)]
pub struct WorkspaceFrame<
    Center = AnyElement,
    Top = AnyElement,
    Left = AnyElement,
    Right = AnyElement,
    Bottom = AnyElement,
> {
    top: Option<Top>,
    left: Option<Left>,
    center: Center,
    right: Option<Right>,
    bottom: Option<Bottom>,
    background: Option<Color>,
}

impl<Center> WorkspaceFrame<Center> {
    pub fn new(center: Center) -> Self {
        Self {
            top: None,
            left: None,
            center,
            right: None,
            bottom: None,
            background: None,
        }
    }
}

impl<Center, Top, Left, Right, Bottom> WorkspaceFrame<Center, Top, Left, Right, Bottom> {
    pub fn top<Top2>(self, top: Top2) -> WorkspaceFrame<Center, Top2, Left, Right, Bottom> {
        let Self {
            left,
            center,
            right,
            bottom,
            background,
            ..
        } = self;
        WorkspaceFrame {
            top: Some(top),
            left,
            center,
            right,
            bottom,
            background,
        }
    }

    pub fn left<Left2>(self, left: Left2) -> WorkspaceFrame<Center, Top, Left2, Right, Bottom> {
        let Self {
            top,
            center,
            right,
            bottom,
            background,
            ..
        } = self;
        WorkspaceFrame {
            top,
            left: Some(left),
            center,
            right,
            bottom,
            background,
        }
    }

    pub fn right<Right2>(self, right: Right2) -> WorkspaceFrame<Center, Top, Left, Right2, Bottom> {
        let Self {
            top,
            left,
            center,
            bottom,
            background,
            ..
        } = self;
        WorkspaceFrame {
            top,
            left,
            center,
            right: Some(right),
            bottom,
            background,
        }
    }

    pub fn bottom<Bottom2>(
        self,
        bottom: Bottom2,
    ) -> WorkspaceFrame<Center, Top, Left, Right, Bottom2> {
        let Self {
            top,
            left,
            center,
            right,
            background,
            ..
        } = self;
        WorkspaceFrame {
            top,
            left,
            center,
            right,
            bottom: Some(bottom),
            background,
        }
    }

    pub fn background(mut self, background: Option<Color>) -> Self {
        self.background = background;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        Center: IntoUiElement<H>,
        Top: IntoUiElement<H>,
        Left: IntoUiElement<H>,
        Right: IntoUiElement<H>,
        Bottom: IntoUiElement<H>,
    {
        let theme = Theme::global(cx.app);
        let background = self
            .background
            .or_else(|| workspace_frame_background(theme));

        let top = self.top.map(|top| top.into_element(cx));
        let left = self.left.map(|left| left.into_element(cx));
        let center = self.center.into_element(cx);
        let right = self.right.map(|right| right.into_element(cx));
        let bottom = self.bottom.map(|bottom| bottom.into_element(cx));

        cx.container(
            ContainerProps {
                layout: fill_layout(),
                background,
                ..Default::default()
            },
            move |cx| {
                let center_row = cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = flex_grow_layout();
                            layout.size.min_height = Some(Length::Px(Px(0.0)));
                            layout
                        },
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

#[cfg(test)]
mod tests {
    use super::WorkspaceFrame;
    use fret_app::App;
    use fret_ui::ElementContext;
    use fret_ui::element::AnyElement;
    use fret_ui_kit::ui;

    #[allow(dead_code)]
    fn workspace_frame_accepts_typed_slot_inputs(cx: &mut ElementContext<'_, App>) -> AnyElement {
        WorkspaceFrame::new(ui::text("center"))
            .top(ui::text("top"))
            .left(ui::text("left"))
            .right(ui::text("right"))
            .bottom(ui::text("bottom"))
            .into_element(cx)
    }
}

/// Context-free top bar aggregator for workspace shells.
///
/// This intentionally remains an explicit `AnyElement` landing seam because it stores
/// heterogeneous child lists before an `ElementContext` exists.
#[derive(Debug)]
pub struct WorkspaceTopBar {
    height: Px,
    padding: Edges,
    left: Vec<AnyElement>,
    center: Vec<AnyElement>,
    right: Vec<AnyElement>,
}

impl Default for WorkspaceTopBar {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTopBar {
    pub fn new() -> Self {
        Self {
            height: Px(40.0),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);
        let bg = workspace_top_bar_background(theme);
        let border = workspace_top_bar_border(theme);

        cx.container(
            ContainerProps {
                layout: row_layout(self.height),
                padding: self.padding.into(),
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
                        gap: Px(8.0).into(),
                        align: CrossAlign::Center,
                        ..Default::default()
                    },
                    |cx| {
                        let mut children = Vec::new();
                        for child in self.left {
                            children.push(cx.container(
                                ContainerProps {
                                    layout: no_shrink_layout(),
                                    ..Default::default()
                                },
                                move |_cx| vec![child],
                            ));
                        }

                        if !self.center.is_empty() {
                            let mut center_layout = flex_grow_layout();
                            center_layout.overflow = Overflow::Clip;
                            children.push(cx.flex(
                                FlexProps {
                                    layout: center_layout,
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(8.0).into(),
                                    align: CrossAlign::Center,
                                    ..Default::default()
                                },
                                |_cx| self.center,
                            ));
                        } else {
                            children.push(cx.spacer(Default::default()));
                        }

                        for child in self.right {
                            children.push(cx.container(
                                ContainerProps {
                                    layout: no_shrink_layout(),
                                    ..Default::default()
                                },
                                move |_cx| vec![child],
                            ));
                        }
                        children
                    },
                )]
            },
        )
    }
}

/// Context-free status bar aggregator for workspace shells.
///
/// This intentionally remains an explicit `AnyElement` landing seam because it stores
/// heterogeneous child lists before an `ElementContext` exists.
#[derive(Debug)]
pub struct WorkspaceStatusBar {
    height: Px,
    padding: Edges,
    left: Vec<AnyElement>,
    right: Vec<AnyElement>,
}

impl Default for WorkspaceStatusBar {
    fn default() -> Self {
        Self::new()
    }
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);
        let bg = workspace_status_bar_background(theme);
        let border = workspace_status_bar_border(theme);

        cx.container(
            ContainerProps {
                layout: row_layout(self.height),
                padding: self.padding.into(),
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
                        gap: Px(8.0).into(),
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
