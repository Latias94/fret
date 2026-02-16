use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_prompt_input_referenced_sources_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};
    use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

    #[derive(Default)]
    struct DemoModels {
        sources: Option<Model<Vec<ui_ai::AttachmentSourceDocumentData>>>,
    }

    let sources = cx.with_state(DemoModels::default, |st| st.sources.clone());
    let sources = match sources {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Vec::<ui_ai::AttachmentSourceDocumentData>::new());
            cx.with_state(DemoModels::default, |st| st.sources = Some(model.clone()));
            model
        }
    };

    let add = Button::new("Add referenced source")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
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

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Prompt Input Referenced Sources (AI Elements)"),
                cx.text("Add a source and remove it via the chip's hover affordance."),
                add,
                input,
            ]
        },
    )]
}
