pub const SOURCE: &str = include_str!("checkpoint_demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::action::OnActivate;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    visible_len: Option<Model<usize>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    const CHECKPOINT_MESSAGE_COUNT: usize = 2;

    #[derive(Clone, Copy)]
    struct DemoMessage {
        role: ui_ai::MessageRole,
        content: &'static str,
    }

    let messages: &[DemoMessage] = &[
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

    let needs_init = cx.with_state(DemoModels::default, |st| st.visible_len.is_none());
    if needs_init {
        let model = cx.app.models_mut().insert(messages.len());
        cx.with_state(DemoModels::default, |st| {
            st.visible_len = Some(model.clone())
        });
    }

    let visible_len_model = cx
        .with_state(DemoModels::default, |st| st.visible_len.clone())
        .expect("visible_len");
    let visible_len = cx
        .get_model_copied(&visible_len_model, Invalidation::Layout)
        .unwrap_or(messages.len())
        .min(messages.len());

    let restore: OnActivate = Arc::new({
        let visible_len_model = visible_len_model.clone();
        move |host, acx, _reason| {
            let _ = host
                .models_mut()
                .update(&visible_len_model, |v| *v = CHECKPOINT_MESSAGE_COUNT);
            host.request_redraw(acx.window);
        }
    });

    let reset: OnActivate = Arc::new({
        let visible_len_model = visible_len_model.clone();
        move |host, acx, _reason| {
            let _ = host
                .models_mut()
                .update(&visible_len_model, |v| *v = messages.len());
            host.request_redraw(acx.window);
        }
    });

    let restored_marker = (visible_len == CHECKPOINT_MESSAGE_COUNT)
        .then(|| {
            cx.text("restored=true")
                .test_id("ui-ai-checkpoint-restored-marker")
        })
        .unwrap_or_else(|| cx.text("restored=false"));

    let transcript = ui::v_flex(move |cx| {
        let mut out: Vec<AnyElement> = Vec::new();

        for (idx, msg) in messages.iter().enumerate().take(visible_len) {
            let message = ui_ai::Message::new(
                msg.role,
                [ui_ai::MessageContent::new(msg.role, [cx.text(msg.content)]).into_element(cx)],
            )
            .into_element(cx);
            out.push(message);

            if idx + 1 == CHECKPOINT_MESSAGE_COUNT {
                out.push(
                    ui_ai::Checkpoint::new([
                        ui_ai::CheckpointIcon::default().into_element(cx),
                        ui_ai::CheckpointTrigger::new([cx.text("Restore checkpoint")])
                            .tooltip("Restores workspace and chat to this point")
                            .tooltip_panel_test_id("ui-ai-checkpoint-tooltip-panel")
                            .test_id("ui-ai-checkpoint-trigger")
                            .on_activate(restore.clone())
                            .into_element(cx),
                    ])
                    .test_id("ui-ai-checkpoint-row")
                    .into_element(cx),
                );
            }
        }

        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx);

    let controls = ui::h_flex(move |cx| {
        vec![
            fret_ui_shadcn::Button::new("Reset")
                .test_id("ui-ai-checkpoint-reset")
                .on_activate(reset.clone())
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let conversation_props = cx.with_theme(|theme| {
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
                .h_px(Px(360.0))
                .min_w_0()
                .min_h_0(),
        )
    });

    let conversation = cx.container(conversation_props, move |_cx| vec![transcript]);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Checkpoint (AI Elements): restore a conversation to a prior state."),
            cx.text("Hover the trigger for tooltip; click to restore messages."),
            controls,
            restored_marker,
            conversation,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
