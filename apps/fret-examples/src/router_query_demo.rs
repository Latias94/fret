use std::sync::Arc;
use std::time::Duration;

use fret_kit::prelude::*;
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryError, QueryPolicy, QueryState, QueryStatus, with_query_client};
use fret_router::{
    MemoryHistory, NavigationAction, PathParam, RouteHooks, RouteLocation, RouteNode,
    RouteSearchTable, RouteTree, Router, RouterUpdateWithPrefetchIntents, SearchMap,
    SearchValidationMode, prefetch_intent_query_key, route_query_key,
};
use fret_router_ui::{RouterLink, RouterUiStore, router_outlet};
use fret_ui::Invalidation;

const ROUTER_QUERY_DEMO_NAV_NS: &str = "fret-examples.router_query_demo.nav_index.v1";
const ROUTER_QUERY_DEMO_PAGE_NS: &str = "fret-examples.router_query_demo.page_content.v1";

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum RouteId {
    Root,
    Settings,
    User,
}

#[derive(Debug)]
struct PageData {
    label: Arc<str>,
}

struct RouterQueryDemoState {
    router: RouterUiStore<RouteId, MemoryHistory>,
    prefetch_log: Model<Vec<Arc<str>>>,
    msg_router: MessageRouter<RouterQueryDemoMsg>,
}

#[derive(Debug, Clone)]
enum RouterQueryDemoMsg {
    NavigateRoot,
    NavigateSettings,
    NavigateUser,
    Back,
    Forward,
    ClearLog,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("router-query-demo", init_window, view, |d| {
        d.on_command(on_command)
    })?
    .with_main_window("router_query_demo", (680.0, 420.0))
    .init_app(|app| {
        shadcn::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
            shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        );
    })
    .run()?;
    Ok(())
}

fn init_window(app: &mut App, window: AppWindowId) -> RouterQueryDemoState {
    let tree = Arc::new(RouteTree::new(
        RouteNode::new(RouteId::Root, "/")
            .unwrap()
            .with_children(vec![
                RouteNode::new(RouteId::Settings, "settings").unwrap(),
                RouteNode::new(RouteId::User, "users/:id").unwrap(),
            ]),
    ));

    let search_table = Arc::new(RouteSearchTable::new());
    let history = MemoryHistory::new(RouteLocation::parse("/"));
    let mut router =
        Router::new(tree, search_table, SearchValidationMode::Strict, history).expect("router");

    router.route_hooks_mut().insert(
        RouteId::Root,
        RouteHooks {
            before_load: None,
            loader: Some(Arc::new(|_ctx| {
                vec![fret_router::RoutePrefetchIntent {
                    route: RouteId::Root,
                    namespace: ROUTER_QUERY_DEMO_NAV_NS,
                    location: RouteLocation::from_path("/nav"),
                    extra: None,
                }]
            })),
        },
    );

    router.route_hooks_mut().insert(
        RouteId::Settings,
        RouteHooks {
            before_load: None,
            loader: Some(Arc::new(|ctx| {
                vec![fret_router::RoutePrefetchIntent {
                    route: ctx.matched.route,
                    namespace: ROUTER_QUERY_DEMO_PAGE_NS,
                    location: ctx.to.clone(),
                    extra: None,
                }]
            })),
        },
    );

    router.route_hooks_mut().insert(
        RouteId::User,
        RouteHooks {
            before_load: None,
            loader: Some(Arc::new(|ctx| {
                vec![fret_router::RoutePrefetchIntent {
                    route: ctx.matched.route,
                    namespace: ROUTER_QUERY_DEMO_PAGE_NS,
                    location: ctx.to.clone(),
                    extra: Some("user"),
                }]
            })),
        },
    );

    RouterQueryDemoState {
        router: RouterUiStore::new(app, window, router),
        prefetch_log: app.models_mut().insert(Vec::new()),
        msg_router: MessageRouter::new(format!("router-query-demo.{window:?}.")),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut RouterQueryDemoState) -> ViewElements {
    let theme = Theme::global(&*cx.app).clone();
    st.msg_router.clear();

    let snapshot_model = st.router.snapshot_model();
    let snapshot = cx
        .get_model_cloned(&snapshot_model, Invalidation::Layout)
        .expect("router snapshot model should be readable");

    let location_label: Arc<str> = Arc::from(snapshot.location.to_url());
    let last_transition = snapshot
        .last_transition
        .as_ref()
        .map(|t| {
            Arc::from(format!(
                "cause={:?} from={} to={}",
                t.cause,
                t.from.to_url(),
                t.to.to_url()
            ))
        })
        .unwrap_or_else(|| Arc::from("<no transition yet>"));

    let nav_key =
        route_query_key::<PageData>(ROUTER_QUERY_DEMO_NAV_NS, &RouteLocation::from_path("/nav"));
    let page_key = route_query_key::<PageData>(ROUTER_QUERY_DEMO_PAGE_NS, &snapshot.location);
    let policy = QueryPolicy {
        stale_time: Duration::from_secs(60),
        cache_time: Duration::from_secs(5 * 60),
        keep_previous_data_while_loading: true,
        ..Default::default()
    };

    let nav_handle = cx.use_query(nav_key, policy.clone(), |_token| {
        Ok::<PageData, QueryError>(PageData {
            label: Arc::from("nav_index:fallback_fetch"),
        })
    });
    let location_label_for_fetch = location_label.clone();
    let page_handle = cx.use_query(page_key, policy, move |_token| {
        Ok::<PageData, QueryError>(PageData {
            label: Arc::from(format!("page:fallback_fetch:{location_label_for_fetch}")),
        })
    });

    let nav_state = cx
        .watch_model(nav_handle.model())
        .layout()
        .cloned()
        .unwrap_or_else(QueryState::<PageData>::default);
    let page_state = cx
        .watch_model(page_handle.model())
        .layout()
        .cloned()
        .unwrap_or_else(QueryState::<PageData>::default);

    let prefetch_log = cx
        .watch_model(&st.prefetch_log)
        .layout()
        .cloned()
        .unwrap_or_default();

    let status_badge = |cx: &mut ElementContext<'_, App>, status: QueryStatus| {
        let label = match status {
            QueryStatus::Idle => "Idle",
            QueryStatus::Loading => "Loading",
            QueryStatus::Success => "Success",
            QueryStatus::Error => "Error",
        };
        shadcn::Badge::new(label)
            .variant(match status {
                QueryStatus::Success => shadcn::BadgeVariant::Default,
                QueryStatus::Error => shadcn::BadgeVariant::Destructive,
                QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
            })
            .into_element(cx)
    };

    let nav_line: Arc<str> = nav_state
        .data
        .as_ref()
        .map(|d| d.label.clone())
        .unwrap_or_else(|| Arc::from("<no nav data>"));
    let page_line: Arc<str> = page_state
        .data
        .as_ref()
        .map(|d| d.label.clone())
        .unwrap_or_else(|| Arc::from("<no page data>"));

    let nav_row = ui::h_flex(cx, |cx| {
        [
            shadcn::Badge::new("nav")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            status_badge(cx, nav_state.status),
            ui::raw_text(cx, nav_line).into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let page_row = ui::h_flex(cx, |cx| {
        [
            shadcn::Badge::new("page")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            status_badge(cx, page_state.status),
            ui::raw_text(cx, page_line).into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let nav_buttons = ui::h_flex(cx, |cx| {
        [
            shadcn::Button::new("/")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::NavigateRoot))
                .into_element(cx),
            shadcn::Button::new("/settings")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::NavigateSettings))
                .into_element(cx),
            shadcn::Button::new("/users/42")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::NavigateUser))
                .into_element(cx),
            shadcn::Button::new("Back")
                .variant(shadcn::ButtonVariant::Ghost)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::Back))
                .into_element(cx),
            shadcn::Button::new("Forward")
                .variant(shadcn::ButtonVariant::Ghost)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::Forward))
                .into_element(cx),
            shadcn::Button::new("Clear log")
                .variant(shadcn::ButtonVariant::Ghost)
                .on_click(st.msg_router.cmd(RouterQueryDemoMsg::ClearLog))
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let header_lines = ui::v_flex(cx, |cx| {
        [
            ui::h_flex(cx, |cx| {
                [
                    ui::raw_text(cx, format!("location={location_label}")).into_element(cx),
                    shadcn::Button::new("Copy URL")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .on_activate(RouterLink::copy_href_on_activate(location_label.clone()))
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::raw_text(cx, last_transition)
                .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                .into_element(cx),
            ui::h_flex(cx, |cx| {
                [
                    shadcn::Badge::new("leaf")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    router_outlet(cx, &snapshot_model, |cx, snap| {
                        let label = match snap.leaf_route().copied() {
                            Some(RouteId::Root) => "Root",
                            Some(RouteId::Settings) => "Settings",
                            Some(RouteId::User) => "User",
                            None => "<none>",
                        };
                        shadcn::Badge::new(label).into_element(cx)
                    }),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        ]
    })
    .gap(Space::N1)
    .into_element(cx);

    let log_lines = ui::v_flex(cx, |cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        for line in prefetch_log.iter().take(12) {
            out.push(
                ui::raw_text(cx, line.clone())
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
            );
        }
        out
    })
    .gap(Space::N1)
    .into_element(cx);

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Router + Query demo").into_element(cx),
            shadcn::CardDescription::new(
                "Route hooks produce portable prefetch intents; app maps intents to typed QueryKeys and executes prefetch.",
            )
            .into_element(cx),
            header_lines,
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            ui::v_flex(cx, |_cx| [nav_buttons, nav_row, page_row, log_lines])
                .gap(Space::N3)
                .w_full()
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .ui()
    .w_full()
    .max_w(Px(640.0))
    .into_element(cx);

    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_required("background")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    vec![page].into()
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    st: &mut RouterQueryDemoState,
    cmd: &CommandId,
) {
    let Some(msg) = st.msg_router.try_take(cmd) else {
        return;
    };

    let update = match msg {
        RouterQueryDemoMsg::NavigateRoot => st.router.navigate_with_prefetch_intents(
            app,
            NavigationAction::Replace,
            Some(RouteLocation::parse("/")),
        ),
        RouterQueryDemoMsg::NavigateSettings => st.router.navigate_with_prefetch_intents(
            app,
            NavigationAction::Push,
            Some(RouteLocation::parse("/settings")),
        ),
        RouterQueryDemoMsg::NavigateUser => {
            let link = st
                .router
                .link_to(
                    app,
                    NavigationAction::Push,
                    &RouteId::User,
                    &[PathParam {
                        name: "id".to_string(),
                        value: "42".to_string(),
                    }],
                    SearchMap::new()
                        .with_typed("tab", Some("profile".to_string()))
                        .with_typed("debug", Some(true)),
                    None,
                )
                .expect("router should build a user link");
            st.router
                .navigate_with_prefetch_intents(app, link.action, Some(link.to))
        }
        RouterQueryDemoMsg::Back => {
            st.router
                .navigate_with_prefetch_intents(app, NavigationAction::Back, None)
        }
        RouterQueryDemoMsg::Forward => {
            st.router
                .navigate_with_prefetch_intents(app, NavigationAction::Forward, None)
        }
        RouterQueryDemoMsg::ClearLog => {
            let _ = app.models_mut().update(&st.prefetch_log, |v| v.clear());
            app.request_redraw(window);
            return;
        }
    };

    let Ok(update) = update else {
        app.request_redraw(window);
        return;
    };

    let RouterUpdateWithPrefetchIntents { update, intents } = update;

    if !update.changed() {
        app.request_redraw(window);
        return;
    }

    if intents.is_empty() {
        app.request_redraw(window);
        return;
    }

    let _ = app.models_mut().update(&st.prefetch_log, |v| {
        for intent in &intents {
            v.insert(
                0,
                Arc::from(format!(
                    "intent: ns={} loc={} extra={:?}",
                    intent.namespace,
                    intent.location.to_url(),
                    intent.extra
                )),
            );
        }
        v.truncate(64);
    });

    let _ = with_query_client(app, |client, app| {
        for intent in intents {
            let key = prefetch_intent_query_key::<PageData, _>(&intent);
            let policy = QueryPolicy::default();
            let seed: Arc<str> = Arc::from(format!(
                "prefetched:{}:{}:{:?}",
                intent.namespace,
                intent.location.to_url(),
                intent.extra
            ));
            let _ = client.prefetch(app, window, key, policy, move |_token| {
                Ok::<PageData, QueryError>(PageData {
                    label: seed.clone(),
                })
            });
        }
    });

    app.request_redraw(window);
}
