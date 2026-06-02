use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::super::containers::{horizontal_container_element, vertical_container_element};
use super::super::super::{
    HorizontalOptions, ImUiFacade, ItemFlowOptions, SameLineOptions, VerticalOptions,
};

pub(in crate::imui) fn items_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: ItemFlowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    vertical_container_element(
        cx,
        build_focus,
        VerticalOptions {
            layout: options.layout,
            gap: options.gap,
            justify: options.justify,
            items: options.items,
            wrap: options.wrap,
            test_id: options.test_id,
        },
        f,
    )
}

pub(in crate::imui) fn same_line_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: SameLineOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    horizontal_container_element(
        cx,
        build_focus,
        HorizontalOptions {
            layout: options.layout,
            gap: options.gap,
            justify: options.justify,
            items: options.items,
            wrap: options.wrap,
            test_id: options.test_id,
        },
        f,
    )
}
