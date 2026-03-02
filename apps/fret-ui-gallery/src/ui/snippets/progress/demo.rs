// region: example
use fret_app::App;
use fret_core::{Px, SemanticsRole, TimerToken};
use fret_runtime::Effect;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

#[derive(Default, Clone)]
struct ProgressModels {
    value: Option<Model<f32>>,
    timer_token: Option<Model<Option<TimerToken>>>,
}

fn ensure_models(cx: &mut ElementContext<'_, App>) -> (Model<f32>, Model<Option<TimerToken>>) {
    let state = cx.with_state(ProgressModels::default, |st| st.clone());
    match (state.value, state.timer_token) {
        (Some(value), Some(token)) => (value, token),
        _ => {
            let models = cx.app.models_mut();
            let value = models.insert(13.0);
            let token = models.insert(None::<TimerToken>);
            cx.with_state(ProgressModels::default, |st| {
                st.value = Some(value.clone());
                st.timer_token = Some(token.clone());
            });
            (value, token)
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (value, timer_token) = ensure_models(cx);

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    cx.keyed("ui_gallery.progress.demo", |cx| {
        let value_for_timer = value.clone();
        let token_for_timer = timer_token.clone();

        let body = cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-progress-demo")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }
                        let _ = host.models_mut().update(&value_for_timer, |v| *v = 66.0);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let armed = cx
                    .get_model_copied(&timer_token, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                if !armed {
                    let token = cx.app.next_timer_token();
                    let _ = cx.app.models_mut().update(&timer_token, |v| *v = Some(token));
                    let _ = cx.app.models_mut().update(&value, |v| *v = 13.0);
                    cx.app.push_effect(Effect::SetTimer {
                        window: Some(cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });
                }

                let bar = shadcn::Progress::new(value.clone())
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx)
                    .test_id("ui-gallery-progress-demo-bar");

                vec![centered(cx, bar)]
            },
        );

        body.test_id("ui-gallery-progress-demo")
    })
}

// endregion: example
