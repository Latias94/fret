use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::{Effect, FrameId, Model, PlatformCapabilities, TickId};
use fret_ui_app::declarative;
use fret_ui_app::element::{ContainerProps, LayoutStyle, Length, PressableProps};
use fret_ui_app::{Invalidation, Theme, UiFrameCx, UiTree};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::Window;

struct WebDemo {
    runner: fret_runner_web::WebRunner,
    input: fret_runner_web::WebInput,
    effects: fret_runner_web::WebEffectPump,
    app: fret_ui_app::App,
    ui: UiTree,
    window: AppWindowId,
    tick_id: TickId,
    frame_id: FrameId,
    counter: Model<u32>,
}

impl WebDemo {
    async fn new(canvas_id: &str, prevent_default: bool) -> Result<Self, JsValue> {
        let canvas = fret_runner_web::canvas_by_id(canvas_id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let effects = fret_runner_web::WebEffectPump::new(canvas.clone());
        let runner = fret_runner_web::WebRunner::new(canvas.clone())
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let input = fret_runner_web::WebInput::new(canvas.clone(), prevent_default)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut app = fret_ui_app::App::new();
        app.set_global(PlatformCapabilities::default());

        let window = AppWindowId::default();
        let mut ui = UiTree::new();
        ui.set_window(window);

        let counter = app.models_mut().insert(0u32);

        Ok(Self {
            runner,
            input,
            effects,
            app,
            ui,
            window,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            counter,
        })
    }

    fn drain_effects(&mut self) {
        const MAX_TURNS: usize = 8;
        for _ in 0..MAX_TURNS {
            let effects = self.app.flush_effects();
            if effects.is_empty() {
                break;
            }
            let unhandled = self.effects.handle_effects(
                &mut self.app,
                &mut self.runner,
                &mut self.input,
                self.window,
                effects,
            );

            for effect in unhandled {
                match effect {
                    Effect::Command { command, .. } => {
                        let services = self.runner.services_mut();
                        let _ = self.ui.dispatch_command(&mut self.app, services, &command);
                    }
                    _ => {}
                }
            }
        }
    }

    fn tick(&mut self) {
        self.effects.tick();

        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.frame_id.0 = self.frame_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.app.set_frame_id(self.frame_id);

        let (bounds, scale_factor) = match self.runner.ui_bounds_and_scale() {
            Ok(v) => v,
            Err(_) => {
                return;
            }
        };

        let platform_events = self.effects.take_events();
        {
            let (services, scene) = self.runner.services_and_scene_mut();

            for event in platform_events {
                self.ui.dispatch_event(&mut self.app, services, &event);
            }

            for event in self.input.take_events() {
                self.ui.dispatch_event(&mut self.app, services, &event);
            }

            let changed_models = self.app.take_changed_models();
            let _ = self
                .ui
                .propagate_model_changes(&mut self.app, &changed_models);
            let changed_globals = self.app.take_changed_globals();
            let _ = self
                .ui
                .propagate_global_changes(&mut self.app, &changed_globals);

            let counter = self.counter.clone();
            let root = declarative::RenderRootContext::new(
                &mut self.ui,
                &mut self.app,
                services,
                self.window,
                bounds,
            )
            .render_root("demo-web", move |cx| {
                cx.observe_model(&counter, Invalidation::Paint);

                let theme = Theme::global(&*cx.app).clone();

                let mut fill = LayoutStyle::default();
                fill.size.width = Length::Fill;
                fill.size.height = Length::Fill;

                let counter_value = cx.app.models().read(&counter, |v| *v).unwrap_or(0);

                let label: Arc<str> = Arc::from(format!(
                    "fret web demo — clicks: {counter_value} (click or press Enter)",
                ));

                vec![cx.container(
                    ContainerProps {
                        layout: fill,
                        background: Some(theme.colors.surface_background),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.pressable(PressableProps::default(), |cx, _state| {
                            let counter = counter.clone();
                            cx.pressable_on_activate(Arc::new(move |host, _acx, _reason| {
                                let _ = host.models_mut().update(&counter, |v| {
                                    *v = v.saturating_add(1);
                                });
                            }));
                            vec![cx.text(label.clone())]
                        })]
                    },
                )]
            });

            self.ui.set_root(root);

            self.ui.request_semantics_snapshot();
            self.ui.ingest_paint_cache_source(scene);
            scene.clear();

            let mut frame = UiFrameCx::new(
                &mut self.ui,
                &mut self.app,
                services,
                self.window,
                bounds,
                scale_factor,
            );
            frame.layout_all();
            frame.paint_all(scene);
        }

        let _ = self.runner.render_once();

        self.drain_effects();
    }
}

fn window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("window is not available"))
}

struct RafLoopState {
    active: bool,
    callback: Option<Closure<dyn FnMut(f64)>>,
    raf_id: Option<i32>,
    window: Window,
}

pub struct RafLoop {
    state: Rc<RefCell<RafLoopState>>,
}

impl RafLoop {
    pub fn stop(&mut self) {
        let mut state = self.state.borrow_mut();
        state.active = false;
        if let Some(id) = state.raf_id.take() {
            let _ = state.window.cancel_animation_frame(id);
        }
        state.callback.take();
    }
}

impl Drop for RafLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

fn request_animation_frame(window: &Window, callback: &Closure<dyn FnMut(f64)>) -> Option<i32> {
    window
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .ok()
}

fn start_raf_loop(demo: Rc<RefCell<WebDemo>>) -> Result<RafLoop, JsValue> {
    let window = window()?;
    let window_for_cb = window.clone();
    let window_for_first = window_for_cb.clone();

    let state = Rc::new(RefCell::new(RafLoopState {
        active: true,
        callback: None,
        raf_id: None,
        window,
    }));

    let state_for_cb = state.clone();
    let demo_for_cb = demo.clone();
    let callback = Closure::wrap(Box::new(move |_ts: f64| {
        if !state_for_cb.borrow().active {
            return;
        }

        if let Ok(mut d) = demo_for_cb.try_borrow_mut() {
            d.tick();
        }

        let next_id = {
            let state = state_for_cb.borrow();
            state
                .callback
                .as_ref()
                .and_then(|cb| request_animation_frame(&window_for_cb, cb))
        };
        if let Some(id) = next_id {
            state_for_cb.borrow_mut().raf_id = Some(id);
        }
    }) as Box<dyn FnMut(f64)>);

    state.borrow_mut().callback = Some(callback);
    let first_id = {
        let state_ref = state.borrow();
        state_ref
            .callback
            .as_ref()
            .and_then(|cb| request_animation_frame(&window_for_first, cb))
    };
    if let Some(id) = first_id {
        state.borrow_mut().raf_id = Some(id);
    }

    Ok(RafLoop { state })
}

#[wasm_bindgen]
pub struct WebDemoHandle {
    demo: Rc<RefCell<WebDemo>>,
    loop_handle: Option<RafLoop>,
}

#[wasm_bindgen]
impl WebDemoHandle {
    #[wasm_bindgen(js_name = create)]
    pub async fn create(
        canvas_id: String,
        prevent_default: bool,
    ) -> Result<WebDemoHandle, JsValue> {
        let demo = WebDemo::new(&canvas_id, prevent_default).await?;
        let demo = Rc::new(RefCell::new(demo));

        // Some browser APIs (file picker, clipboard, window.open) require being called from a user
        // activation stack. We use pointer/keyboard callbacks to synchronously run one UI turn.
        let weak: Weak<RefCell<WebDemo>> = Rc::downgrade(&demo);
        let cb: Rc<dyn Fn()> = Rc::new(move || {
            let Some(demo) = weak.upgrade() else {
                return;
            };
            if let Ok(mut d) = demo.try_borrow_mut() {
                d.tick();
            }
        });
        demo.borrow_mut()
            .input
            .set_user_activation_callback(Some(cb));

        Ok(WebDemoHandle {
            demo,
            loop_handle: None,
        })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        if self.loop_handle.is_some() {
            return Ok(());
        }
        let handle = start_raf_loop(self.demo.clone())?;
        self.loop_handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut handle) = self.loop_handle.take() {
            handle.stop();
        }
    }

    #[wasm_bindgen(js_name = renderOnce)]
    pub fn render_once(&mut self) {
        if let Ok(mut demo) = self.demo.try_borrow_mut() {
            demo.tick();
        }
    }
}

thread_local! {
    static AUTO_START_HANDLE: RefCell<Option<WebDemoHandle>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    spawn_local(async move {
        let handle = WebDemoHandle::create("fret-canvas".to_string(), true).await;
        match handle {
            Ok(mut handle) => {
                let _ = handle.start();
                AUTO_START_HANDLE.with(|slot| {
                    *slot.borrow_mut() = Some(handle);
                });
            }
            Err(err) => {
                web_sys::console::error_1(&err);
            }
        }
    });

    Ok(())
}
