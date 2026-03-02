pub const SOURCE: &str = include_str!("prompt_input_action_menu_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    attachments: Option<Model<Vec<ui_ai::AttachmentData>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let attachments = cx.with_state(DemoModels::default, |st| st.attachments.clone());
    let attachments = match attachments {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Vec::<ui_ai::AttachmentData>::new());
            cx.with_state(DemoModels::default, |st| {
                st.attachments = Some(model.clone())
            });
            model
        }
    };

    let on_add_attachments: fret_ui::action::OnActivate = Arc::new({
        let attachments = attachments.clone();
        move |host, action_cx, _reason| {
            let file = ui_ai::AttachmentFileData::new("att-0")
                .filename("design.png")
                .media_type("image/png")
                .size_bytes(42_000);
            let item = ui_ai::AttachmentData::File(file);
            let _ = host.models_mut().update(&attachments, |v| {
                if v.iter().any(|x| x.id().as_ref() == "att-0") {
                    return;
                }
                v.push(item);
            });
            host.notify(action_cx);
        }
    });

    let input = ui_ai::PromptInputRoot::new_uncontrolled()
        .attachments(attachments)
        .on_add_attachments(on_add_attachments)
        .test_id_root("ui-gallery-ai-prompt-input-action-menu")
        .test_id_attachments("ui-gallery-ai-prompt-input-action-menu-attachments")
        .into_element_with_slots(cx, move |cx| {
            let menu =
                ui_ai::PromptInputActionMenu::new(ui_ai::PromptInputActionMenuContent::new([
                    ui_ai::PromptInputActionAddAttachments::new()
                        .test_id("ui-gallery-ai-prompt-input-action-menu-add-attachments-item")
                        .into_entry(cx),
                ]))
                .trigger(
                    ui_ai::PromptInputActionMenuTrigger::new()
                        .test_id("ui-gallery-ai-prompt-input-action-menu-trigger"),
                )
                .into_element(cx);

            ui_ai::PromptInputSlots {
                block_start: vec![ui_ai::PromptInputAttachmentsRow::new().into_element(cx)],
                block_end: vec![menu],
            }
        });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Prompt Input Action Menu (AI Elements)"),
                cx.text("Use the + menu to add attachments."),
                input,
            ]
        },
    )
}
// endregion: example
