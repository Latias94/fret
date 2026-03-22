pub const SOURCE: &str = include_str!("prompt_input_docs_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

mod act {
    fret::actions!([ToggleSearch = "ui-gallery.ai.prompt_input_docs.toggle_search.v1"]);
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let text = cx.local_model_keyed("text", String::new);
    let attachments = cx.local_model_keyed("attachments", Vec::<ui_ai::AttachmentData>::new);
    let use_web_search = cx.local_model_keyed("use_web_search", || false);
    let model_value = cx.local_model_keyed("model_value", || Some(Arc::<str>::from("gpt-4o")));
    let model_open = cx.local_model_keyed("model_open", || false);

    cx.actions().models::<act::ToggleSearch>({
        let use_web_search = use_web_search.clone();
        move |models| {
            models
                .update(&use_web_search, |value| *value = !*value)
                .is_ok()
        }
    });

    let on_send: fret_ui::action::OnActivate = Arc::new(|host, action_cx, _reason| {
        host.notify(action_cx);
    });

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

    let body = ui_ai::PromptInputProvider::new()
        .text_model(text)
        .attachments_model(attachments)
        .into_element_with_children(cx, move |cx, controller| {
            let menu =
                ui_ai::PromptInputActionMenu::new(ui_ai::PromptInputActionMenuContent::new([
                    ui_ai::PromptInputActionAddAttachments::new()
                        .test_id("ui-gallery-ai-prompt-input-docs-add-attachments-item")
                        .into_entry(cx),
                ]))
                .trigger(
                    ui_ai::PromptInputActionMenuTrigger::new()
                        .test_id("ui-gallery-ai-prompt-input-docs-action-menu-trigger"),
                )
                .into_element(cx);

            let searching = cx
                .get_model_cloned(&use_web_search, fret_ui::Invalidation::Layout)
                .unwrap_or(false);
            let search_btn = ui_ai::PromptInputButton::new("Search")
                .children([
                    decl_icon::icon(cx, IconId::new("lucide.globe")),
                    ui::text("Search").into_element(cx),
                ])
                .tooltip(
                    ui_ai::PromptInputButtonTooltip::new("Search the web")
                        .shortcut("⌘K")
                        .panel_test_id("ui-gallery-ai-prompt-input-docs-search-tooltip-panel"),
                )
                .variant(if searching {
                    shadcn::ButtonVariant::Default
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .test_id("ui-gallery-ai-prompt-input-docs-search")
                .action(act::ToggleSearch)
                .into_element(cx);

            let select = ui_ai::PromptInputSelect::new(model_value.clone(), model_open.clone())
                .trigger_test_id("ui-gallery-ai-prompt-input-docs-model-trigger")
                .on_value_change({
                    let model_value = model_value.clone();
                    move |host, _action_cx, value| {
                        let _ = host.models_mut().update(&model_value, |v| *v = Some(value));
                    }
                })
                .trigger(ui_ai::PromptInputSelectTrigger::new().into())
                .value(ui_ai::PromptInputSelectValue::new().placeholder("Model"))
                .content(ui_ai::PromptInputSelectContent::new())
                .entries([
                    ui_ai::PromptInputSelectItem::new("gpt-4o", "GPT-4o").into(),
                    ui_ai::PromptInputSelectItem::new("claude-opus-4-20250514", "Claude 4 Opus")
                        .into(),
                ])
                .into_element(cx);

            let input =
                ui_ai::PromptInput::new(controller.text)
                    .attachments(controller.attachments.expect("provider sets attachments"))
                    .on_send(on_send)
                    .on_add_attachments(on_add_attachments)
                    .test_id_root("ui-gallery-ai-prompt-input-docs")
                    .test_id_send("ui-gallery-ai-prompt-input-docs-send")
                    .test_id_stop("ui-gallery-ai-prompt-input-docs-stop")
                    .children([
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputHeader::new([
                            ui_ai::PromptInputAttachmentsRow::new().into_element(cx),
                        ])),
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputBody::new([
                            ui_ai::PromptInputTextarea::new()
                                .placeholder("What would you like to know?")
                                .test_id("ui-gallery-ai-prompt-input-docs-textarea"),
                        ])),
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputFooter::new(
                            [ui_ai::PromptInputTools::new([menu, search_btn, select])
                                .into_element(cx)],
                            [ui_ai::PromptInputSubmit::new()
                                .refine_layout(LayoutRefinement::default().ml_auto())
                                .into_element(cx)],
                        )),
                    ])
                    .into_element(cx);

            vec![input]
        });

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N6);
        decl_style::container_props(
            theme,
            chrome,
            LayoutRefinement::default()
                .w_full()
                .h_px(Px(320.0))
                .min_w_0()
                .min_h_0(),
        )
    });

    let frame = cx.container(props, move |_cx| vec![body]);

    ui::v_flex(move |cx| {
            vec![
                cx.text("Prompt Input (AI Elements)"),
                cx.text("Docs-aligned children composition: header + body textarea + footer tools/model picker/submit."),
                frame,
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4).into_element(cx)
}
// endregion: example
