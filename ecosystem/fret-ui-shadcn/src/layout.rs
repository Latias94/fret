use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space, ui};

#[derive(Debug, Clone)]
pub(crate) struct VStackProps {
    gap: Space,
    layout: LayoutRefinement,
    items: Items,
    justify: Justify,
}

impl Default for VStackProps {
    fn default() -> Self {
        Self {
            gap: Space::N0,
            layout: LayoutRefinement::default(),
            items: Items::Stretch,
            justify: Justify::Start,
        }
    }
}

impl VStackProps {
    pub(crate) fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub(crate) fn gap_y(self, gap: Space) -> Self {
        self.gap(gap)
    }

    pub(crate) fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub(crate) fn items(mut self, items: Items) -> Self {
        self.items = items;
        self
    }

    pub(crate) fn items_center(self) -> Self {
        self.items(Items::Center)
    }

    pub(crate) fn items_start(self) -> Self {
        self.items(Items::Start)
    }

    pub(crate) fn items_end(self) -> Self {
        self.items(Items::End)
    }

    pub(crate) fn items_stretch(self) -> Self {
        self.items(Items::Stretch)
    }

    pub(crate) fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub(crate) fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub(crate) fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub(crate) fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HStackProps {
    gap: Space,
    layout: LayoutRefinement,
    items: Items,
    justify: Justify,
}

impl Default for HStackProps {
    fn default() -> Self {
        Self {
            gap: Space::N0,
            layout: LayoutRefinement::default(),
            items: Items::Center,
            justify: Justify::Start,
        }
    }
}

impl HStackProps {
    pub(crate) fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub(crate) fn gap_x(self, gap: Space) -> Self {
        self.gap(gap)
    }

    pub(crate) fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub(crate) fn items(mut self, items: Items) -> Self {
        self.items = items;
        self
    }

    pub(crate) fn items_center(self) -> Self {
        self.items(Items::Center)
    }

    pub(crate) fn items_start(self) -> Self {
        self.items(Items::Start)
    }

    pub(crate) fn items_end(self) -> Self {
        self.items(Items::End)
    }

    pub(crate) fn items_stretch(self) -> Self {
        self.items(Items::Stretch)
    }

    pub(crate) fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub(crate) fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub(crate) fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub(crate) fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    pub(crate) fn justify_between(self) -> Self {
        self.justify(Justify::Between)
    }
}

pub(crate) fn container_vstack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: VStackProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    let VStackProps {
        gap,
        layout,
        items,
        justify,
    } = stack_props;

    cx.container(props, move |cx| {
        let builder = ui::v_stack(move |_cx| children);
        vec![
            builder
                .gap(gap)
                .items(items)
                .justify(justify)
                .layout(layout)
                .into_element(cx),
        ]
    })
}

pub(crate) fn container_vstack_gap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_vstack(cx, props, VStackProps::default().gap(gap), children)
}

pub(crate) fn container_flow<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_vstack(cx, props, VStackProps::default().gap(Space::N0), children)
}

/// Flow stack variant that forces `width: fill`.
///
/// Use this for panel/content roots (popover/hover-card-like surfaces) where wrapped text should
/// resolve its wrap width against the container's inner width rather than shrink-wrapping to its
/// min-content size.
///
/// Important:
///
/// - This is intentionally stronger than `container_flow(...)`: the inner flow child requests
///   `w_full().min_w_0()`, and the layout engine will promote an auto-sized passthrough wrapper to
///   a definite width when needed so percent sizing does not collapse to zero.
/// - Use it only for roots that are expected to own panel/content width.
/// - Do not use it for shrink-wrapped trigger/button/menu-row chrome; those surfaces should keep
///   intrinsic width and rely on `container_flow(...)` (or a custom row/flex root) instead.
pub(crate) fn container_flow_fill_width<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.container(props, move |cx| {
        vec![
            ui::v_flex(move |_cx| children)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0)
                .into_element(cx),
        ]
    })
}

pub(crate) fn container_hstack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: HStackProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    let HStackProps {
        gap,
        layout,
        items,
        justify,
    } = stack_props;

    cx.container(props, move |cx| {
        let builder = ui::h_row(move |_cx| children);
        vec![
            builder
                .gap(gap)
                .items(items)
                .justify(justify)
                .layout(layout)
                .into_element(cx),
        ]
    })
}

pub(crate) fn container_hstack_centered<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_hstack(
        cx,
        props,
        HStackProps::default()
            .gap(gap)
            .layout(LayoutRefinement::default().w_full().h_full())
            .justify_center()
            .items_center(),
        children,
    )
}
