use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

pub(crate) fn container_vstack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: stack::VStackProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.container(props, move |cx| {
        vec![stack::vstack(cx, stack_props, move |_cx| children)]
    })
}

pub(crate) fn container_vstack_gap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_vstack(cx, props, stack::VStackProps::default().gap(gap), children)
}

pub(crate) fn container_flow<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_vstack(
        cx,
        props,
        stack::VStackProps::default().gap(Space::N0),
        children,
    )
}

pub(crate) fn container_hstack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: stack::HStackProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.container(props, move |cx| {
        vec![stack::hstack(cx, stack_props, move |_cx| children)]
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
        stack::HStackProps::default()
            .gap(gap)
            .layout(LayoutRefinement::default().w_full().h_full())
            .justify_center()
            .items_center(),
        children,
    )
}
