use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_query::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, QueryCancelMode, QueryError, QueryKey,
    QueryPolicy, QuerySnapshotEntry, QueryState, QueryStatus, with_query_client,
};
use fret_ui::element::{PressableA11y, PressableProps};
use fret_ui_kit::declarative::QueryHandleWatchExt as _;
use fret_ui_kit::primitives::scroll_area::ScrollAreaType;
use fret_ui_kit::primitives::separator::SeparatorOrientation;

mod act {
    fret::actions!([
        SelectTip = "async_playground_demo.select.tip.v1",
        SelectSearch = "async_playground_demo.select.search.v1",
        SelectStock = "async_playground_demo.select.stock.v1",
        SelectStatus = "async_playground_demo.select.status.v1",
        ToggleTheme = "async_playground_demo.toggle_theme.v1",
        InvalidateSelected = "async_playground_demo.invalidate_selected.v1",
        CancelSelected = "async_playground_demo.cancel_selected.v1",
        InvalidateNamespace = "async_playground_demo.invalidate_namespace.v1"
    ]);
}

const TRANSIENT_INVALIDATE_SELECTED: u64 = 0xAFA0_1002;
const TRANSIENT_CANCEL_SELECTED: u64 = 0xAFA0_1003;
const TRANSIENT_INVALIDATE_NAMESPACE: u64 = 0xAFA0_1004;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum QueryId {
    Tip,
    Search,
    Stock,
    Status,
}

impl Default for QueryId {
    fn default() -> Self {
        Self::Tip
    }
}

impl QueryId {
    const ALL: [QueryId; 4] = [
        QueryId::Tip,
        QueryId::Search,
        QueryId::Stock,
        QueryId::Status,
    ];

    fn label(self) -> &'static str {
        match self {
            QueryId::Tip => "Random Tip",
            QueryId::Search => "Search",
            QueryId::Stock => "Stock Price",
            QueryId::Status => "System Status",
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            QueryId::Tip => "fret-examples.async_playground.tip.v1",
            QueryId::Search => "fret-examples.async_playground.search.v1",
            QueryId::Stock => "fret-examples.async_playground.stock.v1",
            QueryId::Status => "fret-examples.async_playground.status.v1",
        }
    }
}

#[derive(Debug)]
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

fn install_tokio_spawner(app: &mut KernelApp) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("failed to build tokio runtime");
    let rt = Arc::new(rt);

    let spawner: FutureSpawnerHandle = Arc::new(TokioHandleSpawner(rt.handle().clone()));
    app.set_global::<FutureSpawnerHandle>(spawner);
    app.set_global::<TokioRuntimeGlobal>(TokioRuntimeGlobal { _rt: rt });
}

fn apply_theme(app: &mut KernelApp, dark: bool) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Zinc,
        if dark {
            shadcn::themes::ShadcnColorScheme::Dark
        } else {
            shadcn::themes::ShadcnColorScheme::Light
        },
    );
}

fn install_light_theme(app: &mut KernelApp) {
    apply_theme(app, false);
}

#[derive(Clone)]
struct SelectModel {
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
}

impl SelectModel {
    fn new(app: &mut KernelApp, initial: Option<&'static str>) -> Self {
        Self {
            value: app.models_mut().insert(initial.map(Arc::from)),
            open: app.models_mut().insert(false),
        }
    }
}

struct QueryConfigModels {
    stale_time_s: Model<String>,
    cache_time_s: Model<String>,
    keep_prev: Model<bool>,
    cancel_mode: SelectModel,
    fail_mode: Model<bool>,
}

impl QueryConfigModels {
    fn new(app: &mut KernelApp) -> Self {
        Self {
            stale_time_s: app.models_mut().insert("2".to_string()),
            cache_time_s: app.models_mut().insert("30".to_string()),
            keep_prev: app.models_mut().insert(true),
            cancel_mode: SelectModel::new(app, Some("cancel")),
            fail_mode: app.models_mut().insert(false),
        }
    }
}

#[derive(Debug, Clone)]
struct QueryDiag {
    status: QueryStatus,
    stale: Option<bool>,
    inflight: Option<u64>,
}

impl QueryDiag {
    fn from_state(st: &QueryState<Arc<str>>) -> Self {
        Self {
            status: st.status,
            stale: None,
            inflight: st.inflight,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FetchMode {
    Sync,
    Async,
}

struct AsyncPlaygroundState {
    selected: Model<QueryId>,
    dark: Model<bool>,

    global_slow: Model<bool>,
    tabs: Model<Option<Arc<str>>>,

    namespace_input: Model<String>,
    search_input: Model<String>,
    stock_symbol: Model<String>,

    configs: HashMap<QueryId, QueryConfigModels>,
    last_diag: HashMap<QueryId, QueryDiag>,

    catalog_scroll: fret_ui::scroll::ScrollHandle,
    inspector_scroll: fret_ui::scroll::ScrollHandle,
}

struct AsyncPlaygroundView {
    st: AsyncPlaygroundState,
    applied_dark: bool,
}

fn default_namespace_for_id(id: QueryId) -> &'static str {
    match id {
        QueryId::Tip => "tip",
        QueryId::Search => "search",
        QueryId::Stock => "stock",
        QueryId::Status => "status",
    }
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("async-playground")
        .window("async-playground", (1180.0, 720.0))
        .config_files(false)
        .setup((install_tokio_spawner, install_light_theme))
        .view::<AsyncPlaygroundView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for AsyncPlaygroundView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        let mut configs = HashMap::new();
        for id in QueryId::ALL {
            configs.insert(id, QueryConfigModels::new(app));
        }

        Self {
            applied_dark: false,
            st: AsyncPlaygroundState {
                selected: app.models_mut().insert(QueryId::Tip),
                dark: app.models_mut().insert(false),
                global_slow: app.models_mut().insert(false),
                tabs: app.models_mut().insert(Some(Arc::<str>::from("async"))),
                namespace_input: app.models_mut().insert("tip".to_string()),
                search_input: app.models_mut().insert("react".to_string()),
                stock_symbol: app.models_mut().insert("FRET".to_string()),
                configs,
                last_diag: HashMap::new(),
                catalog_scroll: fret_ui::scroll::ScrollHandle::default(),
                inspector_scroll: fret_ui::scroll::ScrollHandle::default(),
            },
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let dark_for_theme = cx
            .app
            .models()
            .get_copied(&self.st.dark)
            .unwrap_or_default();
        if self.applied_dark != dark_for_theme {
            self.applied_dark = dark_for_theme;
            apply_theme(cx.app, dark_for_theme);
        }

        if cx.effects().take_transient(TRANSIENT_INVALIDATE_SELECTED) {
            let selected = cx
                .app
                .models()
                .get_copied(&self.st.selected)
                .unwrap_or_default();
            let key = query_key_for_selected(cx.app, &self.st, selected);
            let _ = with_query_client(cx.app, |client, app| client.invalidate(app, key));
        }

        if cx.effects().take_transient(TRANSIENT_CANCEL_SELECTED) {
            let selected = cx
                .app
                .models()
                .get_copied(&self.st.selected)
                .unwrap_or_default();
            let key = query_key_for_selected(cx.app, &self.st, selected);
            let _ = with_query_client(cx.app, |client, app| client.cancel_inflight(app, key));
        }

        if cx.effects().take_transient(TRANSIENT_INVALIDATE_NAMESPACE) {
            let ns = cx
                .app
                .models()
                .get_cloned(&self.st.namespace_input)
                .unwrap_or_default();
            let ns = ns.trim();
            if let Some(ns) = map_namespace(ns) {
                let _ = with_query_client(cx.app, |client, _app| client.invalidate_namespace(ns));
            }
        }

        let theme = Theme::global(&*cx.app).snapshot();
        let selected = cx
            .watch_model(&self.st.selected)
            .layout()
            .value_or_default();
        let dark = cx.watch_model(&self.st.dark).layout().value_or_default();
        let global_slow = cx
            .watch_model(&self.st.global_slow)
            .layout()
            .value_or_default();

        let header = header_bar(cx, &mut self.st, theme.clone(), global_slow, dark);
        let body = body(cx, &mut self.st, theme, global_slow, selected);

        cx.actions().models::<act::SelectTip>({
            let selected = self.st.selected.clone();
            let namespace_input = self.st.namespace_input.clone();
            move |models| {
                let _ = models.update(&selected, |v| *v = QueryId::Tip);
                let _ = models.update(&namespace_input, |s| {
                    s.clear();
                    s.push_str(default_namespace_for_id(QueryId::Tip));
                });
                true
            }
        });
        cx.actions().models::<act::SelectSearch>({
            let selected = self.st.selected.clone();
            let namespace_input = self.st.namespace_input.clone();
            move |models| {
                let _ = models.update(&selected, |v| *v = QueryId::Search);
                let _ = models.update(&namespace_input, |s| {
                    s.clear();
                    s.push_str(default_namespace_for_id(QueryId::Search));
                });
                true
            }
        });
        cx.actions().models::<act::SelectStock>({
            let selected = self.st.selected.clone();
            let namespace_input = self.st.namespace_input.clone();
            move |models| {
                let _ = models.update(&selected, |v| *v = QueryId::Stock);
                let _ = models.update(&namespace_input, |s| {
                    s.clear();
                    s.push_str(default_namespace_for_id(QueryId::Stock));
                });
                true
            }
        });
        cx.actions().models::<act::SelectStatus>({
            let selected = self.st.selected.clone();
            let namespace_input = self.st.namespace_input.clone();
            move |models| {
                let _ = models.update(&selected, |v| *v = QueryId::Status);
                let _ = models.update(&namespace_input, |s| {
                    s.clear();
                    s.push_str(default_namespace_for_id(QueryId::Status));
                });
                true
            }
        });

        cx.actions().models::<act::ToggleTheme>({
            let dark = self.st.dark.clone();
            move |models| models.update(&dark, |v| *v = !*v).is_ok()
        });

        cx.actions()
            .transient::<act::InvalidateSelected>(TRANSIENT_INVALIDATE_SELECTED);
        cx.actions()
            .transient::<act::CancelSelected>(TRANSIENT_CANCEL_SELECTED);
        cx.actions()
            .transient::<act::InvalidateNamespace>(TRANSIENT_INVALIDATE_NAMESPACE);

        ui::v_flex(|_cx| [header, body])
            .w_full()
            .h_full()
            .into_element(cx)
            .into()
    }
}

fn header_bar(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    global_slow: bool,
    dark: bool,
) -> AnyElement {
    let title = ui::text("Async Playground")
        .text_sm()
        .font_semibold()
        .truncate()
        .into_element(cx);

    let slow_label = ui::text("Slow network (x2)")
        .text_sm()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .into_element(cx);
    let slow_switch = shadcn::Switch::new(st.global_slow.clone())
        .a11y_label("Simulate slow network")
        .into_element(cx);
    let slow_row = ui::h_flex(|_cx| [slow_label, slow_switch])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let theme_btn = shadcn::Button::new(if dark { "Light" } else { "Dark" })
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .action(act::ToggleTheme)
        .into_element(cx);

    let right = ui::h_flex(|_cx| [slow_row, theme_btn])
        .gap(Space::N4)
        .items_center()
        .into_element(cx);

    let spacer = ui::container(|_cx| Vec::<AnyElement>::new())
        .flex_grow(1.0)
        .into_element(cx);

    ui::h_flex(|_cx| [title, spacer, right])
        .px(Space::N6)
        .py(Space::N3)
        .bg(ColorRef::Color(theme.color_token("card")))
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .into_element(cx)
        .test_id(format!("async-playground.header.slow={global_slow}"))
}

fn body(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    global_slow: bool,
    selected: QueryId,
) -> AnyElement {
    let left = catalog_panel(cx, st, theme.clone(), selected);
    let mid = main_panel(cx, st, theme.clone(), global_slow, selected);
    let right = inspector_panel(cx, st, theme, selected);

    let sep_1 = shadcn::Separator::new()
        .orientation(SeparatorOrientation::Vertical)
        .flex_stretch_cross_axis(true)
        .into_element(cx);
    let sep_2 = shadcn::Separator::new()
        .orientation(SeparatorOrientation::Vertical)
        .flex_stretch_cross_axis(true)
        .into_element(cx);

    ui::h_flex(|_cx| [left, sep_1, mid, sep_2, right])
        .w_full()
        .h_full()
        .items_stretch()
        .into_element(cx)
}

fn catalog_panel(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    selected: QueryId,
) -> AnyElement {
    let catalog_scroll = st.catalog_scroll.clone();
    let header = ui::text("Catalog")
        .font_semibold()
        .text_sm()
        .into_element(cx);
    let header_row = ui::container(|_cx| vec![header])
        .px(Space::N4)
        .py(Space::N3)
        .bg(ColorRef::Color(theme.color_token("card")))
        .into_element(cx);

    let list = shadcn::ScrollArea::build(|cx, out| {
        out.push_ui(
            cx,
            ui::v_flex_build(|cx, out| {
                for id in QueryId::ALL {
                    out.push(catalog_item(cx, st, theme.clone(), selected, id));
                }
            })
            .gap(Space::N1)
            .p(Space::N2)
            .w_full()
            .items_stretch(),
        );
    })
    .scroll_handle(catalog_scroll)
    .type_(ScrollAreaType::Hover)
    .refine_layout(LayoutRefinement::default().size_full())
    .into_element(cx);

    ui::v_flex(|_cx| [header_row, list])
        .w_px(Px(288.0))
        .h_full()
        .bg(ColorRef::Color(theme.color_token("muted")))
        .into_element(cx)
}

fn catalog_item(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    selected: QueryId,
    id: QueryId,
) -> AnyElement {
    let selected = selected == id;
    let select_cmd = select_command_for_id(id);
    let diag = st.last_diag.get(&id).cloned();

    let bg_idle = theme.color_token("muted");
    let bg_selected = theme.color_token("background");
    let bg_hover = theme.color_token("card");

    cx.pressable(
        PressableProps {
            enabled: true,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::from(id.label())),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st_press| {
            cx.pressable_dispatch_command_if_enabled(select_cmd);

            let bg = if selected {
                bg_selected
            } else if st_press.pressed || st_press.hovered {
                bg_hover
            } else {
                bg_idle
            };

            let title = ui::text(id.label())
                .font_medium()
                .text_sm()
                .truncate()
                .into_element(cx);
            let badge = status_badge(cx, diag.as_ref());

            let row = ui::h_flex(|cx| {
                let spacer = ui::container(|_cx| Vec::<AnyElement>::new())
                    .flex_grow(1.0)
                    .into_element(cx);
                [title, spacer, badge]
            })
            .items_center()
            .into_element(cx);

            vec![
                ui::container(|_cx| vec![row])
                    .bg(ColorRef::Color(bg))
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .rounded_md()
                    .p(Space::N2)
                    .w_full()
                    .into_element(cx),
            ]
        },
    )
}

fn select_command_for_id(id: QueryId) -> CommandId {
    match id {
        QueryId::Tip => act::SelectTip.into(),
        QueryId::Search => act::SelectSearch.into(),
        QueryId::Stock => act::SelectStock.into(),
        QueryId::Status => act::SelectStatus.into(),
    }
}

fn main_panel(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    global_slow: bool,
    selected: QueryId,
) -> AnyElement {
    let mode = active_mode(cx, st);

    let title = ui::text(selected.label())
        .font_semibold()
        .text_sm()
        .into_element(cx);

    let cancel = shadcn::Button::new("Cancel")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .action(act::CancelSelected)
        .into_element(cx);
    let invalidate = shadcn::Button::new("Invalidate")
        .variant(shadcn::ButtonVariant::Default)
        .size(shadcn::ButtonSize::Sm)
        .action(act::InvalidateSelected)
        .into_element(cx);
    let actions = ui::h_flex(|_cx| [cancel, invalidate])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let header_row = ui::h_flex(|cx| {
        let spacer = ui::container(|_cx| Vec::<AnyElement>::new())
            .flex_grow(1.0)
            .into_element(cx);
        [title, spacer, actions]
    })
    .px(Space::N4)
    .py(Space::N3)
    .bg(ColorRef::Color(theme.color_token("card")))
    .border_color(ColorRef::Color(theme.color_token("border")))
    .items_center()
    .into_element(cx);

    let callout = shadcn::Alert::new([
        shadcn::AlertTitle::new("Stale != Polling").into_element(cx),
        shadcn::AlertDescription::new(Arc::<str>::from(
            "Stale-by-time does not automatically refetch. Refetch happens on (re)mount, or when explicitly invalidated.",
        ))
        .into_element(cx),
    ])
    .ui()
    .w_full()
    .into_element(cx);

    let sync_panel = if mode == FetchMode::Sync {
        query_panel_for_mode(
            cx,
            st,
            theme.clone(),
            global_slow,
            selected,
            FetchMode::Sync,
        )
    } else {
        ui::container(|_cx| Vec::<AnyElement>::new())
            .h_full()
            .into_element(cx)
    };
    let async_panel = if mode == FetchMode::Async {
        query_panel_for_mode(
            cx,
            st,
            theme.clone(),
            global_slow,
            selected,
            FetchMode::Async,
        )
    } else {
        ui::container(|_cx| Vec::<AnyElement>::new())
            .h_full()
            .into_element(cx)
    };

    let tabs = shadcn::Tabs::new(st.tabs.clone())
        .content_fill_remaining(true)
        .items([
            shadcn::TabsItem::new("sync", "Sync", [sync_panel]),
            shadcn::TabsItem::new("async", "Async (tokio)", [async_panel]),
        ])
        .ui()
        .w_full()
        .into_element(cx);

    let content = ui::v_flex(|_cx| [callout, tabs])
        .gap(Space::N3)
        .p(Space::N4)
        .w_full()
        .h_full()
        .items_stretch()
        .into_element(cx);

    let scroll = shadcn::ScrollArea::new([content])
        .type_(ScrollAreaType::Hover)
        .refine_layout(LayoutRefinement::default().size_full())
        .into_element(cx);

    ui::v_flex(|_cx| [header_row, scroll])
        .flex_grow(1.0)
        .h_full()
        .bg(ColorRef::Color(theme.color_token("background")))
        .into_element(cx)
}

fn inspector_panel(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    selected: QueryId,
) -> AnyElement {
    let policy = query_policy(cx, st, selected);
    let key = query_key_for_id(cx, st, selected);
    let snap = snapshot_entry_for_key(cx, key);

    let status = snap.as_ref().map(|s| s.status).unwrap_or(QueryStatus::Idle);
    let stale = snap.as_ref().map(|s| s.stale);

    let summary = ui::v_flex_build(|cx, out| {
        out.push(ui::text(key.namespace()).text_xs().into_element(cx));
        out.push(ui::text(key.hash().to_string()).text_xs().into_element(cx));
        out.push(
            ui::text(format!("status: {status:?}"))
                .text_xs()
                .into_element(cx),
        );
        if let Some(stale) = stale {
            out.push(
                ui::text(format!("stale: {stale}"))
                    .text_xs()
                    .into_element(cx),
            );
        }
        out.push(
            ui::text(format!(
                "policy: stale={}s, cache={}s, keep_prev={}, cancel_mode={:?}",
                policy.stale_time.as_secs(),
                policy.cache_time.as_secs(),
                policy.keep_previous_data_while_loading,
                policy.cancel_mode
            ))
            .text_xs()
            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
            .into_element(cx),
        );
    })
    .gap(Space::N1)
    .w_full()
    .items_stretch()
    .into_element(cx);

    let policy_editor = policy_editor(cx, st, theme.clone(), selected);

    let ns_row = ui::h_flex(|cx| {
        let input = shadcn::Input::new(st.namespace_input.clone())
            .placeholder("tip/search/stock/status")
            .refine_layout(LayoutRefinement::default().flex_grow(1.0))
            .into_element(cx);
        let btn = shadcn::Button::new("Invalidate ns")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .action(act::InvalidateNamespace)
            .into_element(cx);
        [input, btn]
    })
    .gap(Space::N2)
    .items_center()
    .w_full()
    .into_element(cx);

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Inspector").into_element(cx),
            shadcn::CardDescription::new(Arc::<str>::from(
                "Snapshot + policy controls (selected query only).",
            ))
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([summary, ns_row, policy_editor]).into_element(cx),
    ])
    .ui()
    .w_full()
    .into_element(cx);

    let scroll = shadcn::ScrollArea::new([card])
        .scroll_handle(st.inspector_scroll.clone())
        .type_(ScrollAreaType::Hover)
        .refine_layout(LayoutRefinement::default().size_full())
        .into_element(cx);

    ui::v_flex(|_cx| [scroll])
        .w_px(Px(320.0))
        .h_full()
        .bg(ColorRef::Color(theme.color_token("muted")))
        .into_element(cx)
}

fn policy_editor(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    id: QueryId,
) -> AnyElement {
    let config = st.configs.get(&id).expect("missing config");

    let stale = shadcn::Input::new(config.stale_time_s.clone())
        .placeholder("stale_time (s)")
        .into_element(cx);
    let cache = shadcn::Input::new(config.cache_time_s.clone())
        .placeholder("cache_time (s)")
        .into_element(cx);

    let keep_prev_label = ui::text("keepPreviousDataWhileLoading")
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .into_element(cx);
    let keep_prev = shadcn::Switch::new(config.keep_prev.clone())
        .a11y_label("Keep previous data while loading")
        .into_element(cx);

    let fail_label = ui::text("fail mode")
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .into_element(cx);
    let fail = shadcn::Switch::new(config.fail_mode.clone())
        .a11y_label("Force failures")
        .into_element(cx);

    let cancel_mode = shadcn::Select::new(
        config.cancel_mode.value.clone(),
        config.cancel_mode.open.clone(),
    )
    .a11y_label("Cancel mode")
    .value(shadcn::SelectValue::new().placeholder("Cancel mode"))
    .items([
        shadcn::SelectItem::new("cancel", "Cancel inflight"),
        shadcn::SelectItem::new("keep", "Keep inflight"),
    ])
    .into_element(cx);

    ui::v_flex(|cx| {
        [
            shadcn::Separator::new().into_element(cx),
            ui::text("Policy")
                .font_semibold()
                .text_sm()
                .into_element(cx),
            ui::h_flex(|_cx| [stale, cache])
                .gap(Space::N2)
                .into_element(cx),
            ui::h_flex(|_cx| [keep_prev_label, keep_prev, fail_label, fail])
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
            cancel_mode,
        ]
    })
    .gap(Space::N2)
    .into_element(cx)
}

fn query_panel_for_mode(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    global_slow: bool,
    selected: QueryId,
    mode: FetchMode,
) -> AnyElement {
    let id = selected;
    let policy = query_policy(cx, st, id);
    let fail_mode = query_fail_mode(cx, st, id);
    let key = query_key_for_id(cx, st, id);

    let base_delay = match id {
        QueryId::Tip => Duration::from_millis(240),
        QueryId::Search => Duration::from_millis(650),
        QueryId::Stock => Duration::from_millis(450),
        QueryId::Status => Duration::from_millis(280),
    };
    let delay = if global_slow {
        base_delay + base_delay
    } else {
        base_delay
    };

    let handle = match mode {
        FetchMode::Sync => {
            let search = cx.watch_model(&st.search_input).layout().value_or_default();
            let symbol = cx.watch_model(&st.stock_symbol).layout().value_or_default();
            cx.data().query(key, policy.clone(), move |token| {
                mock_fetch_sync(token, id, delay, fail_mode, search, symbol)
            })
        }
        FetchMode::Async => {
            let search = cx.watch_model(&st.search_input).layout().value_or_default();
            let symbol = cx.watch_model(&st.stock_symbol).layout().value_or_default();
            cx.data()
                .query_async(key, policy.clone(), move |token| async move {
                    mock_fetch_async(token, id, delay, fail_mode, search, symbol).await
                })
        }
    };

    let state = handle
        .layout_query(cx)
        .value_or_else(QueryState::<Arc<str>>::default);

    let snap = snapshot_entry_for_key(cx, key);
    observe_query_diag(st, id, &state, snap.as_ref());

    let inputs = query_inputs_row(cx, st, theme.clone(), id);
    let view = query_result_view(cx, theme, id, key, &state, snap.as_ref(), &policy);
    ui::v_flex(|_cx| [inputs, view])
        .gap(Space::N4)
        .w_full()
        .items_stretch()
        .into_element(cx)
}

fn query_inputs_row(
    cx: &mut UiCx<'_>,
    st: &mut AsyncPlaygroundState,
    theme: ThemeSnapshot,
    id: QueryId,
) -> AnyElement {
    let mut children: Vec<AnyElement> = Vec::new();
    children.push(
        ui::text(match id {
            QueryId::Tip | QueryId::Status => "No params (key is stable).",
            QueryId::Search => "Type to change key and trigger a new query.",
            QueryId::Stock => "Change symbol to create a new key.",
        })
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .into_element(cx),
    );

    match id {
        QueryId::Search => {
            children.push(
                shadcn::Input::new(st.search_input.clone())
                    .placeholder("Search query…")
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            );
        }
        QueryId::Stock => {
            children.push(
                shadcn::Input::new(st.stock_symbol.clone())
                    .placeholder("Symbol…")
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            );
        }
        QueryId::Tip | QueryId::Status => {}
    }

    ui::v_flex(|_cx| children)
        .gap(Space::N2)
        .w_full()
        .items_stretch()
        .into_element(cx)
}

fn query_result_view(
    cx: &mut UiCx<'_>,
    theme: ThemeSnapshot,
    id: QueryId,
    key: QueryKey<Arc<str>>,
    state: &QueryState<Arc<str>>,
    snap: Option<&QuerySnapshotEntry>,
    policy: &QueryPolicy,
) -> AnyElement {
    let stale = snap.map(|s| s.stale).unwrap_or(false);
    let badge = status_badge(
        cx,
        Some(&QueryDiag {
            stale: Some(stale),
            ..QueryDiag::from_state(state)
        }),
    );

    let meta = ui::h_flex(|cx| {
        let left = ui::text(id.namespace())
            .text_xs()
            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
            .into_element(cx);
        let right = ui::text(key.hash().to_string())
            .text_xs()
            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
            .into_element(cx);
        [left, right]
    })
    .justify_between()
    .into_element(cx);

    let body = match state.status {
        QueryStatus::Idle => ui::text("Idle (not fetched yet).")
            .text_sm()
            .into_element(cx),
        QueryStatus::Loading => {
            let kept = policy.keep_previous_data_while_loading && state.data.is_some();
            ui::text(if kept {
                "Loading… (keepPreviousDataWhileLoading=true)"
            } else {
                "Loading…"
            })
            .text_sm()
            .into_element(cx)
        }
        QueryStatus::Error => {
            let msg = state
                .error
                .as_ref()
                .map(|e| e.message().clone())
                .unwrap_or_else(|| Arc::from("<no error message>"));
            shadcn::Alert::new([
                shadcn::AlertTitle::new("Query error").into_element(cx),
                shadcn::AlertDescription::new(msg).into_element(cx),
            ])
            .variant(shadcn::AlertVariant::Destructive)
            .into_element(cx)
        }
        QueryStatus::Success => ui::text(
            state
                .data
                .as_deref()
                .cloned()
                .unwrap_or_else(|| Arc::from("<no data>")),
        )
        .text_sm()
        .into_element(cx),
    };

    let header = ui::h_flex(|cx| {
        let title = ui::text("Result")
            .font_semibold()
            .text_sm()
            .into_element(cx);
        let spacer = ui::container(|_cx| Vec::<AnyElement>::new())
            .flex_grow(1.0)
            .into_element(cx);
        [title, spacer, badge]
    })
    .items_center()
    .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([header, meta]).into_element(cx),
        shadcn::CardContent::new([body]).into_element(cx),
    ])
    .ui()
    .w_full()
    .into_element(cx)
}

fn active_mode(cx: &mut UiCx<'_>, st: &AsyncPlaygroundState) -> FetchMode {
    let tab = cx.watch_model(&st.tabs).layout().value_or_default();
    match tab.as_deref() {
        Some("sync") => FetchMode::Sync,
        _ => FetchMode::Async,
    }
}

fn query_policy(cx: &mut UiCx<'_>, st: &AsyncPlaygroundState, id: QueryId) -> QueryPolicy {
    let config = st.configs.get(&id).expect("missing config");
    let stale_s = cx
        .watch_model(&config.stale_time_s)
        .layout()
        .value_or_default();
    let cache_s = cx
        .watch_model(&config.cache_time_s)
        .layout()
        .value_or_default();
    let keep_prev = cx
        .watch_model(&config.keep_prev)
        .layout()
        .value_or_default();

    let cancel_mode = cx
        .watch_model(&config.cancel_mode.value)
        .layout()
        .value_or_default()
        .unwrap_or_else(|| Arc::<str>::from("cancel"));
    let cancel_mode = match cancel_mode.as_ref() {
        "keep" => QueryCancelMode::KeepInFlight,
        _ => QueryCancelMode::CancelInFlight,
    };

    QueryPolicy {
        stale_time: Duration::from_secs(parse_u64_or(&stale_s, 2)),
        cache_time: Duration::from_secs(parse_u64_or(&cache_s, 30)),
        keep_previous_data_while_loading: keep_prev,
        cancel_mode,
        ..Default::default()
    }
}

fn query_fail_mode(cx: &mut UiCx<'_>, st: &AsyncPlaygroundState, id: QueryId) -> bool {
    let config = st.configs.get(&id).expect("missing config");
    cx.watch_model(&config.fail_mode)
        .layout()
        .value_or_default()
}

fn parse_u64_or(s: &str, fallback: u64) -> u64 {
    s.trim().parse::<u64>().unwrap_or(fallback)
}

fn query_key_for_selected(
    app: &KernelApp,
    st: &AsyncPlaygroundState,
    selected: QueryId,
) -> QueryKey<Arc<str>> {
    let search = app
        .models()
        .get_cloned(&st.search_input)
        .unwrap_or_default();
    let symbol = app
        .models()
        .get_cloned(&st.stock_symbol)
        .unwrap_or_default();
    query_key_for_params(selected, search, symbol)
}

fn query_key_for_id(
    cx: &mut UiCx<'_>,
    st: &AsyncPlaygroundState,
    id: QueryId,
) -> QueryKey<Arc<str>> {
    let search = cx.watch_model(&st.search_input).layout().value_or_default();
    let symbol = cx.watch_model(&st.stock_symbol).layout().value_or_default();
    query_key_for_params(id, search, symbol)
}

fn query_key_for_params(id: QueryId, search: String, symbol: String) -> QueryKey<Arc<str>> {
    match id {
        QueryId::Tip => QueryKey::new(id.namespace(), &("tip",)),
        QueryId::Search => QueryKey::new(id.namespace(), &search),
        QueryId::Stock => QueryKey::new(id.namespace(), &symbol),
        QueryId::Status => QueryKey::new(id.namespace(), &("status",)),
    }
}

fn map_namespace(ns: &str) -> Option<&'static str> {
    match ns {
        "tip" => Some(QueryId::Tip.namespace()),
        "search" => Some(QueryId::Search.namespace()),
        "stock" => Some(QueryId::Stock.namespace()),
        "status" => Some(QueryId::Status.namespace()),
        _ => None,
    }
}

fn observe_query_diag(
    st: &mut AsyncPlaygroundState,
    id: QueryId,
    state: &QueryState<Arc<str>>,
    snap: Option<&QuerySnapshotEntry>,
) {
    let mut diag = QueryDiag::from_state(state);
    diag.stale = snap.map(|s| s.stale);
    st.last_diag.insert(id, diag);
}

fn status_badge(cx: &mut UiCx<'_>, diag: Option<&QueryDiag>) -> AnyElement {
    let Some(diag) = diag else {
        return shadcn::Badge::new("Not mounted")
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx);
    };

    let mut label = format!("{:?}", diag.status);
    if diag.stale == Some(true) {
        label.push_str(" (stale)");
    }

    let variant = match diag.status {
        QueryStatus::Success => shadcn::BadgeVariant::Default,
        QueryStatus::Error => shadcn::BadgeVariant::Destructive,
        QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
    };

    shadcn::Badge::new(label).variant(variant).into_element(cx)
}

fn snapshot_entry_for_key(
    cx: &mut UiCx<'_>,
    key: QueryKey<Arc<str>>,
) -> Option<QuerySnapshotEntry> {
    let type_name = std::any::type_name::<Arc<str>>();
    with_query_client(cx.app, |client, _app| client.snapshot()).and_then(|snap| {
        snap.entries.into_iter().find(|e| {
            e.namespace == key.namespace() && e.hash == key.hash() && e.type_name == type_name
        })
    })
}

fn sleep_sync(token: &CancellationToken, dur: Duration) -> Result<(), QueryError> {
    let mut remaining = dur;
    let step = Duration::from_millis(25);
    while remaining > Duration::ZERO {
        if token.is_cancelled() {
            return Err(QueryError::transient("cancelled"));
        }
        let d = step.min(remaining);
        std::thread::sleep(d);
        remaining = remaining.saturating_sub(d);
    }
    Ok(())
}

async fn sleep_async(token: &CancellationToken, dur: Duration) -> Result<(), QueryError> {
    let mut remaining = dur;
    let step = Duration::from_millis(25);
    while remaining > Duration::ZERO {
        if token.is_cancelled() {
            return Err(QueryError::transient("cancelled"));
        }
        let d = step.min(remaining);
        tokio::time::sleep(d).await;
        remaining = remaining.saturating_sub(d);
    }
    Ok(())
}

static TIP_SEQ: AtomicU64 = AtomicU64::new(1);
static STOCK_SEQ: AtomicU64 = AtomicU64::new(1);

fn mock_fetch_sync(
    token: CancellationToken,
    id: QueryId,
    delay: Duration,
    fail_mode: bool,
    search: String,
    symbol: String,
) -> Result<Arc<str>, QueryError> {
    sleep_sync(&token, delay)?;
    if fail_mode {
        return Err(QueryError::transient("forced failure (demo)"));
    }
    Ok(mock_payload(id, search, symbol))
}

async fn mock_fetch_async(
    token: CancellationToken,
    id: QueryId,
    delay: Duration,
    fail_mode: bool,
    search: String,
    symbol: String,
) -> Result<Arc<str>, QueryError> {
    sleep_async(&token, delay).await?;
    if fail_mode {
        return Err(QueryError::transient("forced failure (demo)"));
    }
    Ok(mock_payload(id, search, symbol))
}

fn mock_payload(id: QueryId, search: String, symbol: String) -> Arc<str> {
    match id {
        QueryId::Tip => {
            let tips = [
                "Invalidate is an explicit user intent.",
                "Stale-by-time does not mean auto-refetch.",
                "Cancellation is cooperative via CancellationToken.",
            ];
            let n = TIP_SEQ.fetch_add(1, Ordering::Relaxed) as usize;
            Arc::from(format!("{} (fetch#{n})", tips[n % tips.len()]))
        }
        QueryId::Search => Arc::from(format!("Results for: {search}")),
        QueryId::Stock => {
            let n = STOCK_SEQ.fetch_add(1, Ordering::Relaxed) as f64;
            let price = 120.0 + (n % 10.0);
            Arc::from(format!("{symbol}: {price:.2}"))
        }
        QueryId::Status => Arc::from("ok=true, queue_depth=0"),
    }
}
