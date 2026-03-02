use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use fret::prelude::*;
use fret_executor::{
    BackgroundTask, CancellationToken, Executors, Inbox, InboxConfig, InboxDrainer,
};
use fret_runtime::{DispatchPriority, DispatcherHandle, InboxDrainRegistry};

const TEST_ID_ROOT: &str = "cookbook.async_inbox_basics.root";
const TEST_ID_START: &str = "cookbook.async_inbox_basics.start";
const TEST_ID_CANCEL: &str = "cookbook.async_inbox_basics.cancel";
const TEST_ID_CLEAR_LOG: &str = "cookbook.async_inbox_basics.clear_log";
const TEST_ID_STATUS: &str = "cookbook.async_inbox_basics.status";
const TEST_ID_PROGRESS: &str = "cookbook.async_inbox_basics.progress";
const TEST_ID_LOG: &str = "cookbook.async_inbox_basics.log";

const MAX_LOG_LINES: usize = 64;

#[derive(Debug, Clone)]
enum InboxMsg {
    Progress { job: u64, value: f32 },
    Completed { job: u64, cancelled: bool },
    Log { job: u64, line: Arc<str> },
}

fn append_log(log: &mut String, line: &str) {
    if !log.is_empty() {
        log.push('\n');
    }
    log.push_str(line);

    let mut lines = log.lines().collect::<Vec<_>>();
    if lines.len() <= MAX_LOG_LINES {
        return;
    }
    lines.drain(0..(lines.len() - MAX_LOG_LINES));
    *log = lines.join("\n");
}

struct AsyncInboxBasicsState {
    window: AppWindowId,
    dispatcher: Option<DispatcherHandle>,
    current_job: Arc<AtomicU64>,

    // UI state.
    status: Model<Arc<str>>,
    running: Model<bool>,
    progress: Model<f32>,
    log: Model<String>,
    active_job: Model<u64>,

    // Execution.
    inbox: Inbox<InboxMsg>,
    task: Option<BackgroundTask>,
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    Start,
    Cancel,
    ClearLog,
}

struct AsyncInboxBasicsProgram;

impl MvuProgram for AsyncInboxBasicsProgram {
    type State = AsyncInboxBasicsState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        let dispatcher = app.global::<DispatcherHandle>().cloned();

        let current_job = Arc::new(AtomicU64::new(0));

        let status = app.models_mut().insert(Arc::<str>::from("Idle"));
        let running = app.models_mut().insert(false);
        let progress = app.models_mut().insert(0.0);
        let log = app.models_mut().insert(String::new());
        let active_job = app.models_mut().insert(0u64);

        let inbox = Inbox::new(InboxConfig {
            capacity: 256,
            ..Default::default()
        });

        // Background work must communicate via data-only messages. The runner drains inboxes at a
        // driver boundary (ADR 0175), so we register an inbox drainer in `InboxDrainRegistry`.
        let drainer = InboxDrainer::new(inbox.clone(), {
            let current_job = current_job.clone();
            let status_id = status.id();
            let running_id = running.id();
            let progress_id = progress.id();
            let log_id = log.id();

            move |host, window, msg| {
                let Some(window) = window else {
                    return;
                };

                let current_job = current_job.load(Ordering::Relaxed);

                match msg {
                    InboxMsg::Progress { job, value } => {
                        if job != current_job {
                            return;
                        }
                        let _ = host.models_mut().update_any(progress_id, |any| {
                            let Some(v) = any.downcast_mut::<f32>() else {
                                return;
                            };
                            *v = value;
                        });
                    }
                    InboxMsg::Completed { job, cancelled } => {
                        if job != current_job {
                            return;
                        }
                        let _ = host.models_mut().update_any(running_id, |any| {
                            let Some(v) = any.downcast_mut::<bool>() else {
                                return;
                            };
                            *v = false;
                        });
                        let _ = host.models_mut().update_any(progress_id, |any| {
                            let Some(v) = any.downcast_mut::<f32>() else {
                                return;
                            };
                            *v = 100.0;
                        });
                        let _ = host.models_mut().update_any(status_id, |any| {
                            let Some(v) = any.downcast_mut::<Arc<str>>() else {
                                return;
                            };
                            *v = if cancelled {
                                Arc::<str>::from("Cancelled")
                            } else {
                                Arc::<str>::from("Completed")
                            };
                        });
                        let _ = host.models_mut().update_any(log_id, |any| {
                            let Some(v) = any.downcast_mut::<String>() else {
                                return;
                            };
                            append_log(
                                v,
                                if cancelled {
                                    "job cancelled"
                                } else {
                                    "job completed"
                                },
                            );
                        });
                    }
                    InboxMsg::Log { job, line } => {
                        if job != current_job {
                            return;
                        }
                        let _ = host.models_mut().update_any(log_id, |any| {
                            let Some(v) = any.downcast_mut::<String>() else {
                                return;
                            };
                            append_log(v, &line);
                        });
                    }
                }

                host.request_redraw(window);
            }
        })
        .with_window_hint(window);

        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _app| {
            registry.register(Arc::new(drainer));
        });

        AsyncInboxBasicsState {
            window,
            dispatcher,
            current_job,
            status,
            running,
            progress,
            log,
            active_job,
            inbox,
            task: None,
        }
    }

    fn update(app: &mut App, st: &mut Self::State, msg: Self::Message) {
        match msg {
            Msg::ClearLog => {
                let _ = app.models_mut().update(&st.log, |v| v.clear());
                app.request_redraw(st.window);
            }
            Msg::Cancel => {
                if let Some(task) = st.task.take() {
                    task.cancel();
                }

                let _ = app.models_mut().update(&st.running, |v| *v = false);
                let _ = app
                    .models_mut()
                    .update(&st.status, |v| *v = Arc::<str>::from("Cancelling…"));
                app.request_redraw(st.window);
            }
            Msg::Start => {
                let Some(dispatcher) = st.dispatcher.clone() else {
                    let _ = app.models_mut().update(&st.status, |v| {
                        *v = Arc::<str>::from("Missing DispatcherHandle global (runner bug?)");
                    });
                    app.request_redraw(st.window);
                    return;
                };

                if let Some(task) = st.task.take() {
                    task.cancel();
                }

                let job = st
                    .current_job
                    .fetch_add(1, Ordering::Relaxed)
                    .wrapping_add(1)
                    .max(1);
                let _ = app.models_mut().update(&st.active_job, |v| *v = job);

                let _ = app.models_mut().update(&st.running, |v| *v = true);
                let _ = app.models_mut().update(&st.progress, |v| *v = 0.0);
                let _ = app
                    .models_mut()
                    .update(&st.status, |v| *v = Arc::<str>::from("Running"));
                let _ = app.models_mut().update(&st.log, |v| {
                    append_log(v, &format!("start job {job}"));
                });

                let inbox = st.inbox.clone().sender();
                let window = st.window;
                let executors = Executors::new(dispatcher.clone());

                st.task = Some(executors.spawn_background(
                    DispatchPriority::Normal,
                    move |token: CancellationToken| {
                        let steps = 48u32;
                        for step in 0..=steps {
                            if token.is_cancelled() {
                                let _ = inbox.send(InboxMsg::Completed {
                                    job,
                                    cancelled: true,
                                });
                                dispatcher.wake(Some(window));
                                return;
                            }

                            let value = (step as f32 / steps as f32) * 100.0;
                            let _ = inbox.send(InboxMsg::Progress { job, value });
                            if step == 0 {
                                let _ = inbox.send(InboxMsg::Log {
                                    job,
                                    line: Arc::<str>::from("background task started"),
                                });
                            }
                            dispatcher.wake(Some(window));

                            std::thread::sleep(Duration::from_millis(15));
                        }

                        let _ = inbox.send(InboxMsg::Completed {
                            job,
                            cancelled: false,
                        });
                        dispatcher.wake(Some(window));
                    },
                ));

                app.request_redraw(st.window);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let status = cx
            .watch_model(&state.status)
            .layout()
            .read_ref(|v| Arc::clone(v))
            .ok()
            .unwrap_or_else(|| Arc::<str>::from("<missing>"));
        let running = cx.watch_model(&state.running).layout().copied_or(false);
        let progress = cx.watch_model(&state.progress).layout().copied_or(0.0);
        let inbox_stats = state.inbox.stats();

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Async inbox basics").into_element(cx),
            shadcn::CardDescription::new(
                "Background work sends data-only messages into an Inbox, drained at a runner boundary (ADR 0175).",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let start_button = shadcn::Button::new("Start background job")
            .variant(shadcn::ButtonVariant::Default)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.play"))
            .disabled(running)
            .on_click(msg.cmd(Msg::Start))
            .into_element(cx)
            .test_id(TEST_ID_START);

        let cancel_button = shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.x"))
            .disabled(!running)
            .on_click(msg.cmd(Msg::Cancel))
            .into_element(cx)
            .test_id(TEST_ID_CANCEL);

        let clear_log_button = shadcn::Button::new("Clear log")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.trash"))
            .on_click(msg.cmd(Msg::ClearLog))
            .into_element(cx)
            .test_id(TEST_ID_CLEAR_LOG);

        let status_row = ui::h_flex(cx, |cx| {
            [
                shadcn::Label::new("Status:").into_element(cx),
                shadcn::Badge::new(status.as_ref())
                    .variant(if running {
                        shadcn::BadgeVariant::Default
                    } else {
                        shadcn::BadgeVariant::Secondary
                    })
                    .into_element(cx)
                    .test_id(TEST_ID_STATUS),
                shadcn::Badge::new(format!("Dropped oldest: {}", inbox_stats.dropped_oldest))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new(format!("Dropped newest: {}", inbox_stats.dropped_newest))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let progress_el = shadcn::Progress::new(state.progress.clone())
            .a11y_label("Background job progress")
            .range(0.0, 100.0)
            .into_element(cx)
            .test_id(TEST_ID_PROGRESS);

        let progress_label = cx.text(format!("{progress:.0}%"));
        let progress_row = ui::h_flex(cx, |_cx| [progress_el, progress_label])
            .gap(Space::N3)
            .items_center()
            .into_element(cx);

        let log = shadcn::Textarea::new(state.log.clone())
            .a11y_label("Inbox log")
            .placeholder("Log…")
            .disabled(true)
            .min_height(Px(240.0))
            .into_element(cx)
            .test_id(TEST_ID_LOG);

        let controls = ui::v_flex(cx, |_cx| [start_button, cancel_button, clear_log_button])
            .gap(Space::N2)
            .into_element(cx);

        let body = ui::v_flex(cx, |_cx| [status_row, progress_row, controls, log])
            .gap(Space::N3)
            .into_element(cx);

        let card = shadcn::Card::new([header, shadcn::CardContent::new([body]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(720.0))
            .into_element(cx);

        ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [card])
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-async-inbox-basics")
        .window("cookbook-async-inbox-basics", (860.0, 680.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<AsyncInboxBasicsProgram>()
        .map_err(anyhow::Error::from)
}
