use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::async_work::{self, AppAsyncWorkExt as _};
use fret::{icons::IconId, style::Space};
use fret_executor::{
    BackgroundTask, CancellationToken, Executors, Inbox, InboxConfig, InboxDrainer,
};

mod act {
    fret::actions!([
        Start = "cookbook.async_inbox_basics.start.v1",
        Cancel = "cookbook.async_inbox_basics.cancel.v1",
        ClearLog = "cookbook.async_inbox_basics.clear_log.v1"
    ]);
}

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
    window: WindowId,
    dispatcher: Option<async_work::DispatcherHandle>,
    current_job: Arc<AtomicU64>,

    // UI state.
    status: LocalState<Arc<str>>,
    running: LocalState<bool>,
    progress: LocalState<f32>,
    log: LocalState<String>,
    active_job: LocalState<u64>,

    // Execution.
    inbox: Inbox<InboxMsg>,
    task: LocalState<Option<BackgroundTask>>,
}

struct AsyncInboxBasicsView {
    st: AsyncInboxBasicsState,
}

impl View for AsyncInboxBasicsView {
    fn init(app: &mut App, window: WindowId) -> Self {
        let dispatcher = app.dispatcher();

        let current_job = Arc::new(AtomicU64::new(0));

        let status = app.local_state(Arc::<str>::from("Idle"));
        let running = app.local_state(false);
        let progress = app.local_state(0.0);
        let log = app.local_state(String::new());
        let active_job = app.local_state(0u64);
        let task = app.local_state(None::<BackgroundTask>);

        let inbox = Inbox::new(InboxConfig {
            capacity: 256,
            ..Default::default()
        });

        // Background work communicates via data-only messages. The runner drains inboxes at a
        // driver boundary (ADR 0175), and the app async-work facade owns the registration seam.
        let drainer = InboxDrainer::new(inbox.clone(), {
            let current_job = current_job.clone();
            let status = async_work::inbox_local(&status);
            let running = async_work::inbox_local(&running);
            let progress = async_work::inbox_local(&progress);
            let log = async_work::inbox_local(&log);

            async_work::inbox_drain_apply(move |cx, msg| {
                if cx.window_id().is_none() {
                    return;
                }

                let current_job = current_job.load(Ordering::Relaxed);

                match msg {
                    InboxMsg::Progress { job, value } => {
                        if job != current_job {
                            return;
                        }
                        cx.set_local(&progress, value);
                    }
                    InboxMsg::Completed { job, cancelled } => {
                        if job != current_job {
                            return;
                        }
                        cx.set_local(&running, false);
                        cx.set_local(&progress, 100.0);
                        cx.set_local(
                            &status,
                            if cancelled {
                                Arc::<str>::from("Cancelled")
                            } else {
                                Arc::<str>::from("Completed")
                            },
                        );
                        cx.update_local(&log, |v| {
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
                        cx.update_local(&log, |v| {
                            append_log(v, &line);
                        });
                    }
                }
            })
        })
        .with_window_hint(window);

        app.register_inbox_drainer(drainer);

        Self {
            st: AsyncInboxBasicsState {
                window,
                dispatcher,
                current_job,
                status,
                running,
                progress,
                log,
                active_job,
                inbox,
                task,
            },
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let status = self.st.status.layout_read_ref(cx, Arc::clone);
        let running = self.st.running.layout_value(cx);
        let progress = self.st.progress.layout_value(cx);
        let inbox_stats = self.st.inbox.stats();

        let start_button = shadcn::Button::new("Start background job")
            .variant(shadcn::ButtonVariant::Default)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.play"))
            .disabled(running)
            .action(act::Start)
            .test_id(TEST_ID_START);

        let cancel_button = shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.x"))
            .disabled(!running)
            .action(act::Cancel)
            .test_id(TEST_ID_CANCEL);

        let clear_log_button = shadcn::Button::new("Clear log")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.trash"))
            .action(act::ClearLog)
            .test_id(TEST_ID_CLEAR_LOG);

        let status_row = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Status:"),
                shadcn::Badge::new(status.as_ref())
                    .variant(if running {
                        shadcn::BadgeVariant::Default
                    } else {
                        shadcn::BadgeVariant::Secondary
                    })
                    .test_id(TEST_ID_STATUS),
                shadcn::Badge::new(format!("Dropped oldest: {}", inbox_stats.dropped_oldest))
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new(format!("Dropped newest: {}", inbox_stats.dropped_newest))
                    .variant(shadcn::BadgeVariant::Secondary),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element_in(cx);

        let progress_el = shadcn::Progress::new(&self.st.progress)
            .a11y_label("Background job progress")
            .range(0.0, 100.0)
            .into_element_in(cx)
            .test_id(TEST_ID_PROGRESS);

        let progress_label = ui::text(format!("{progress:.0}%"));
        let progress_row = ui::h_flex(|cx| ui::children![cx; progress_el, progress_label])
            .gap(Space::N3)
            .items_center()
            .into_element_in(cx);

        let log = shadcn::Textarea::new(&self.st.log)
            .a11y_label("Inbox log")
            .placeholder("Log…")
            .disabled(true)
            .min_height(Px(240.0))
            .test_id(TEST_ID_LOG);

        let controls = ui::v_flex(|cx| {
            ui::children![cx;
                start_button,
                cancel_button,
                clear_log_button,
            ]
        })
        .gap(Space::N2)
        .into_element_in(cx);

        let body = ui::v_flex(|cx| ui::children![cx; status_row, progress_row, controls, log])
            .gap(Space::N3);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Async inbox basics"),
                        shadcn::card_description(
                            "Background work sends data-only messages into an Inbox, drained at a runner boundary (ADR 0175).",
                        ),
                    ]
                })
                .into_element(cx),
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        cx.actions()
            .local(&self.st.log)
            .update::<act::ClearLog>(String::clear);

        cx.actions()
            .locals_with((&self.st.task, &self.st.running, &self.st.status))
            .on::<act::Cancel>(|tx, (task, running, status)| {
                let task_updated = tx.update(&task, |slot| {
                    if let Some(task) = slot.take() {
                        task.cancel();
                    }
                });
                let running_updated = tx.set(&running, false);
                let status_updated = tx.set(&status, Arc::<str>::from("Cancelling?"));

                task_updated && running_updated && status_updated
            });

        cx.actions()
            .locals_with((
                &self.st.active_job,
                &self.st.running,
                &self.st.progress,
                &self.st.status,
                &self.st.log,
                &self.st.task,
            ))
            .on::<act::Start>({
                let dispatcher = self.st.dispatcher.clone();
                let current_job = self.st.current_job.clone();
                let inbox_sender = self.st.inbox.clone().sender();
                let window = self.st.window;

                move |tx, (active_job, running, progress_model, status, log, task)| {
                    let Some(dispatcher) = dispatcher.clone() else {
                        tx.set(
                            &status,
                            Arc::<str>::from("Missing DispatcherHandle global (runner bug?)"),
                        );
                        return true;
                    };

                    tx.update(&task, |slot| {
                        if let Some(task) = slot.take() {
                            task.cancel();
                        }
                    });

                    let job = current_job
                        .fetch_add(1, Ordering::Relaxed)
                        .wrapping_add(1)
                        .max(1);
                    tx.set(&active_job, job);

                    tx.set(&running, true);
                    tx.set(&progress_model, 0.0);
                    tx.set(&status, Arc::<str>::from("Running"));
                    tx.update(&log, |v| {
                        append_log(v, &format!("start job {job}"));
                    });

                    let executors = Executors::new(dispatcher.clone());
                    let inbox = inbox_sender.clone();
                    let bg_task = executors.spawn_background(
                        async_work::DispatchPriority::Normal,
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
                    );

                    tx.set(&task, Some(bg_task));
                    true
                }
            });

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-async-inbox-basics")
        .window("cookbook-async-inbox-basics", (860.0, 680.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<AsyncInboxBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
