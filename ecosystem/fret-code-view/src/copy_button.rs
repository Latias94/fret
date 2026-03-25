use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    ClipboardToken, ClipboardWriteOutcome, Edges, FontId, FontWeight, Px, SemanticsRole,
    TextOverflow, TextStyle, TextWrap, TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PositionStyle, PressableProps,
    TextInkOverflow, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::typography;
use fret_ui_kit::{MetricRef, Space};

#[derive(Debug, Default)]
struct CopyFeedback {
    copied: bool,
    reset_token: Option<TimerToken>,
    pending_clipboard_token: Option<ClipboardToken>,
}

#[derive(Clone, Default)]
pub(crate) struct CopyFeedbackRef(Arc<Mutex<CopyFeedback>>);

impl CopyFeedbackRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, CopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn is_copied(&self) -> bool {
        self.lock().copied
    }
}

pub(crate) fn render_copy_button_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    feedback: CopyFeedbackRef,
    code: Arc<str>,
) -> AnyElement {
    let inset = MetricRef::space(Space::N1p5).resolve(theme);

    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.top = Some(inset).into();
    props.layout.inset.right = Some(inset).into();
    props.layout.size.width = Length::Auto;

    cx.container(props, |cx| {
        vec![render_copy_button(cx, theme, feedback, code)]
    })
}

pub(crate) fn render_copy_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    feedback: CopyFeedbackRef,
    code: Arc<str>,
) -> AnyElement {
    control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
        let copied = feedback.lock().copied;
        let label = if copied { "Copied" } else { "Copy" };

        let mut props = PressableProps::default();
        props.a11y.role = Some(SemanticsRole::Button);
        props.a11y.label = Some(Arc::<str>::from(label));
        props.focusable = false;

        cx.timer_on_timer_for(
            id,
            Arc::new({
                let feedback = feedback.clone();
                move |host, action_cx, token| {
                    let mut feedback = feedback.lock();
                    if feedback.reset_token != Some(token) {
                        return false;
                    }
                    feedback.reset_token = None;
                    feedback.copied = false;
                    host.notify(action_cx);
                    true
                }
            }),
        );

        cx.pressable_on_clipboard_write_completed({
            let feedback = feedback.clone();
            Arc::new(move |host, action_cx, token, outcome| {
                let prev_reset = {
                    let mut feedback = feedback.lock();
                    if feedback.pending_clipboard_token != Some(token) {
                        return false;
                    }
                    feedback.pending_clipboard_token = None;
                    match outcome {
                        ClipboardWriteOutcome::Succeeded => {
                            let prev_reset = feedback.reset_token.take();
                            let reset_token = host.next_timer_token();
                            feedback.copied = true;
                            feedback.reset_token = Some(reset_token);
                            Some((prev_reset, Some(reset_token)))
                        }
                        ClipboardWriteOutcome::Failed { .. } => {
                            let prev_reset = feedback.reset_token.take();
                            feedback.copied = false;
                            Some((prev_reset, None))
                        }
                    }
                };

                let Some((prev_reset, next_reset)) = prev_reset else {
                    return false;
                };

                if let Some(prev_reset) = prev_reset {
                    host.push_effect(Effect::CancelTimer { token: prev_reset });
                }
                if let Some(reset_token) = next_reset {
                    host.push_effect(Effect::SetTimer {
                        window: Some(action_cx.window),
                        token: reset_token,
                        after: Duration::from_secs(2),
                        repeat: None,
                    });
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            })
        });

        cx.pressable_on_activate({
            let code = code.clone();
            let feedback = feedback.clone();
            Arc::new(move |host, action_cx, _reason| {
                let (prev_reset, clipboard_token) = {
                    let mut feedback = feedback.lock();
                    if feedback.copied || feedback.pending_clipboard_token.is_some() {
                        return;
                    }
                    let prev_reset = feedback.reset_token.take();
                    let clipboard_token = host.next_clipboard_token();
                    feedback.pending_clipboard_token = Some(clipboard_token);
                    (prev_reset, clipboard_token)
                };

                if let Some(prev_reset) = prev_reset {
                    host.push_effect(Effect::CancelTimer { token: prev_reset });
                }
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token: clipboard_token,
                    text: code.to_string(),
                });
            })
        });

        // Align to shadcn/ui ghost button semantics:
        // - idle: transparent + muted-foreground
        // - hover/pressed: bg-accent + text-accent-foreground
        let bg_active = theme.color_token("accent");
        let radius_sm = theme.metric_token("metric.radius.sm");
        let font_size = theme.metric_token("metric.font.size");
        let line_height = theme.metric_token("metric.font.line_height");

        let (bg, fg) = if st.pressed || st.hovered {
            (Some(bg_active), theme.color_token("accent-foreground"))
        } else {
            (None, theme.color_token("muted-foreground"))
        };

        let pad_y = MetricRef::space(Space::N0p5).resolve(theme);
        let pad_x = MetricRef::space(Space::N1p5).resolve(theme);

        let chrome = ContainerProps {
            padding: Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            }
            .into(),
            corner_radii: fret_core::Corners::all(radius_sm),
            background: bg,
            border: Edges::all(Px(0.0)),
            ..Default::default()
        };

        (props, chrome, move |cx| {
            vec![cx.text_props(TextProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Auto;
                    layout
                },
                text: Arc::<str>::from(label),
                style: Some(typography::as_control_text(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    weight: FontWeight::SEMIBOLD,
                    slant: Default::default(),
                    line_height: Some(line_height),
                    letter_spacing_em: None,
                    ..Default::default()
                })),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: TextInkOverflow::None,
            })]
        })
    })
}
