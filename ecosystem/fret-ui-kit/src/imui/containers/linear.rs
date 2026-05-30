use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{HorizontalOptions, ImUiFacade, VerticalOptions};
use super::children::build_imui_children_with_focus;

pub(in crate::imui) fn horizontal_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut builder = crate::ui::h_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}

pub(in crate::imui) fn vertical_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut builder = crate::ui::v_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}
