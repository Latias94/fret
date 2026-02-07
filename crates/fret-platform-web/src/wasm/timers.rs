use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use fret_core::{Event, TimerToken};
use wasm_bindgen::JsCast as _;

use super::WebWaker;

#[derive(Debug)]
pub(crate) struct WebTimer {
    pub(crate) id: i32,
    pub(crate) repeat: Option<Duration>,
    pub(crate) callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
}

pub(crate) fn ms(duration: Duration) -> i32 {
    let ms = duration.as_millis().min(i32::MAX as u128);
    i32::try_from(ms).unwrap_or(i32::MAX)
}

pub(crate) fn set_timer(
    token: TimerToken,
    after: Duration,
    repeat: Option<Duration>,
    fired_timeouts: &Rc<RefCell<Vec<TimerToken>>>,
    waker: &Option<WebWaker>,
    timers: &mut HashMap<TimerToken, WebTimer>,
) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let queue = fired_timeouts.clone();
    let wake = waker.clone();
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let _ = queue.try_borrow_mut().map(|mut q| q.push(token));
        if let Some(wake) = wake.as_ref() {
            wake();
        }
    }) as Box<dyn FnMut()>);

    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            ms(after),
        )
        .unwrap_or(0);

    timers.insert(
        token,
        WebTimer {
            id,
            repeat,
            callback,
        },
    );
}

pub(crate) fn cancel_timer(token: TimerToken, timers: &mut HashMap<TimerToken, WebTimer>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(timer) = timers.remove(&token) else {
        return;
    };
    window.clear_timeout_with_handle(timer.id);
}

pub(crate) fn collect_fired_timeouts(
    queued_events: &Rc<RefCell<Vec<Event>>>,
    fired_timeouts: &Rc<RefCell<Vec<TimerToken>>>,
    timers: &mut HashMap<TimerToken, WebTimer>,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let fired = std::mem::take(&mut *fired_timeouts.borrow_mut());
    for token in fired {
        let Some(timer) = timers.remove(&token) else {
            continue;
        };

        queued_events.borrow_mut().push(Event::Timer { token });
        window.clear_timeout_with_handle(timer.id);

        let Some(repeat) = timer.repeat else {
            continue;
        };

        let id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timer.callback.as_ref().unchecked_ref(),
                ms(repeat),
            )
            .unwrap_or(0);
        timers.insert(
            token,
            WebTimer {
                id,
                repeat: Some(repeat),
                callback: timer.callback,
            },
        );
    }
}
