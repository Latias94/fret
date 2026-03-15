pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{SemanticsRole, TimerToken};
use fret_runtime::Effect;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || 13.0);
    let timer_token = cx.local_model_keyed("timer_token", || None::<TimerToken>);

    let centered = |cx: &mut UiCx<'_>, body: AnyElement| {
        ui::h_flex(move |_cx| [body])
            .layout(LayoutRefinement::default().w_full())
            .justify_center()
            .into_element(cx)
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
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&timer_token, |v| *v = Some(token));
                    let _ = cx.app.models_mut().update(&value, |v| *v = 13.0);
                    cx.app.push_effect(Effect::SetTimer {
                        window: Some(cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });
                }

                let bar = shadcn::Progress::new(value.clone())
                    // shadcn/ui v4 demo: `className="w-[60%]"`
                    .refine_layout(LayoutRefinement::default().w_percent(60.0))
                    .into_element(cx)
                    .test_id("ui-gallery-progress-demo-bar");

                vec![centered(cx, bar)]
            },
        );

        body.test_id("ui-gallery-progress-demo")
    })
}

// endregion: example
