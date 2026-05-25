use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};

pub(in crate::imui) fn tooltip_text_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    text: Arc<str>,
    options: TooltipOptions,
) -> bool {
    super::tooltip_with_options(ui, id, trigger, options, move |ui| {
        let element = ui.with_cx_mut(|cx| tooltip_body_text(cx, text));
        ui.add(element);
    })
}

pub(super) fn tooltip_body_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    crate::declarative::text::text_compact_paragraph(cx, text)
}
