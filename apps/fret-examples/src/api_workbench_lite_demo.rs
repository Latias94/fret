use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use fret::advanced::prelude::LocalState;
use fret::app::prelude::*;
use fret::children::UiElementSinkExt as _;
use fret::icons::IconId;
use fret::mutation::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, MutationError, MutationHandle,
    MutationPolicy, MutationState,
};
use fret::style::{ColorRef, LayoutRefinement, Space, Theme};
use fret::{FretApp, shadcn};
use fret_app::{CommandId, CommandMeta, CommandScope, DefaultKeybinding, KeyChord, PlatformFilter};
use fret_core::{KeyCode, Modifiers, Px};
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::ElementContextThemeExt as _;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

const TEST_ID_ROOT: &str = "api-workbench-lite.root";
const TEST_ID_SIDEBAR: &str = "api-workbench-lite.sidebar";
const TEST_ID_URL: &str = "api-workbench-lite.request.url";
const TEST_ID_METHOD: &str = "api-workbench-lite.request.method";
const TEST_ID_HEADERS: &str = "api-workbench-lite.request.headers";
const TEST_ID_BODY: &str = "api-workbench-lite.request.body";
const TEST_ID_SEND: &str = "api-workbench-lite.request.send";
const TEST_ID_SETTINGS_BUTTON: &str = "api-workbench-lite.settings.button";
const TEST_ID_COMMAND_BUTTON: &str = "api-workbench-lite.command.button";
const TEST_ID_RESPONSE_STATUS: &str = "api-workbench-lite.response.status";
const TEST_ID_RESPONSE_PRETTY: &str = "api-workbench-lite.response.pretty";
const TEST_ID_RESPONSE_RAW: &str = "api-workbench-lite.response.raw";
const TEST_ID_RESPONSE_HEADERS: &str = "api-workbench-lite.response.headers";
const TEST_ID_SETTINGS_DIALOG: &str = "api-workbench-lite.settings.dialog";
const TEST_ID_SETTINGS_BASE_URL: &str = "api-workbench-lite.settings.base_url";
const TEST_ID_SETTINGS_AUTH_TOKEN: &str = "api-workbench-lite.settings.auth_token";
const TEST_ID_HISTORY_EMPTY: &str = "api-workbench-lite.history.empty";

mod act {
    fret::actions!([
        SendRequest = "api_workbench_lite.send_request.v1",
        OpenSettings = "api_workbench_lite.open_settings.v1",
        CloseSettings = "api_workbench_lite.close_settings.v1",
    ]);

    fret::payload_actions!([
        LoadCollection(u8) = "api_workbench_lite.load_collection.v1",
        LoadHistory(u64) = "api_workbench_lite.load_history.v1",
    ]);
}

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

#[derive(Debug, Clone)]
struct EnvironmentSnapshot {
    base_url: Arc<str>,
    auth_token: Arc<str>,
}

#[derive(Debug, Clone)]
struct RequestSnapshot {
    seq: u64,
    collection_id: Option<u8>,
    collection_label: Arc<str>,
    method: Arc<str>,
    url: Arc<str>,
    headers_text: Arc<str>,
    body: Arc<str>,
    env: EnvironmentSnapshot,
}

#[derive(Debug, Clone)]
struct ResponsePayload {
    status_code: u16,
    status_text: Arc<str>,
    duration_ms: u64,
    header_lines: Arc<str>,
    raw_body: Arc<str>,
    pretty_body: Arc<str>,
    resolved_url: Arc<str>,
    is_http_error: bool,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    id: u64,
    snapshot: RequestSnapshot,
    status_line: Arc<str>,
    timing_line: Arc<str>,
}

#[derive(Clone)]
struct WorkbenchLocals {
    method: LocalState<Option<Arc<str>>>,
    method_open: LocalState<bool>,
    url: LocalState<String>,
    headers: LocalState<String>,
    body: LocalState<String>,
    request_tab: LocalState<Option<Arc<str>>>,
    response_tab: LocalState<Option<Arc<str>>>,
    settings_open: LocalState<bool>,
    base_url: LocalState<String>,
    auth_token: LocalState<String>,
    next_seq: LocalState<u64>,
    last_applied_seq: LocalState<u64>,
    next_history_id: LocalState<u64>,
    selected_collection: LocalState<Option<Arc<str>>>,
    selected_history: LocalState<Option<u64>>,
    history: LocalState<Vec<HistoryEntry>>,
    response_status: LocalState<String>,
    response_pretty: LocalState<String>,
    response_raw: LocalState<String>,
    response_headers: LocalState<String>,
}

impl WorkbenchLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            method: cx.state().local_init(|| Some(Arc::<str>::from("POST"))),
            method_open: cx.state().local_init(|| false),
            url: cx
                .state()
                .local_init(|| "{{base_url}}/anything".to_string()),
            headers: cx.state().local_init(|| {
                "Content-Type: application/json\nX-Fret-Probe: api-workbench-lite".to_string()
            }),
            body: cx.state().local_init(default_echo_body),
            request_tab: cx.state().local_init(|| Some(Arc::<str>::from("body"))),
            response_tab: cx.state().local_init(|| Some(Arc::<str>::from("pretty"))),
            settings_open: cx.state().local_init(|| false),
            base_url: cx.state().local_init(|| "http://httpbin.org".to_string()),
            auth_token: cx.state().local_init(String::new),
            next_seq: cx.state().local_init(|| 1u64),
            last_applied_seq: cx.state().local_init(|| 0u64),
            next_history_id: cx.state().local_init(|| 1u64),
            selected_collection: cx
                .state()
                .local_init(|| Some(Arc::<str>::from(collection_key(0)))),
            selected_history: cx.state().local_init(|| None::<u64>),
            history: cx.state().local_init(Vec::new),
            response_status: cx.state().local_init(|| "Idle".to_string()),
            response_pretty: cx
                .state()
                .local_init(|| "Press Send Request to capture a response.".to_string()),
            response_raw: cx
                .state()
                .local_init(|| "Press Send Request to capture a response.".to_string()),
            response_headers: cx
                .state()
                .local_init(|| "Response headers will appear here.".to_string()),
        }
    }
}

struct ApiWorkbenchLiteView {
    window: WindowId,
}

impl View for ApiWorkbenchLiteView {
    fn init(_app: &mut App, window: WindowId) -> Self {
        Self { window }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let locals = WorkbenchLocals::new(cx);
        let response_mutation = cx.data().mutation_async(
            MutationPolicy::default(),
            move |token, snapshot: Arc<RequestSnapshot>| run_request((*snapshot).clone(), token),
        );
        bind_actions(cx, &locals, &response_mutation, self.window);

        let theme = Theme::global(&*cx.app).snapshot();
        let method = locals
            .method
            .layout_value(cx)
            .unwrap_or_else(|| Arc::<str>::from("GET"));
        let history = locals.history.layout_value(cx);
        let selected_collection = locals.selected_collection.layout_value(cx);
        let selected_history = locals.selected_history.layout_value(cx);
        let response_status = locals.response_status.layout_value(cx);
        let mutation_state = response_mutation.read_layout(cx);
        maybe_apply_response_snapshot(cx, self.window, &locals, &mutation_state);

        let status_badge = status_badge(&response_status).test_id(TEST_ID_RESPONSE_STATUS);
        let send_button = shadcn::Button::new("Send Request")
            .variant(shadcn::ButtonVariant::Default)
            .icon(IconId::new_static("lucide.send-horizontal"))
            .action(act::SendRequest)
            .test_id(TEST_ID_SEND);
        let settings_button = shadcn::Button::new("Environments")
            .variant(shadcn::ButtonVariant::Outline)
            .icon(IconId::new_static("lucide.settings-2"))
            .action(act::OpenSettings)
            .test_id(TEST_ID_SETTINGS_BUTTON);
        let command_button = shadcn::Button::new("Command Palette")
            .variant(shadcn::ButtonVariant::Ghost)
            .icon(IconId::new_static("lucide.search"))
            .action(CommandId::new("app.command_palette"))
            .test_id(TEST_ID_COMMAND_BUTTON);

        let shell = shell_frame(
            cx,
            &locals,
            theme.clone(),
            method,
            history,
            selected_collection,
            selected_history,
            status_badge,
            send_button,
            settings_button,
            command_button,
        );

        let settings_open_for_close = locals.settings_open.clone();
        let dialog = shadcn::Dialog::new(&locals.settings_open).into_element(
            cx,
            move |_cx| shell,
            move |cx| {
                shadcn::DialogContent::build(|cx, out| {
                    let settings_fields = ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            ui::v_flex(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Label::new("Base URL"),
                                    shadcn::Input::new(&locals.base_url)
                                        .a11y_label("Base URL")
                                        .placeholder("http://httpbin.org")
                                        .test_id(TEST_ID_SETTINGS_BASE_URL),
                                ]
                            })
                            .gap(Space::N1),
                            ui::v_flex(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Label::new("Authorization"),
                                    shadcn::Input::new(&locals.auth_token)
                                        .a11y_label("Authorization token")
                                        .placeholder("Bearer ...")
                                        .test_id(TEST_ID_SETTINGS_AUTH_TOKEN),
                                ]
                            })
                            .gap(Space::N1),
                        ]
                    })
                    .gap(Space::N3)
                    .w_full()
                    .into_element(cx);

                    out.push_ui(
                        cx,
                        shadcn::DialogHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::DialogTitle::new("Environment Settings"));
                            out.push_ui(
                                cx,
                                shadcn::DialogDescription::new(
                                    "Use {{base_url}} inside the request URL to keep the request editor portable.",
                                ),
                            );
                        }),
                    );
                    out.push_ui(cx, settings_fields);
                    out.push_ui(
                        cx,
                        shadcn::DialogFooter::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::Button::new("Done")
                                    .variant(shadcn::ButtonVariant::Default)
                                    .toggle_model(settings_open_for_close.clone()),
                            );
                        }),
                    );
                })
                .show_close_button(true)
                .ui()
                .test_id(TEST_ID_SETTINGS_DIALOG)
                .into_element(cx)
            },
        );

        ui::single(cx, dialog)
    }
}

fn bind_actions(
    cx: &mut AppUi<'_, '_>,
    locals: &WorkbenchLocals,
    response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
    window: WindowId,
) {
    cx.actions()
        .local(&locals.settings_open)
        .set::<act::OpenSettings>(true);
    cx.actions()
        .local(&locals.settings_open)
        .set::<act::CloseSettings>(false);

    cx.actions().models::<act::SendRequest>({
        let locals = locals.clone();
        let response_mutation = response_mutation.clone();
        move |models| submit_request(models, window, &locals, &response_mutation)
    });

    cx.actions().availability::<act::SendRequest>({
        let url = locals.url.clone();
        move |host, _acx| {
            if url.value_in_or_default(host.models_mut()).trim().is_empty() {
                fret_ui::CommandAvailability::Blocked
            } else {
                fret_ui::CommandAvailability::Available
            }
        }
    });

    cx.actions().payload_models::<act::LoadCollection>({
        let locals = locals.clone();
        move |models, preset_id| apply_collection(models, &locals, preset_id)
    });

    cx.actions().payload_models::<act::LoadHistory>({
        let locals = locals.clone();
        move |models, history_id| load_history(models, &locals, history_id)
    });
}

fn submit_request(
    models: &mut fret_runtime::ModelStore,
    window: WindowId,
    locals: &WorkbenchLocals,
    response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
) -> bool {
    let url_value = locals.url.value_in_or_default(models);
    if url_value.trim().is_empty() {
        return false;
    }

    let seq = locals.next_seq.value_in_or_default(models);
    let selected_collection = locals.selected_collection.value_in(models).flatten();
    let snapshot = RequestSnapshot {
        seq,
        collection_id: selected_collection
            .as_deref()
            .and_then(collection_id_from_key),
        collection_label: selected_collection
            .as_deref()
            .and_then(collection_id_from_key)
            .map(|id| Arc::<str>::from(collection_preset(id).label))
            .unwrap_or_else(|| Arc::<str>::from("Scratch request")),
        method: locals
            .method
            .value_in(models)
            .flatten()
            .unwrap_or_else(|| Arc::<str>::from("GET")),
        url: Arc::from(url_value),
        headers_text: Arc::from(locals.headers.value_in_or_default(models)),
        body: Arc::from(locals.body.value_in_or_default(models)),
        env: EnvironmentSnapshot {
            base_url: Arc::from(locals.base_url.value_in_or_default(models)),
            auth_token: Arc::from(locals.auth_token.value_in_or_default(models)),
        },
    };
    let snapshot_url = snapshot.url.clone();

    let mut handled = false;
    handled = locals.next_seq.set_in(models, seq.saturating_add(1)) || handled;
    handled = locals.selected_history.set_in(models, None) || handled;
    handled = locals
        .response_tab
        .set_in(models, Some(Arc::<str>::from("pretty")))
        || handled;
    handled = locals.response_status.set_in(models, "Loading".to_string()) || handled;
    handled = locals
        .response_pretty
        .set_in(models, "Sending request...".to_string())
        || handled;
    handled = locals
        .response_raw
        .set_in(models, "Sending request...".to_string())
        || handled;
    handled = locals
        .response_headers
        .set_in(models, "Response headers will appear here.".to_string())
        || handled;
    handled = response_mutation.submit(models, window, snapshot) || handled;

    tracing::info!(
        seq,
        url = %snapshot_url,
        handled,
        "api_workbench_lite queued request"
    );

    handled
}

fn shell_frame(
    cx: &mut AppUi<'_, '_>,
    locals: &WorkbenchLocals,
    theme: fret::style::ThemeSnapshot,
    method: Arc<str>,
    history: Vec<HistoryEntry>,
    selected_collection: Option<Arc<str>>,
    selected_history: Option<u64>,
    status_badge: impl UiChild,
    send_button: impl UiChild,
    settings_button: impl UiChild,
    command_button: impl UiChild,
) -> AnyElement {
    let base_url = locals.base_url.layout_value(cx);
    let header = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            ui::h_flex(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("API Workbench Lite")
                                        .font_semibold()
                                        .text_base(),
                                    shadcn::Badge::new("consumer probe")
                                        .variant(shadcn::BadgeVariant::Secondary),
                                    status_badge,
                                ]
                            })
                            .gap(Space::N2)
                            .items_center(),
                            ui::text(
                                "First-contact task: build a Postman-like tool only from Fret's public app surface.",
                            )
                            .text_sm()
                            .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
                        ]
                    })
                    .gap(Space::N1)
                    .flex_1()
                    .min_w_0(),
                    ui::h_flex(|cx| ui::children![cx; command_button, settings_button, send_button])
                        .gap(Space::N2)
                        .items_center(),
                ]
            }),
        ]
    })
    .ui()
    .w_full();

    let request_panel = request_panel(cx, locals, method);
    let response_panel = response_panel(cx, locals);
    let sidebar = sidebar_frame(
        cx,
        locals,
        base_url,
        history,
        selected_collection,
        selected_history,
    );

    let main = shadcn::SidebarInset::new([ui::v_flex(|cx| {
        ui::children![
            cx;
            header,
            ui::h_flex(|cx| ui::children![cx; request_panel, response_panel])
                .gap(Space::N4)
                .items_stretch()
                .w_full()
                .flex_1()
                .min_w_0(),
        ]
    })
    .gap(Space::N4)
    .p(Space::N4)
    .w_full()
    .h_full()
    .items_stretch()
    .into_element(cx)])
    .into_element(cx);

    let content = shadcn::SidebarProvider::new()
        .width(Px(300.0))
        .width_icon(Px(72.0))
        .width_mobile(Px(320.0))
        .with(cx, |cx| {
            vec![
                ui::h_flex(|_cx| vec![sidebar, main])
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().size_full())
                    .into_element(cx)
                    .test_id(TEST_ID_ROOT),
            ]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| ui::container(|_cx| Vec::<AnyElement>::new()).into_element(cx))
}

fn sidebar_frame(
    cx: &mut AppUi<'_, '_>,
    locals: &WorkbenchLocals,
    base_url: String,
    history: Vec<HistoryEntry>,
    selected_collection: Option<Arc<str>>,
    selected_history: Option<u64>,
) -> AnyElement {
    let collection_group = shadcn::SidebarGroup::new([
        shadcn::SidebarGroupLabel::new("Collections").into_element(cx),
        shadcn::SidebarGroupContent::new([shadcn::SidebarMenu::new(collection_buttons(
            cx,
            selected_collection,
        ))
        .into_element(cx)])
        .into_element(cx),
    ])
    .into_element(cx);

    let history_group = shadcn::SidebarGroup::new([
        shadcn::SidebarGroupLabel::new("History").into_element(cx),
        shadcn::SidebarGroupContent::new([history_menu(cx, history, selected_history)])
            .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Sidebar::new([
        shadcn::SidebarHeader::new([ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text("Fret API Probe").font_semibold(),
                ui::text("Postman-like first contact")
                    .text_sm()
                    .text_color(ColorRef::Color(
                        cx.theme_snapshot().color_token("muted-foreground"),
                    )),
            ]
        })
        .gap(Space::N1)
        .into_element(cx)])
        .into_element(cx),
        shadcn::SidebarContent::new([collection_group, history_group]).into_element(cx),
        shadcn::SidebarFooter::new([ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text("Active base URL").text_xs().font_semibold(),
                ui::text(base_url)
                    .text_xs()
                    .text_color(ColorRef::Color(
                        cx.theme_snapshot().color_token("muted-foreground"),
                    )),
            ]
        })
        .gap(Space::N1)
        .into_element(cx)])
        .into_element(cx),
    ])
    .collapsible(shadcn::SidebarCollapsible::Icon)
    .refine_layout(LayoutRefinement::default().h_full())
    .into_element(cx)
    .test_id(TEST_ID_SIDEBAR)
}

fn collection_buttons(
    cx: &mut AppUi<'_, '_>,
    selected_collection: Option<Arc<str>>,
) -> Vec<AnyElement> {
    (0u8..3u8)
        .map(|id| {
            let preset = collection_preset(id);
            let key = Arc::<str>::from(collection_key(id));
            shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new(preset.label)
                    .icon(IconId::new_static(preset.icon))
                    .active(selected_collection.as_deref() == Some(key.as_ref()))
                    .action(act::LoadCollection)
                    .action_payload(id)
                    .test_id(format!(
                        "api-workbench-lite.collection.{}",
                        collection_key(id)
                    ))
                    .into_element(cx),
            )
            .into_element(cx)
        })
        .collect()
}

fn history_menu(
    cx: &mut AppUi<'_, '_>,
    history: Vec<HistoryEntry>,
    selected_history: Option<u64>,
) -> AnyElement {
    if history.is_empty() {
        return ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text("No requests yet.")
                    .text_sm()
                    .text_color(ColorRef::Color(
                        cx.theme_snapshot().color_token("muted-foreground"),
                    ))
                    .test_id(TEST_ID_HISTORY_EMPTY),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);
    }

    shadcn::SidebarMenu::new(history.into_iter().map(|entry| {
        let active = selected_history == Some(entry.id);
        let label = format!(
            "{} · {}",
            entry.status_line.as_ref(),
            short_request_label(&entry.snapshot)
        );
        shadcn::SidebarMenuItem::new(
            shadcn::SidebarMenuButton::new(label)
                .icon(IconId::new_static("lucide.history"))
                .active(active)
                .action(act::LoadHistory)
                .action_payload(entry.id)
                .test_id(format!("api-workbench-lite.history.row.{}", entry.id))
                .into_element(cx),
        )
        .into_element(cx)
    }))
    .into_element(cx)
}

fn request_panel(cx: &mut AppUi<'_, '_>, locals: &WorkbenchLocals, method: Arc<str>) -> AnyElement {
    let request_url = locals.url.layout_value(cx);
    let method_select = shadcn::Select::new(&locals.method, &locals.method_open)
        .a11y_label("Method")
        .value(shadcn::SelectValue::new().placeholder("Method"))
        .items([
            shadcn::SelectItem::new("GET", "GET"),
            shadcn::SelectItem::new("POST", "POST"),
            shadcn::SelectItem::new("PUT", "PUT"),
            shadcn::SelectItem::new("PATCH", "PATCH"),
            shadcn::SelectItem::new("DELETE", "DELETE"),
        ])
        .test_id(TEST_ID_METHOD);

    let url = shadcn::Input::new(&locals.url)
        .a11y_label("Request URL")
        .placeholder("{{base_url}}/anything")
        .test_id(TEST_ID_URL)
        .ui()
        .flex_1()
        .min_w_0();

    let request_tabs = shadcn::Tabs::new(&locals.request_tab)
        .content_fill_remaining(true)
        .items([
            shadcn::TabsItem::new(
                "body",
                "Body",
                [shadcn::Textarea::new(&locals.body)
                    .a11y_label("Request body")
                    .placeholder("{ ... }")
                    .min_height(Px(260.0))
                    .test_id(TEST_ID_BODY)
                    .into_element(cx)],
            ),
            shadcn::TabsItem::new(
                "headers",
                "Headers",
                [shadcn::Textarea::new(&locals.headers)
                    .a11y_label("Request headers")
                    .placeholder("Content-Type: application/json")
                    .min_height(Px(260.0))
                    .test_id(TEST_ID_HEADERS)
                    .into_element(cx)],
            ),
        ])
        .ui()
        .w_full()
        .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Request"),
                    shadcn::card_description(format!(
                        "Editing {} {}. Use {{base_url}} to keep environments portable.",
                        method,
                        request_url
                    )),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::h_flex(|cx| ui::children![cx; method_select, url])
                        .gap(Space::N2)
                        .items_center()
                        .w_full(),
                    request_tabs,
                ]
            }),
        ]
    })
    .ui()
    .flex_1()
    .min_w_0()
    .h_full()
    .into_element(cx)
}

fn response_panel(cx: &mut AppUi<'_, '_>, locals: &WorkbenchLocals) -> AnyElement {
    let response_tabs = shadcn::Tabs::new(&locals.response_tab)
        .content_fill_remaining(true)
        .items([
            shadcn::TabsItem::new(
                "pretty",
                "Pretty",
                [shadcn::Textarea::new(&locals.response_pretty)
                    .a11y_label("Pretty response")
                    .disabled(true)
                    .min_height(Px(260.0))
                    .test_id(TEST_ID_RESPONSE_PRETTY)
                    .into_element(cx)],
            ),
            shadcn::TabsItem::new(
                "raw",
                "Raw",
                [shadcn::Textarea::new(&locals.response_raw)
                    .a11y_label("Raw response")
                    .disabled(true)
                    .min_height(Px(260.0))
                    .test_id(TEST_ID_RESPONSE_RAW)
                    .into_element(cx)],
            ),
            shadcn::TabsItem::new(
                "headers",
                "Headers",
                [shadcn::Textarea::new(&locals.response_headers)
                    .a11y_label("Response headers")
                    .disabled(true)
                    .min_height(Px(260.0))
                    .test_id(TEST_ID_RESPONSE_HEADERS)
                    .into_element(cx)],
            ),
        ])
        .ui()
        .w_full()
        .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Response"),
                    shadcn::card_description(
                        "Pretty/raw/headers views stay on the public shadcn Tabs + Textarea lane.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; response_tabs]),
        ]
    })
    .ui()
    .flex_1()
    .min_w_0()
    .h_full()
    .into_element(cx)
}

fn maybe_apply_response_snapshot(
    cx: &mut AppUi<'_, '_>,
    window: WindowId,
    locals: &WorkbenchLocals,
    state: &MutationState<RequestSnapshot, ResponsePayload>,
) {
    let Some(snapshot) = state.input.as_deref() else {
        return;
    };
    let ready = state.is_success() || state.is_error();
    let last_applied = locals.last_applied_seq.layout_value(cx);
    if !ready || snapshot.seq <= last_applied {
        return;
    }

    let (status_line, pretty, raw, headers, timing_line) = if let Some(data) = state.data.as_ref() {
        (
            if data.is_http_error {
                format!("HTTP {} {}", data.status_code, data.status_text)
            } else {
                format!("{} {}", data.status_code, data.status_text)
            },
            data.pretty_body.to_string(),
            data.raw_body.to_string(),
            format!(
                "Resolved URL: {}\n\n{}",
                data.resolved_url, data.header_lines
            ),
            format!("{} ms", data.duration_ms),
        )
    } else {
        let err = state
            .error
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unknown transport failure".to_string());
        (
            "Network error".to_string(),
            err.clone(),
            err.clone(),
            "No response headers captured.".to_string(),
            "n/a".to_string(),
        )
    };

    let history_id = locals.next_history_id.layout_value(cx);
    let entry = HistoryEntry {
        id: history_id,
        snapshot: snapshot.clone(),
        status_line: Arc::from(status_line.clone()),
        timing_line: Arc::from(timing_line.clone()),
    };

    let _ = cx
        .app
        .models_mut()
        .update(locals.response_status.model(), |value: &mut String| {
            *value = status_line
        });
    let _ = cx
        .app
        .models_mut()
        .update(locals.response_pretty.model(), |value: &mut String| {
            *value = pretty
        });
    let _ = cx
        .app
        .models_mut()
        .update(locals.response_raw.model(), |value: &mut String| {
            *value = raw
        });
    let _ = cx
        .app
        .models_mut()
        .update(locals.response_headers.model(), |value: &mut String| {
            *value = headers
        });
    let _ = cx
        .app
        .models_mut()
        .update(locals.history.model(), |items: &mut Vec<HistoryEntry>| {
            items.insert(0, entry);
            if items.len() > 8 {
                items.truncate(8);
            }
        });
    let _ = cx
        .app
        .models_mut()
        .update(locals.next_history_id.model(), |value: &mut u64| {
            *value = value.saturating_add(1);
        });
    let _ = cx.app.models_mut().update(
        locals.selected_history.model(),
        |value: &mut Option<u64>| *value = Some(history_id),
    );
    let _ = cx
        .app
        .models_mut()
        .update(locals.last_applied_seq.model(), |value: &mut u64| {
            *value = snapshot.seq
        });
    cx.app.request_redraw(window);
}

fn install_tokio_spawner(app: &mut App) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("failed to build tokio runtime");
    let rt = Arc::new(rt);
    let spawner: FutureSpawnerHandle = Arc::new(TokioHandleSpawner(rt.handle().clone()));
    app.set_global::<FutureSpawnerHandle>(spawner);
    app.set_global::<TokioRuntimeGlobal>(TokioRuntimeGlobal { _rt: rt });
}

fn install_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

fn install_commands(app: &mut App) {
    app.commands_mut().register(
        CommandId::new(act::SendRequest::ID_STR),
        CommandMeta::new("Send request")
            .with_category("API Workbench")
            .with_description("Submit the current request from the editor surface.")
            .with_keywords(["http", "request", "send", "postman"])
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::Enter,
                        Modifiers {
                            meta: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::All,
                    KeyChord::new(
                        KeyCode::Enter,
                        Modifiers {
                            ctrl: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
            ]),
    );
    app.commands_mut().register(
        CommandId::new(act::OpenSettings::ID_STR),
        CommandMeta::new("Open environments")
            .with_category("API Workbench")
            .with_description("Edit the base URL and auth token used by the request editor.")
            .with_keywords(["settings", "environment", "auth"])
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::Comma,
                        Modifiers {
                            meta: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::All,
                    KeyChord::new(
                        KeyCode::Comma,
                        Modifiers {
                            ctrl: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
            ]),
    );
    fret_app::install_command_default_keybindings_into_keymap(app);
}

async fn run_request(
    snapshot: RequestSnapshot,
    token: CancellationToken,
) -> Result<ResponsePayload, MutationError> {
    if token.is_cancelled() {
        return Err(MutationError::transient("request cancelled before start"));
    }

    let request = snapshot.clone();
    tracing::info!(
        seq = request.seq,
        method = %request.method,
        url = %request.url,
        "api_workbench_lite run_request started"
    );
    let join_result = tokio::time::timeout(
        REQUEST_TIMEOUT,
        tokio::task::spawn_blocking(move || execute_request_blocking(&request)),
    )
    .await
    .map_err(|_| {
        tracing::warn!(
            seq = snapshot.seq,
            timeout_s = REQUEST_TIMEOUT.as_secs(),
            "api_workbench_lite request timed out"
        );
        MutationError::transient("request timed out")
    })?;

    join_result.map_err(|err| MutationError::transient(format!("request task failed: {err}")))?
}

fn execute_request_blocking(snapshot: &RequestSnapshot) -> Result<ResponsePayload, MutationError> {
    let resolved_url = resolve_url(snapshot)?;
    let start = std::time::Instant::now();

    tracing::info!(
        seq = snapshot.seq,
        method = %snapshot.method,
        resolved_url = %resolved_url,
        "api_workbench_lite execute_request_blocking started"
    );

    let mut request = ureq::request(snapshot.method.as_ref(), &resolved_url);
    for (name, value) in parse_request_headers(snapshot)? {
        request = request.set(&name, &value);
    }

    let response = if request_has_body(snapshot.method.as_ref()) {
        request.send_string(snapshot.body.as_ref())
    } else {
        request.call()
    };

    match response {
        Ok(resp) => response_payload_from_ureq_response(resp, resolved_url, start.elapsed(), false),
        Err(ureq::Error::Status(_, resp)) => {
            response_payload_from_ureq_response(resp, resolved_url, start.elapsed(), true)
        }
        Err(ureq::Error::Transport(err)) => Err(MutationError::transient(err.to_string())),
    }
}

fn response_payload_from_ureq_response(
    resp: ureq::Response,
    resolved_url: String,
    duration: Duration,
    is_http_error: bool,
) -> Result<ResponsePayload, MutationError> {
    let status_code = resp.status();
    let status_text = resp.status_text().to_string();

    let mut header_lines: Vec<String> = resp
        .headers_names()
        .into_iter()
        .filter_map(|name| resp.header(&name).map(|value| format!("{name}: {value}")))
        .collect();
    header_lines.sort();

    let content_type = resp.header("Content-Type").unwrap_or_default().to_string();
    let raw_body = resp
        .into_string()
        .map_err(|err| MutationError::transient(format!("failed to read response body: {err}")))?;

    Ok(ResponsePayload {
        status_code,
        status_text: Arc::from(status_text),
        duration_ms: duration.as_millis().min(u128::from(u64::MAX)) as u64,
        header_lines: Arc::from(header_lines.join("\n")),
        pretty_body: Arc::from(pretty_body(&raw_body, &content_type)),
        raw_body: Arc::from(raw_body),
        resolved_url: Arc::from(resolved_url),
        is_http_error,
    })
}

fn parse_request_headers(
    snapshot: &RequestSnapshot,
) -> Result<Vec<(String, String)>, MutationError> {
    let mut headers = Vec::new();
    for line in snapshot
        .headers_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Some((name, value)) = line.split_once(':') else {
            return Err(MutationError::permanent(format!(
                "invalid header line `{line}`; expected `Name: Value`"
            )));
        };
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    let auth_token = snapshot.env.auth_token.trim();
    if !auth_token.is_empty() {
        headers.push(("Authorization".to_string(), auth_token.to_string()));
    }

    Ok(headers)
}

fn resolve_url(snapshot: &RequestSnapshot) -> Result<String, MutationError> {
    let url = snapshot.url.trim();
    if url.is_empty() {
        return Err(MutationError::permanent("request URL is empty"));
    }

    let resolved = url.replace("{{base_url}}", snapshot.env.base_url.trim());
    if !(resolved.starts_with("http://") || resolved.starts_with("https://")) {
        return Err(MutationError::permanent(format!(
            "resolved URL must start with http:// or https://, got `{resolved}`"
        )));
    }
    Ok(resolved)
}

fn request_has_body(method: &str) -> bool {
    !matches!(method, "GET" | "HEAD")
}

fn pretty_body(raw_body: &str, content_type: &str) -> String {
    if content_type.contains("json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_body) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                return pretty;
            }
        }
    }
    raw_body.to_string()
}

fn apply_collection(
    models: &mut fret_runtime::ModelStore,
    locals: &WorkbenchLocals,
    preset_id: u8,
) -> bool {
    let preset = collection_preset(preset_id);
    let mut handled = locals
        .selected_collection
        .set_in(models, Some(Arc::<str>::from(collection_key(preset_id))));
    handled = locals.selected_history.set_in(models, None) && handled;
    handled = locals
        .method
        .set_in(models, Some(Arc::<str>::from(preset.method)))
        && handled;
    handled = locals.url.set_in(models, preset.url.to_string()) && handled;
    handled = locals.headers.set_in(models, preset.headers.to_string()) && handled;
    handled = locals.body.set_in(models, preset.body.to_string()) && handled;
    handled = locals
        .request_tab
        .set_in(models, Some(Arc::<str>::from("body")))
        && handled;
    handled
}

fn load_history(
    models: &mut fret_runtime::ModelStore,
    locals: &WorkbenchLocals,
    history_id: u64,
) -> bool {
    let history = locals.history.value_in_or_default(models);
    let Some(entry) = history.iter().find(|entry| entry.id == history_id).cloned() else {
        return false;
    };

    let mut handled = locals.selected_history.set_in(models, Some(history_id));
    handled = locals.selected_collection.set_in(
        models,
        entry
            .snapshot
            .collection_id
            .map(|id| Arc::<str>::from(collection_key(id))),
    ) && handled;
    handled = locals
        .method
        .set_in(models, Some(entry.snapshot.method.clone()))
        && handled;
    handled = locals.url.set_in(models, entry.snapshot.url.to_string()) && handled;
    handled = locals
        .headers
        .set_in(models, entry.snapshot.headers_text.to_string())
        && handled;
    handled = locals.body.set_in(models, entry.snapshot.body.to_string()) && handled;
    handled = locals
        .request_tab
        .set_in(models, Some(Arc::<str>::from("body")))
        && handled;
    handled
}

fn status_badge(label: &str) -> shadcn::Badge {
    let variant = if label.starts_with("2") {
        shadcn::BadgeVariant::Default
    } else if label.starts_with("HTTP") || label.starts_with("4") || label.starts_with("5") {
        shadcn::BadgeVariant::Destructive
    } else {
        shadcn::BadgeVariant::Secondary
    };
    shadcn::Badge::new(label).variant(variant)
}

fn short_request_label(snapshot: &RequestSnapshot) -> String {
    format!("{} {}", snapshot.method, compact_url(snapshot.url.as_ref()))
}

fn compact_url(url: &str) -> String {
    if let Some(path) = url.strip_prefix("{{base_url}}") {
        return path.to_string();
    }
    url.to_string()
}

fn default_echo_body() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "message": "Hello from Fret",
        "tool": "api-workbench-lite",
        "tags": ["consumer-audit", "first-contact", "postman-like"]
    }))
    .unwrap_or_else(|_| "{\"message\":\"Hello from Fret\"}".to_string())
}

struct CollectionPreset {
    label: &'static str,
    icon: &'static str,
    method: &'static str,
    url: &'static str,
    headers: &'static str,
    body: &'static str,
}

fn collection_preset(id: u8) -> CollectionPreset {
    match id {
        1 => CollectionPreset {
            label: "Teapot Check",
            icon: "lucide.coffee",
            method: "GET",
            url: "{{base_url}}/status/418",
            headers: "Accept: application/json",
            body: "",
        },
        2 => CollectionPreset {
            label: "Delay Probe",
            icon: "lucide.timer-reset",
            method: "GET",
            url: "{{base_url}}/delay/1",
            headers: "Accept: application/json",
            body: "",
        },
        _ => CollectionPreset {
            label: "Echo JSON",
            icon: "lucide.braces",
            method: "POST",
            url: "{{base_url}}/anything",
            headers: "Content-Type: application/json\nX-Fret-Probe: api-workbench-lite",
            body: "{\n  \"message\": \"Hello from Fret\",\n  \"tool\": \"api-workbench-lite\",\n  \"tags\": [\"consumer-audit\", \"first-contact\", \"postman-like\"]\n}",
        },
    }
}

fn collection_key(id: u8) -> &'static str {
    match id {
        1 => "teapot_check",
        2 => "delay_probe",
        _ => "echo_json",
    }
}

fn collection_id_from_key(key: &str) -> Option<u8> {
    match key {
        "echo_json" => Some(0),
        "teapot_check" => Some(1),
        "delay_probe" => Some(2),
        _ => None,
    }
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("api-workbench-lite")
        .window("api-workbench-lite", (1320.0, 860.0))
        .config_files(false)
        .setup((
            install_tokio_spawner,
            install_demo_theme,
            install_commands,
            fret_icons_lucide::app::install,
        ))
        .command_palette(true)
        .view::<ApiWorkbenchLiteView>()?
        .run()
        .map_err(anyhow::Error::from)
}
