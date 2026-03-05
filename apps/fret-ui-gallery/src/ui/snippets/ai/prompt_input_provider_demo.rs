pub const SOURCE: &str = include_str!("prompt_input_provider_demo.rs");

// region: example
use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{ContainerProps, PressableA11y, PressableProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    text: Option<Model<String>>,
    attachments: Option<Model<Vec<ui_ai::AttachmentData>>>,
    sent_count: Option<Model<u32>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text = cx.with_state(DemoModels::default, |st| st.text.clone());
    let text = match text {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(DemoModels::default, |st| st.text = Some(model.clone()));
            model
        }
    };

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

    let sent_count = cx.with_state(DemoModels::default, |st| st.sent_count.clone());
    let sent_count = match sent_count {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0u32);
            cx.with_state(DemoModels::default, |st| {
                st.sent_count = Some(model.clone())
            });
            model
        }
    };

    let on_send: fret_ui::action::OnActivate = Arc::new({
        let sent_count = sent_count.clone();
        move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&sent_count, |v| *v += 1);
            host.notify(action_cx);
        }
    });

    let add_external_activate: fret_ui::action::OnActivate = Arc::new({
        let attachments = attachments.clone();
        move |host, action_cx, _reason| {
            let file = ui_ai::AttachmentFileData::new("ext-att-0")
                .filename("spec.md")
                .media_type("text/markdown")
                .size_bytes(12_345);
            let item = ui_ai::AttachmentData::File(file);
            let _ = host.models_mut().update(&attachments, |v| {
                if v.iter().any(|x| x.id().as_ref() == "ext-att-0") {
                    return;
                }
                v.push(item);
            });
            host.notify(action_cx);
        }
    });

    let add_external_label: Arc<str> = Arc::from("Add external attachment");

    let body = ui_ai::PromptInputProvider::new()
        .text_model(text)
        .attachments_model(attachments)
        .into_element_with_children(cx, move |cx, controller| {
            let add_external = cx.pressable(
                PressableProps {
                    layout: Default::default(),
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(add_external_label.clone()),
                        test_id: Some(Arc::from(
                            "ui-gallery-ai-prompt-input-provider-external-add",
                        )),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    cx.pressable_on_activate(add_external_activate.clone());

                    let theme = Theme::global(&*cx.app).clone();
                    let bg = if st.hovered {
                        theme
                            .color_by_key("muted")
                            .unwrap_or_else(|| theme.color_token("secondary"))
                    } else {
                        theme.color_token("secondary")
                    };
                    let fg = if st.hovered {
                        theme.color_token("foreground")
                    } else {
                        theme.color_token("secondary-foreground")
                    };

                    let mut props = ContainerProps::default();
                    props.padding = Edges::symmetric(Px(12.0), Px(6.0)).into();
                    props.background = Some(bg);
                    props.corner_radii = Corners::all(theme.metric_token("metric.radius.sm"));

                    vec![cx.container(props, move |cx| {
                        vec![
                            ui::text(add_external_label.clone())
                                .text_color(ColorRef::Color(fg))
                                .into_element(cx),
                        ]
                    })]
                },
            );

            let sent_marker = cx
                .app
                .models_mut()
                .read(&sent_count, |v| *v)
                .ok()
                .and_then(|n| (n == 1).then_some(()))
                .map(|_| {
                    cx.text("")
                        .test_id("ui-gallery-ai-prompt-input-provider-sent-count-1")
                });

            vec![
                ui::v_stack(move |cx| {
                    let mut children = Vec::new();
                    children.push(add_external);
                    children.push(
                        ui_ai::PromptInput::new(controller.text)
                            .on_send(on_send)
                            .test_id_root("ui-gallery-ai-prompt-input-provider")
                            .test_id_textarea("ui-gallery-ai-prompt-input-provider-textarea")
                            .test_id_send("ui-gallery-ai-prompt-input-provider-send")
                            .test_id_attachments("ui-gallery-ai-prompt-input-provider-attachments")
                            .into_element(cx),
                    );
                    if let Some(marker) = sent_marker {
                        children.push(marker);
                    }
                    children
                })
                .gap(Space::N4)
                .into_element(cx),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Prompt Input Provider (AI Elements)"),
            cx.text("External add mutates the provider attachments; send clears attachments."),
            body,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
