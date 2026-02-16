use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_message_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::declarative::icon as decl_icon;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{Justify, LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        last_action: Option<Model<Option<Arc<str>>>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| st.last_action.is_none());
    if needs_init {
        let model = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, |st| {
            st.last_action = Some(model.clone())
        });
    }

    let last_action_model = cx
        .with_state(DemoModels::default, |st| st.last_action.clone())
        .expect("last_action");
    let last_action = cx
        .get_model_cloned(&last_action_model, Invalidation::Paint)
        .flatten()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<none>".to_string());

    let marker = cx
        .text(format!("last_action={last_action}"))
        .test_id("ui-ai-message-demo-last-action");

    let set_action = |label: &'static str| -> OnActivate {
        let last_action_model = last_action_model.clone();
        Arc::new(move |host, acx, _reason| {
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Some(Arc::<str>::from(label));
            });
            host.request_redraw(acx.window);
        })
    };

    let assistant_actions = ui_ai::MessageActions::new([
        ui_ai::MessageAction::new("Copy")
            .tooltip("Copy message")
            .children([decl_icon::icon(cx, fret_icons::ids::ui::COPY)])
            .test_id("ui-ai-message-demo-assistant-action-copy")
            .on_activate(set_action("assistant.copy"))
            .into_element(cx),
        ui_ai::MessageAction::new("Regenerate")
            .tooltip("Regenerate")
            .children([decl_icon::icon(cx, fret_icons::ids::ui::LOADER)])
            .test_id("ui-ai-message-demo-assistant-action-regenerate")
            .on_activate(set_action("assistant.regenerate"))
            .into_element(cx),
    ])
    .justify(Justify::Start)
    .test_id("ui-ai-message-demo-assistant-actions")
    .into_element(cx);

    let user_actions = ui_ai::MessageActions::new([ui_ai::MessageAction::new("Edit")
        .tooltip("Edit message")
        .children([decl_icon::icon(cx, fret_icons::ids::ui::FILE)])
        .test_id("ui-ai-message-demo-user-action-edit")
        .on_activate(set_action("user.edit"))
        .into_element(cx)])
    .justify(Justify::End)
    .test_id("ui-ai-message-demo-user-actions")
    .into_element(cx);

    let assistant = ui_ai::Message::new(
        ui_ai::MessageRole::Assistant,
        [
            ui_ai::MessageContent::new(
                ui_ai::MessageRole::Assistant,
                [
                    cx.text("Assistant messages default to a full-width flow (no bubble)."),
                    cx.text("Compose actions/toolbar slots as separate children."),
                ],
            )
            .test_id("ui-ai-message-demo-assistant-content")
            .into_element(cx),
            assistant_actions,
        ],
    )
    .test_id("ui-ai-message-demo-assistant")
    .into_element(cx);

    let user = ui_ai::Message::new(
        ui_ai::MessageRole::User,
        [
            ui_ai::MessageContent::new(
                ui_ai::MessageRole::User,
                [
                    cx.text("User messages render as a bubble aligned to the right."),
                    cx.text("Bubble chrome is controlled by theme tokens."),
                ],
            )
            .test_id("ui-ai-message-demo-user-content")
            .into_element(cx),
            user_actions,
        ],
    )
    .test_id("ui-ai-message-demo-user")
    .into_element(cx);

    let title = cx.text("Message (AI Elements): alignment + bubble + actions rows.");

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |_cx| vec![title, marker, assistant, user],
    )]
}
