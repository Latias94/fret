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

pub(crate) fn container_vstack_build<H: UiHost, B>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: VStackProps,
    build: B,
) -> AnyElement
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    let VStackProps {
        gap,
        layout,
        items,
        justify,
    } = stack_props;

    cx.container(props, move |cx| {
        vec![
            ui::v_stack_build(build)
                .gap(gap)
                .items(items)
                .justify(justify)
                .layout(layout)
                .into_element(cx),
        ]
    })
}

pub(crate) fn container_vstack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: VStackProps,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_vstack_build(cx, props, stack_props, move |_cx, out| out.extend(children))
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

pub(crate) fn container_hstack_build<H: UiHost, B>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    stack_props: HStackProps,
    build: B,
) -> AnyElement
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    let HStackProps {
        gap,
        layout,
        items,
        justify,
    } = stack_props;

    cx.container(props, move |cx| {
        vec![
            ui::h_row_build(build)
                .gap(gap)
                .items(items)
                .justify(justify)
                .layout(layout)
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
    container_hstack_build(cx, props, stack_props, move |_cx, out| out.extend(children))
}

pub(crate) fn container_hstack_centered_build<H: UiHost, B>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    gap: Space,
    build: B,
) -> AnyElement
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    container_hstack_build(
        cx,
        props,
        HStackProps::default()
            .gap(gap)
            .layout(LayoutRefinement::default().w_full().h_full())
            .justify_center()
            .items_center(),
        build,
    )
}

pub(crate) fn container_hstack_centered<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ContainerProps,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    container_hstack_centered_build(cx, props, gap, move |_cx, out| out.extend(children))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Card, CardContent, CardHeader};
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui_kit::ui::UiElementSinkExt as _;

    fn contains_kind(el: &AnyElement, pred: &impl Fn(&ElementKind) -> bool) -> bool {
        pred(&el.kind) || el.children.iter().any(|child| contains_kind(child, pred))
    }

    #[test]
    fn container_vstack_build_accepts_host_bound_builders() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let container = container_vstack_build(
                cx,
                ContainerProps::default(),
                VStackProps::default().gap(Space::N2),
                |cx, out| {
                    out.push_ui(cx, CardHeader::build(|_cx, _out| {}));
                    out.push_ui(cx, CardContent::build(|_cx, _out| {}));
                },
            );

            assert!(contains_kind(&container, &|kind| matches!(
                kind,
                ElementKind::Flex(_)
            )));
            assert!(contains_kind(&container, &|kind| matches!(
                kind,
                ElementKind::Container(_)
            ),));
        });
    }

    #[test]
    fn container_hstack_centered_build_accepts_host_bound_builders() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let container = container_hstack_centered_build(
                cx,
                ContainerProps::default(),
                Space::N2,
                |cx, out| {
                    out.push_ui(cx, Card::build(|_cx, _out| {}));
                    out.push_ui(cx, CardHeader::build(|_cx, _out| {}));
                },
            );

            assert!(contains_kind(&container, &|kind| matches!(
                kind,
                ElementKind::Flex(_)
            )));
            assert!(contains_kind(&container, &|kind| matches!(
                kind,
                ElementKind::ForegroundScope(_)
            )));
            assert!(contains_kind(&container, &|kind| matches!(
                kind,
                ElementKind::Container(_)
            )));
        });
    }
}
