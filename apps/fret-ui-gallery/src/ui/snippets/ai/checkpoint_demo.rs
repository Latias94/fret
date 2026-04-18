pub const SOURCE: &str = include_str!("checkpoint_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Clone, Copy)]
struct DemoMessage {
    role: ui_ai::MessageRole,
    content: &'static str,
}

#[derive(Clone, Copy)]
struct DemoCheckpoint {
    message_count: usize,
    trigger_label: &'static str,
    tooltip: &'static str,
}

const INITIAL_MESSAGES: &[DemoMessage] = &[
    DemoMessage {
        role: ui_ai::MessageRole::User,
        content: "What is React?",
    },
    DemoMessage {
        role: ui_ai::MessageRole::Assistant,
        content: "React is a JavaScript library for building user interfaces. It was developed by Facebook and is now maintained by Meta and a community of developers.",
    },
    DemoMessage {
        role: ui_ai::MessageRole::User,
        content: "How does component state work?",
    },
];

const INITIAL_CHECKPOINTS: &[DemoCheckpoint] = &[DemoCheckpoint {
    message_count: 2,
    trigger_label: "Restore checkpoint",
    tooltip: "Restores workspace and chat to this point",
}];

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let visible_message_count_model =
        cx.local_model_keyed("visible_message_count", || INITIAL_MESSAGES.len());
    let visible_message_count = cx
        .get_model_copied(&visible_message_count_model, Invalidation::Layout)
        .unwrap_or(INITIAL_MESSAGES.len())
        .min(INITIAL_MESSAGES.len());

    let restore_to_checkpoint = {
        let visible_message_count_model = visible_message_count_model.clone();
        move |host: &mut dyn UiActionHost, acx: ActionCx| {
            let _ = host
                .models_mut()
                .update(&visible_message_count_model, |count| {
                    *count = INITIAL_CHECKPOINTS[0].message_count;
                });
            host.request_redraw(acx.window);
        }
    };

    let reset_demo = {
        let visible_message_count_model = visible_message_count_model.clone();
        move |host: &mut dyn UiActionHost, acx: ActionCx| {
            let _ = host
                .models_mut()
                .update(&visible_message_count_model, |count| {
                    *count = INITIAL_MESSAGES.len();
                });
            host.request_redraw(acx.window);
        }
    };

    let restored_marker = (visible_message_count == INITIAL_CHECKPOINTS[0].message_count)
        .then(|| {
            cx.text("Preview restored to the checkpoint.")
                .test_id("ui-ai-checkpoint-restored-marker")
        })
        .unwrap_or_else(|| cx.text("The preview currently shows the latest conversation state."));

    let conversation = ui_ai::Conversation::new([])
        .content_revision(visible_message_count as u64)
        .test_id("ui-ai-checkpoint-conversation")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0(),
        )
        .into_element_with_children(cx, move |cx| {
            let mut content_children: Vec<AnyElement> = Vec::new();

            for (idx, message) in INITIAL_MESSAGES
                .iter()
                .enumerate()
                .take(visible_message_count)
            {
                let bubble = ui_ai::Message::new(
                    message.role,
                    [
                        ui_ai::MessageContent::new(message.role, [cx.text(message.content)])
                            .into_element(cx),
                    ],
                )
                .into_element(cx);
                content_children.push(bubble);

                if let Some(checkpoint) = INITIAL_CHECKPOINTS
                    .iter()
                    .find(|checkpoint| checkpoint.message_count == idx + 1)
                    .copied()
                {
                    content_children.push(
                        ui_ai::Checkpoint::new([
                            ui_ai::CheckpointIcon::default().into_element(cx),
                            ui_ai::CheckpointTrigger::new([cx.text(checkpoint.trigger_label)])
                                .tooltip(checkpoint.tooltip)
                                .tooltip_panel_test_id("ui-ai-checkpoint-tooltip-panel")
                                .test_id("ui-ai-checkpoint-trigger")
                                .on_activate(cx.actions().listen(restore_to_checkpoint.clone()))
                                .into_element(cx),
                        ])
                        .test_id("ui-ai-checkpoint-row")
                        .into_element(cx),
                    );
                }
            }

            vec![ui_ai::ConversationContent::new(content_children).into_element(cx)]
        });

    let conversation_shell_props = cx.with_theme(|theme| {
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
                .max_w(Px(896.0))
                .h_px(Px(520.0))
                .min_w_0()
                .min_h_0(),
        )
    });
    let conversation_shell = cx.container(conversation_shell_props, move |_cx| vec![conversation]);
    let conversation_shell = ui::h_flex(move |_cx| vec![conversation_shell])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .items_center()
        .into_element(cx);

    let controls = ui::h_flex(move |cx| {
        vec![
            shadcn::Button::new("Reset preview")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .test_id("ui-ai-checkpoint-reset")
                .on_activate(cx.actions().listen(reset_demo.clone()))
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .justify_end()
    .items_center()
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state.",
            ),
            cx.text(
                "Docs-aligned composition: `Conversation` + `Message` + `Checkpoint`. Hover the trigger to preview the tooltip, then activate it to restore the conversation.",
            ),
            controls,
            restored_marker,
            conversation_shell,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example

// region: custom_icon
#[allow(dead_code)]
fn custom_checkpoint_icon(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    fret_ui_ai::CheckpointIcon::default()
        .into_element_with_children(cx, |cx| vec![cx.text("⟲"), cx.text("•")])
}
// endregion: custom_icon

// region: manual_checkpoints
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct ManualCheckpoint {
    message_count: usize,
}

#[allow(dead_code)]
fn create_checkpoint(checkpoints: &mut Vec<ManualCheckpoint>, message_count: usize) {
    checkpoints.push(ManualCheckpoint { message_count });
}
// endregion: manual_checkpoints

// region: automatic_checkpoints
#[allow(dead_code)]
fn maybe_create_automatic_checkpoint(
    checkpoints: &mut Vec<ManualCheckpoint>,
    message_count: usize,
) {
    if message_count > 0 && message_count % 5 == 0 {
        create_checkpoint(checkpoints, message_count);
    }
}
// endregion: automatic_checkpoints

// region: branching_conversations
#[allow(dead_code)]
fn restore_and_branch<T: Clone>(messages: &[T], message_count: usize) -> (Vec<T>, Vec<T>) {
    let restored = messages[..message_count].to_vec();
    let branch = messages[message_count..].to_vec();
    (restored, branch)
}
// endregion: branching_conversations
