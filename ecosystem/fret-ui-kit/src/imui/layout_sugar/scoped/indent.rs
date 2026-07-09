use fret_core::{Px, Size};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::super::containers::horizontal_container_element;
use super::super::super::{
    DummyOptions, HorizontalOptions, ImUiFacade, IndentOptions, ItemFlowOptions,
};
use super::super::spacers::dummy_element;
use super::flow::items_element;

pub(in crate::imui) fn indent_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<std::rc::Rc<std::cell::Cell<Option<fret_ui::GlobalElementId>>>>,
    options: IndentOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let indent_width = options.width.resolve(theme);
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
