use super::*;

pub(super) fn floating_layer<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let element = ui.with_cx_mut(|cx| floating_layer_element(cx, id, f));
    ui.add(element);
}

pub(super) fn floating_area_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

pub(super) fn floating_area_drag_surface<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

pub(super) fn popup_open_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> fret_runtime::Model<bool> {
    popup_overlay::popup_open_model(ui, id)
}

pub(super) fn drop_popup_scope<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    popup_overlay::drop_popup_scope(ui, id);
}

pub(super) fn open_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    popup_overlay::open_popup(ui, id);
}

pub(super) fn open_popup_at<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    anchor: fret_core::Rect,
) {
    popup_overlay::open_popup_at(ui, id, anchor);
}

pub(super) fn close_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    popup_overlay::close_popup(ui, id);
}

pub(super) fn begin_popup_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_menu_with_options(ui, id, trigger, options, false, f)
}

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_modal_with_options(ui, id, trigger, options, f)
}

pub(super) fn tooltip_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    text: impl Into<Arc<str>>,
    options: TooltipOptions,
) -> bool {
    tooltip_overlay::tooltip_text_with_options(ui, id, trigger, text.into(), options)
}

pub(super) fn tooltip_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: TooltipOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    tooltip_overlay::tooltip_with_options(ui, id, trigger, options, f)
}

pub(super) fn drag_source_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: std::any::Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: DragSourceOptions,
) -> DragSourceResponse {
    drag_drop::drag_source_with_options(ui, trigger, payload, options)
}

pub(super) fn drop_target_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: std::any::Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    options: DropTargetOptions,
) -> DropTargetResponse<T> {
    drag_drop::drop_target_with_options(ui, trigger, options)
}

pub(super) fn begin_popup_context_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_context_menu_with_options(ui, id, trigger, options, f)
}

pub(super) fn window<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window::floating_window_show(ui, id, title, initial_position, f)
}

pub(super) fn window_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    options: WindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window::floating_window_show_with_options(ui, id, title, initial_position, options, f)
}
