use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap, TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PositionStyle, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{MetricRef, Space};

#[derive(Debug, Default)]
struct CopyFeedback {
    copied: bool,
    token: Option<TimerToken>,
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
    props.layout.inset.top = Some(inset);
    props.layout.inset.right = Some(inset);
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
    cx.pressable_with_id_props(move |cx, st, id| {
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
                    if feedback.token != Some(token) {
                        return false;
                    }
                    feedback.token = None;
                    feedback.copied = false;
                    host.notify(action_cx);
                    true
                }
            }),
        );

        cx.pressable_on_activate({
            let code = code.clone();
            let feedback = feedback.clone();
            Arc::new(move |host, action_cx, _reason| {
                host.push_effect(Effect::ClipboardSetText {
                    text: code.to_string(),
                });

                let (prev, token) = {
                    let mut feedback = feedback.lock();
                    let prev = feedback.token.take();
                    let token = host.next_timer_token();
                    feedback.copied = true;
                    feedback.token = Some(token);
                    (prev, token)
                };

                if let Some(prev) = prev {
                    host.push_effect(Effect::CancelTimer { token: prev });
                }
                host.push_effect(Effect::SetTimer {
                    window: Some(action_cx.window),
                    token,
                    after: Duration::from_secs(2),
                    repeat: None,
                });
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            })
        });

        let bg_pressed = theme.color_required("accent");
        let bg_hover = theme.color_required("color.menu.item.hover");
        let bg_idle = theme.color_required("secondary");
        let radius_sm = theme.metric_required("metric.radius.sm");
        let font_size = theme.metric_required("metric.font.size");
        let line_height = theme.metric_required("metric.font.line_height");
        let fg = theme.color_required("foreground");

        let bg = if st.pressed {
            bg_pressed
        } else if st.hovered {
            bg_hover
        } else {
            bg_idle
        };

        let pad_y = MetricRef::space(Space::N0p5).resolve(theme);
        let pad_x = MetricRef::space(Space::N1p5).resolve(theme);

        let container = ContainerProps {
            padding: Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            },
            corner_radii: fret_core::Corners::all(radius_sm),
            background: Some(bg),
            border: Edges::all(Px(0.0)),
            ..Default::default()
        };

        (
            props,
            vec![cx.container(container, |cx| {
                vec![cx.text_props(TextProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Auto;
                        layout
                    },
                    text: Arc::<str>::from(label),
                    style: Some(TextStyle {
                        font: FontId::default(),
                        size: font_size,
                        weight: FontWeight::SEMIBOLD,
                        slant: Default::default(),
                        line_height: Some(line_height),
                        letter_spacing_em: None,
                    }),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            })],
        )
    })
}
