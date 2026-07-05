use std::sync::Arc;

use fret::imui::{UiWriter as _, UiWriterImUiFacadeExt};

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

pub(super) fn render_collection_header(ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized)) {
    proof_collection_section_label(
        ui,
        "Collection-first asset browser proof",
        "imui-editor-proof.authoring.imui.collection.title",
    );
    ui.text_wrapped(
        "Stable keys keep browser selection pinned while visible order flips and selected-set drag/drop stays app-defined.",
    );
    ui.text_wrapped(
        "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
    );
}

pub(super) fn proof_collection_section_label(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: &'static str,
    test_id: &'static str,
) {
    let element = ui.with_cx_mut(|cx| proof_section_chrome_label(cx, text, test_id));
    ui.add(element);
}
