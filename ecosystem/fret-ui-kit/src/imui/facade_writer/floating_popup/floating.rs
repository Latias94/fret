use super::*;

pub(in crate::imui::facade_writer) fn floating_layer<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let element = ui.with_cx_mut(|cx| floating_layer_element(cx, id, f));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn floating_area_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    initial_position: Point,
    options: FloatingAreaOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
) -> FloatingAreaResponse {
    let (element, response) =
        ui.with_cx_mut(|cx| floating_area_element(cx, id, initial_position, options, f));
    ui.add(element);
    response
}

pub(in crate::imui::facade_writer) fn floating_area_drag_surface<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    area: FloatingAreaContext,
    props: PointerRegionProps,
    setup: impl FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    ui.with_cx_mut(|cx| {
        floating_area_drag_surface_element(cx, area, props, None, true, true, setup, f)
    })
}
