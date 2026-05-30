use fret_core::{Px, Size};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::containers::{horizontal_container_element, vertical_container_element};
use super::super::{
    DummyOptions, HorizontalOptions, ImUiFacade, IndentOptions, ItemFlowOptions, SameLineOptions,
    VerticalOptions,
};
use super::spacers::dummy_element;

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

pub(in crate::imui) fn indent_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: IndentOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let indent_width = options.width.resolve(&*theme);
    let spacer = dummy_element(
        cx,
        Size::new(indent_width, Px(0.0)),
        DummyOptions::default(),
    );
    let content = items_element(
        cx,
        build_focus,
        ItemFlowOptions {
            test_id: options.content_test_id,
            ..Default::default()
        },
        f,
    );

    horizontal_container_element(
        cx,
        None,
        HorizontalOptions {
            gap: Px(0.0).into(),
            items: crate::Items::Start,
            test_id: options.test_id,
            ..Default::default()
        },
        move |ui| {
            ui.add(spacer);
            ui.add(content);
        },
    )
}
