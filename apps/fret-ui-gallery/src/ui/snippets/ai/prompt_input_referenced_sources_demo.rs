pub const SOURCE: &str = include_str!("prompt_input_referenced_sources_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sources = cx.local_model_keyed("sources", Vec::<ui_ai::AttachmentSourceDocumentData>::new);

    let add = shadcn::Button::new("Add referenced source")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .test_id("ui-gallery-ai-prompt-input-referenced-sources-add")
        .on_activate(Arc::new({
            let sources = sources.clone();
            move |host, action_cx, _reason| {
                let src = ui_ai::AttachmentSourceDocumentData::new("src-0")
                    .title("ADR 0066: Runtime Contract Surface")
                    .filename("docs/adr/0066-fret-ui-runtime-contract-surface.md")
                    .url("https://example.com/adr/0066");
                let _ = host.models_mut().update(&sources, |v| {
                    if v.iter().any(|x| x.id.as_ref() == "src-0") {
                        return;
                    }
                    v.push(src);
                });
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let input = ui_ai::PromptInputRoot::new_uncontrolled()
        .referenced_sources_model(sources)
        .test_id_root("ui-gallery-ai-prompt-input-referenced-sources")
        .test_id_referenced_sources("ui-gallery-ai-prompt-input-referenced-sources")
        .into_element_with_slots(cx, move |cx| ui_ai::PromptInputSlots {
            block_start: vec![ui_ai::PromptInputReferencedSourcesRow::new().into_element(cx)],
            block_end: Vec::new(),
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Prompt Input Referenced Sources (AI Elements)"),
            cx.text("Add a source and remove it via the chip's hover affordance."),
            add,
            input,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
