use std::sync::Arc;

use fret::imui::prelude::*;

use super::super::{KernelApp, proof_compact_readout_element, proof_section_chrome_label};

pub(in super::super) fn proof_collection_readout_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
    test_id: &'static str,
) {
    let element =
        ui.with_cx_mut(|cx| proof_compact_readout_element(cx, text, Arc::<str>::from(test_id)));
    ui.add(element);
}

pub(super) fn proof_collection_section_label(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: &'static str,
    test_id: &'static str,
) {
    let element = ui.with_cx_mut(|cx| proof_section_chrome_label(cx, text, test_id));
    ui.add(element);
}
