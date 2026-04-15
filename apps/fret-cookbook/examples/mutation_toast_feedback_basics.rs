use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::mutation::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, MutationError, MutationHandle,
    MutationPolicy, MutationState, MutationStatus,
};
use fret::style::Space;
use fret_ui::action::UiActionHostAdapter;

mod act {
    fret::actions!([
        SavePreset = "cookbook.mutation_toast_feedback_basics.save_preset.v1",
        RetryLastSave = "cookbook.mutation_toast_feedback_basics.retry_last_save.v1"
    ]);
}

const EFFECT_APPLY_PROJECTION: u64 = 0xAFA0_2101;
const EFFECT_SUCCESS_TOAST: u64 = 0xAFA0_2102;
const EFFECT_ERROR_TOAST: u64 = 0xAFA0_2103;

const TEST_ID_ROOT: &str = "cookbook.mutation_toast_feedback_basics.root";
const TEST_ID_METHOD_TOGGLE: &str = "cookbook.mutation_toast_feedback_basics.method_toggle";
const TEST_ID_NAME: &str = "cookbook.mutation_toast_feedback_basics.name";
const TEST_ID_ENDPOINT: &str = "cookbook.mutation_toast_feedback_basics.endpoint";
const TEST_ID_SAVE: &str = "cookbook.mutation_toast_feedback_basics.save";
const TEST_ID_RETRY: &str = "cookbook.mutation_toast_feedback_basics.retry";
const TEST_ID_STATUS_BADGE: &str = "cookbook.mutation_toast_feedback_basics.status.badge";
const TEST_ID_PROJECTION_NOTE: &str = "cookbook.mutation_toast_feedback_basics.projection.note";
const TEST_ID_LAST_SAVED: &str = "cookbook.mutation_toast_feedback_basics.last_saved";
const TEST_ID_DURATION: &str = "cookbook.mutation_toast_feedback_basics.duration";

#[derive(Debug, Clone)]
struct TokioRuntimeGlobal {
    _rt: Arc<tokio::runtime::Runtime>,
}

#[derive(Clone)]
struct TokioHandleSpawner(tokio::runtime::Handle);

impl FutureSpawner for TokioHandleSpawner {
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        let _ = self.0.spawn(fut);
    }
}

fn install_mutation_runtime(app: &mut App) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("failed to build tokio runtime");
    let rt = Arc::new(rt);

    let spawner: FutureSpawnerHandle = Arc::new(TokioHandleSpawner(rt.handle().clone()));
    app.set_global::<FutureSpawnerHandle>(spawner);
    app.set_global::<TokioRuntimeGlobal>(TokioRuntimeGlobal { _rt: rt });
}

#[derive(Debug, Clone)]
struct RequestPresetDraft {
    name: Arc<str>,
    method: Arc<str>,
    endpoint: Arc<str>,
}

#[derive(Debug, Clone)]
struct SavedRequestPreset {
    name: Arc<str>,
    method: Arc<str>,
    endpoint: Arc<str>,
    summary: Arc<str>,
}

#[derive(Clone)]
struct MutationToastFeedbackLocals {
    name: LocalState<String>,
    method: LocalState<Option<Arc<str>>>,
    endpoint: LocalState<String>,
    projection_note: LocalState<String>,
    last_saved_summary: LocalState<String>,
}

impl MutationToastFeedbackLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            name: cx.state().local_init(|| "Create issue".to_string()),
            method: cx.state().local_init(|| Some(Arc::<str>::from("POST"))),
            endpoint: cx.state().local_init(|| "{{base_url}}/issues".to_string()),
            projection_note: cx
                .state()
                .local_init(|| "Submit to save the current API request preset.".to_string()),
            last_saved_summary: cx
                .state()
                .local_init(|| "No preset has been saved yet.".to_string()),
        }
    }
}

struct MutationToastFeedbackBasicsView {
    window: WindowId,
}

impl View for MutationToastFeedbackBasicsView {
    fn init(_app: &mut App, window: WindowId) -> Self {
        Self { window }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let locals = MutationToastFeedbackLocals::new(cx);
        let handle = cx
            .data()
            .mutation_async(MutationPolicy::default(), save_request_preset);
        bind_actions(cx, &locals, &handle, self.window);

        let state = handle.read_layout(cx);
        let _ = cx
            .data()
            .update_after_mutation_completion(EFFECT_APPLY_PROJECTION, &handle, {
                let locals = locals.clone();
                move |models, state| apply_save_projection(models, &locals, state)
            });
        emit_save_feedback_toasts(cx, self.window, &handle, &state);

        let method_label = locals
            .method
            .layout_value(cx)
            .unwrap_or_else(|| Arc::<str>::from("POST"));
        let projection_note = locals.projection_note.layout_value(cx);
        let last_saved_summary = locals.last_saved_summary.layout_value(cx);
        let can_retry = state.input.is_some() && !state.is_running();
        let duration_label = state
            .last_duration
            .map(|duration| format!("Latest completion: {} ms", duration.as_millis()))
            .unwrap_or_else(|| "Latest completion: pending".to_string());

        let method_toggle = shadcn::ToggleGroup::single(&locals.method)
            .items([
                shadcn::ToggleGroupItem::new("GET", [cx.text("GET")])
                    .a11y_label("GET")
                    .test_id("cookbook.mutation_toast_feedback_basics.method.get"),
                shadcn::ToggleGroupItem::new("POST", [cx.text("POST")])
                    .a11y_label("POST")
                    .test_id("cookbook.mutation_toast_feedback_basics.method.post"),
                shadcn::ToggleGroupItem::new("PATCH", [cx.text("PATCH")])
                    .a11y_label("PATCH")
                    .test_id("cookbook.mutation_toast_feedback_basics.method.patch"),
            ])
            .test_id(TEST_ID_METHOD_TOGGLE);

        let fields = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Label::new("HTTP method"),
                        ui::h_row(|_cx| [method_toggle]).w_full(),
                    ]
                })
                .gap(Space::N1),
                ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Label::new("Preset name"),
                        shadcn::Input::new(&locals.name)
                            .a11y_label("Preset name")
                            .placeholder("Create issue")
                            .test_id(TEST_ID_NAME),
                    ]
                })
                .gap(Space::N1),
                ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Label::new("Endpoint template"),
                        shadcn::Input::new(&locals.endpoint)
                            .a11y_label("Endpoint template")
                            .placeholder("{{base_url}}/issues")
                            .test_id(TEST_ID_ENDPOINT),
                    ]
                })
                .gap(Space::N1),
            ]
        })
        .gap(Space::N3)
        .w_full();

        let status_badge = shadcn::Badge::new(state.status.as_str())
            .variant(match state.status {
                MutationStatus::Idle => shadcn::BadgeVariant::Secondary,
                MutationStatus::Running => shadcn::BadgeVariant::Outline,
                MutationStatus::Success => shadcn::BadgeVariant::Default,
                MutationStatus::Error => shadcn::BadgeVariant::Destructive,
            })
            .test_id(TEST_ID_STATUS_BADGE);

        let status_row = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Submit lane status"),
                status_badge,
                shadcn::Badge::new(method_label.as_ref())
                    .variant(shadcn::BadgeVariant::Outline),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .w_full();

        let actions = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Save preset")
                    .action(act::SavePreset)
                    .disabled(state.is_running())
                    .test_id(TEST_ID_SAVE),
                shadcn::Button::new("Retry last submit")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::RetryLastSave)
                    .disabled(!can_retry)
                    .test_id(TEST_ID_RETRY),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .w_full();

        let projection_card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("App-owned projection"),
                        shadcn::card_description(
                            "The saved summary and note live on ordinary locals. Sonner only mirrors the same completion as feedback.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                shadcn::Label::new("Projection note"),
                                cx.text(projection_note.clone()).test_id(TEST_ID_PROJECTION_NOTE),
                            ]
                        })
                        .gap(Space::N1),
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                shadcn::Label::new("Last saved summary"),
                                cx.text(last_saved_summary.clone()).test_id(TEST_ID_LAST_SAVED),
                            ]
                        })
                        .gap(Space::N1),
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                shadcn::Label::new("Handle completion timing"),
                                cx.text(duration_label).test_id(TEST_ID_DURATION),
                            ]
                        })
                        .gap(Space::N1),
                    ]
                }),
            ]
        })
        .ui()
        .w_full();

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Mutation + toast feedback"),
                        shadcn::card_description(
                            "A Postman-style request preset save: `fret-mutation` owns the authoritative submit lifecycle, while Sonner mirrors success/error as feedback only.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        status_row,
                        fields,
                        actions,
                        projection_card,
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(820.0));

        let mut root = fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card);

        // Keep the toaster mounted in-tree so feedback projection remains a replaceable recipe
        // concern above the authoritative mutation state.
        root.push(shadcn::Toaster::new().into_element(cx));
        root.into()
    }
}

fn bind_actions(
    cx: &mut AppUi<'_, '_>,
    locals: &MutationToastFeedbackLocals,
    handle: &MutationHandle<RequestPresetDraft, SavedRequestPreset>,
    window: WindowId,
) {
    cx.actions().models::<act::SavePreset>({
        let locals = locals.clone();
        let handle = handle.clone();
        move |models| submit_preset(models, window, &locals, &handle)
    });
    cx.actions().models::<act::RetryLastSave>({
        let locals = locals.clone();
        let handle = handle.clone();
        move |models| retry_last_save(models, window, &locals, &handle)
    });
}

fn submit_preset(
    models: &mut fret_runtime::ModelStore,
    window: WindowId,
    locals: &MutationToastFeedbackLocals,
    handle: &MutationHandle<RequestPresetDraft, SavedRequestPreset>,
) -> bool {
    let draft = build_draft(models, locals);
    let mut handled = false;
    handled = locals.projection_note.set_in(
        models,
        "Submitting the current request preset...".to_string(),
    ) || handled;
    handled = handle.submit(models, window, draft) || handled;
    handled
}

fn retry_last_save(
    models: &mut fret_runtime::ModelStore,
    window: WindowId,
    locals: &MutationToastFeedbackLocals,
    handle: &MutationHandle<RequestPresetDraft, SavedRequestPreset>,
) -> bool {
    if !can_retry_last_save(models, handle) {
        return false;
    }

    let mut handled = false;
    handled = locals
        .projection_note
        .set_in(models, "Retrying the last submitted preset...".to_string())
        || handled;
    handled = handle.retry_last(models, window) || handled;
    handled
}

fn can_retry_last_save(
    models: &mut fret_runtime::ModelStore,
    handle: &MutationHandle<RequestPresetDraft, SavedRequestPreset>,
) -> bool {
    models
        .read(handle.model(), |state| {
            state.input.is_some() && !state.is_running()
        })
        .ok()
        .unwrap_or(false)
}

fn build_draft(
    models: &mut fret_runtime::ModelStore,
    locals: &MutationToastFeedbackLocals,
) -> RequestPresetDraft {
    RequestPresetDraft {
        name: Arc::from(locals.name.value_in_or_default(models)),
        method: locals
            .method
            .value_in(models)
            .flatten()
            .unwrap_or_else(|| Arc::<str>::from("POST")),
        endpoint: Arc::from(locals.endpoint.value_in_or_default(models)),
    }
}

fn apply_save_projection(
    models: &mut fret_runtime::ModelStore,
    locals: &MutationToastFeedbackLocals,
    state: MutationState<RequestPresetDraft, SavedRequestPreset>,
) -> bool {
    let mut changed = false;
    let note = if let Some(saved) = state.data.as_ref() {
        changed = locals
            .last_saved_summary
            .set_in(models, saved.summary.to_string())
            || changed;
        format!("Saved \"{}\" to the request preset catalog.", saved.name)
    } else {
        let message = state
            .error
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unknown save failure".to_string());
        format!("Save failed: {message}")
    };
    changed = locals.projection_note.set_in(models, note) || changed;
    changed
}

fn emit_save_feedback_toasts(
    cx: &mut AppUi<'_, '_>,
    window: WindowId,
    handle: &MutationHandle<RequestPresetDraft, SavedRequestPreset>,
    state: &MutationState<RequestPresetDraft, SavedRequestPreset>,
) {
    // Keep feedback on the host-owned Sonner bridge so the mutation handle stays the
    // authoritative submit owner and the toast remains a replaceable projection.
    if state.is_success()
        && cx
            .data()
            .take_mutation_success(EFFECT_SUCCESS_TOAST, handle)
    {
        let Some(saved) = state.data.as_ref() else {
            return;
        };
        let sonner = shadcn::Sonner::global(cx.app);
        let mut host = UiActionHostAdapter { app: &mut *cx.app };
        sonner.toast_success_message(
            &mut host,
            window,
            "Preset saved",
            shadcn::ToastMessageOptions::new().description(format!(
                "{} {} -> {}",
                saved.method, saved.name, saved.endpoint
            )),
        );
        return;
    }

    if state.is_error()
        && cx
            .data()
            .take_mutation_completion(EFFECT_ERROR_TOAST, handle)
    {
        let description = state
            .error
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unknown save failure".to_string());
        let sonner = shadcn::Sonner::global(cx.app);
        let mut host = UiActionHostAdapter { app: &mut *cx.app };
        sonner.toast_error_message(
            &mut host,
            window,
            "Save failed",
            shadcn::ToastMessageOptions::new().description(description),
        );
    }
}

async fn save_request_preset(
    token: CancellationToken,
    input: Arc<RequestPresetDraft>,
) -> Result<SavedRequestPreset, MutationError> {
    tokio::time::sleep(Duration::from_millis(350)).await;

    if token.is_cancelled() {
        return Err(MutationError::transient("save cancelled"));
    }

    let name = input.name.trim();
    if name.is_empty() {
        return Err(MutationError::permanent("Preset name is required"));
    }

    let endpoint = input.endpoint.trim();
    if endpoint.is_empty() {
        return Err(MutationError::permanent("Endpoint template is required"));
    }

    if !(endpoint.starts_with('/')
        || endpoint.starts_with("http://")
        || endpoint.starts_with("https://")
        || endpoint.starts_with("{{base_url}}"))
    {
        return Err(MutationError::permanent(
            "Endpoint should start with `/`, `http(s)://`, or `{{base_url}}`",
        ));
    }

    let method = input.method.trim();
    let summary = Arc::<str>::from(format!("{method} {name} -> {endpoint}"));

    Ok(SavedRequestPreset {
        name: Arc::from(name),
        method: Arc::from(method),
        endpoint: Arc::from(endpoint),
        summary,
    })
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-mutation-toast-feedback-basics")
        .window("cookbook-mutation-toast-feedback-basics", (900.0, 640.0))
        .config_files(false)
        .setup((
            install_mutation_runtime,
            fret_cookbook::install_cookbook_defaults,
        ))
        .view::<MutationToastFeedbackBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
