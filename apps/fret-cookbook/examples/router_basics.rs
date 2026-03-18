use std::sync::Arc;

use fret::advanced::AppUiRawActionNotifyExt as _;
use fret::app::prelude::*;
use fret::router::{
    MemoryHistory, NavigationAction, PathPattern, RouteCodec, RouteHooks, RouteLocation, RouteNode,
    RoutePrefetchIntent, RouteSearchTable, RouteTree, Router, RouterOutlet, RouterUiStore,
    SearchValidationMode, router_link_to_typed_route_with_test_id, router_link_with_test_id,
};
use fret::style::{LayoutRefinement, Space};
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([
        RouterBack = "router.back",
        RouterForward = "router.forward",
        ClearIntents = "cookbook.router_basics.clear_intents.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.router_basics.root";
const TEST_ID_LOCATION_LABEL: &str = "cookbook.router_basics.location.label";
const TEST_ID_BTN_BACK: &str = "cookbook.router_basics.back.button";
const TEST_ID_BTN_FORWARD: &str = "cookbook.router_basics.forward.button";
const TEST_ID_BTN_CLEAR_INTENTS: &str = "cookbook.router_basics.intents.clear.button";
const TEST_ID_LINK_HOME: &str = "cookbook.router_basics.link.home";
const TEST_ID_LINK_SETTINGS: &str = "cookbook.router_basics.link.settings";
const TEST_ID_LINK_USER_42: &str = "cookbook.router_basics.link.user_42";
const TEST_ID_LINK_MISSING: &str = "cookbook.router_basics.link.missing";
const TEST_ID_OUTLET: &str = "cookbook.router_basics.outlet";
const TEST_ID_INTENTS_ROOT: &str = "cookbook.router_basics.intents.root";

const PREFETCH_NAV_NS: &str = "fret-cookbook.router_basics.nav.v1";
const PREFETCH_PAGE_NS: &str = "fret-cookbook.router_basics.page.v1";

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum RouteId {
    Home,
    Settings,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AppRoute {
    Home,
    Settings,
    User { id: Arc<str> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppRouteDecodeError {
    NoMatch,
    MissingUserId,
}

struct RouterBasicsRouteCodec;

impl RouteCodec for RouterBasicsRouteCodec {
    type Route = AppRoute;
    type Error = AppRouteDecodeError;

    fn encode(&self, route: &Self::Route) -> RouteLocation {
        match route {
            AppRoute::Home => RouteLocation::from_path("/"),
            AppRoute::Settings => RouteLocation::from_path("/settings"),
            AppRoute::User { id } => RouteLocation::from_path(format!("/users/{id}")),
        }
    }

    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
        match location.path.as_str() {
            "/" => Ok(AppRoute::Home),
            "/settings" => Ok(AppRoute::Settings),
            _ => {
                let pattern = PathPattern::parse("/users/:id").expect("pattern should parse");
                let matched = pattern
                    .match_path(location.path.as_str())
                    .ok_or(AppRouteDecodeError::NoMatch)?;
                let id = matched
                    .param("id")
                    .ok_or(AppRouteDecodeError::MissingUserId)?;
                Ok(AppRoute::User { id: Arc::from(id) })
            }
        }
    }
}

const APP_ROUTE_CODEC: RouterBasicsRouteCodec = RouterBasicsRouteCodec;

struct RouterBasicsView {
    store: RouterUiStore<RouteId, MemoryHistory>,
}

impl View for RouterBasicsView {
    fn init(app: &mut App, window: WindowId) -> Self {
        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Home, "/")
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
            RouteId::Home,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|_ctx| {
                    vec![RoutePrefetchIntent {
                        route: RouteId::Home,
                        namespace: PREFETCH_NAV_NS,
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
                    vec![RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: PREFETCH_PAGE_NS,
                        location: ctx.to.clone(),
                        extra: Some("settings"),
                    }]
                })),
            },
        );
        router.route_hooks_mut().insert(
            RouteId::User,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    vec![RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: PREFETCH_PAGE_NS,
                        location: ctx.to.clone(),
                        extra: Some("user"),
                    }]
                })),
            },
        );

        let store = RouterUiStore::new(app, window, router);
        let _ = store.init_with_prefetch_intents(app);

        Self { store }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let snapshot_model = self.store.snapshot_model();
        let intents_model = self.store.intents_model();
        let intents_weak_for_clear = intents_model.downgrade();

        cx.on_action_notify::<act::RouterBack>(self.store.back_on_action());
        cx.on_action_notify::<act::RouterForward>(self.store.forward_on_action());
        cx.actions().models::<act::ClearIntents>({
            let intents = intents_weak_for_clear;
            move |models| {
                let Some(intents) = intents.upgrade() else {
                    return true;
                };
                models.update(&intents, |v| v.clear()).is_ok()
            }
        });

        cx.actions()
            .availability::<act::RouterBack>(|_host, _acx| CommandAvailability::Available);
        cx.actions()
            .availability::<act::RouterForward>(|_host, _acx| CommandAvailability::Available);
        cx.actions()
            .availability::<act::ClearIntents>(|_host, _acx| CommandAvailability::Available);

        let snapshot = snapshot_model
            .layout(cx)
            .value()
            .expect("router snapshot should be readable");
        let intents = intents_model.layout(cx).value_or_default();
        let typed_route_label = APP_ROUTE_CODEC
            .decode_canonical(&snapshot.location)
            .map(|route| format!("{route:?}"))
            .unwrap_or_else(|_| "<unmatched>".to_string());

        let location_label: Arc<str> = Arc::from(snapshot.location.to_url());
        let can_back = self.store.can_navigate(cx.app, NavigationAction::Back);
        let can_forward = self.store.can_navigate(cx.app, NavigationAction::Forward);

        let back = shadcn::Button::new("Back")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(!can_back)
            .action(act::RouterBack)
            .test_id(TEST_ID_BTN_BACK);

        let forward = shadcn::Button::new("Forward")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(!can_forward)
            .action(act::RouterForward)
            .test_id(TEST_ID_BTN_FORWARD);

        let location = cx.text(location_label).test_id(TEST_ID_LOCATION_LABEL);
        let typed_route = cx.text(format!("typed={typed_route_label}"));

        let header_row = ui::h_flex(|cx| ui::children![cx; back, forward, location, typed_route])
            .gap(Space::N2)
            .items_center()
            .wrap();

        let home_label = cx.text("Home");
        let settings_label = cx.text("Settings");
        let user_label = cx.text("User 42");
        let missing_label = cx.text("Missing");

        let home_link = router_link_to_typed_route_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &APP_ROUTE_CODEC,
            &AppRoute::Home,
            TEST_ID_LINK_HOME,
            [home_label],
        );

        let settings_link = router_link_to_typed_route_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &APP_ROUTE_CODEC,
            &AppRoute::Settings,
            TEST_ID_LINK_SETTINGS,
            [settings_label],
        );

        let user_link = router_link_to_typed_route_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &APP_ROUTE_CODEC,
            &AppRoute::User {
                id: Arc::from("42"),
            },
            TEST_ID_LINK_USER_42,
            [user_label],
        );

        let missing_link = {
            let link = self
                .store
                .link_to_location(NavigationAction::Push, RouteLocation::parse("/missing"));
            router_link_with_test_id(cx, &self.store, link, TEST_ID_LINK_MISSING, [missing_label])
        };

        let nav = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Nav"),
                        shadcn::card_description(
                            "Typed-route links (codec encode + hover prefetch + click navigate).",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![cx;
                        ui::v_flex(|cx| {
                            ui::children![cx; home_link, settings_link, user_link, missing_link]
                        })
                        .gap(Space::N2)
                    ]
                }),
            ]
        })
        .ui()
        .w_px(Px(200.0));

        let outlet = RouterOutlet::new(snapshot_model.clone())
            .test_id(TEST_ID_OUTLET)
            .into_element_by_leaf(
                cx,
                |_cx, route, snap| {
                    let leaf = snap.leaf_match().expect("leaf match should exist");
                    let matched_path = leaf.matched_path.clone();
                    let params = leaf.params.clone();
                    let depth = snap.match_depth();

                    let title = match route {
                        RouteId::Home => "Home",
                        RouteId::Settings => "Settings",
                        RouteId::User => "User",
                    };

                    let params_line: Arc<str> = if params.is_empty() {
                        Arc::from("<no params>")
                    } else {
                        Arc::from(format!("params={params:?}"))
                    };

                    shadcn::card(move |cx| {
                        ui::children![cx;
                            shadcn::card_header(|cx| {
                                ui::children![cx;
                                    shadcn::card_title(title),
                                    shadcn::card_description(format!("matched_path={matched_path}")),
                                ]
                            }),
                            shadcn::card_content(|cx| {
                                ui::children![cx;
                                    ui::text(params_line),
                                    ui::text(format!("depth={depth}")),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full())
                },
                |_cx, snap| {
                    let location = snap.location.to_url();

                    shadcn::card(move |cx| {
                        ui::children![cx;
                            shadcn::card_header(|cx| {
                                ui::children![cx;
                                    shadcn::card_title("Not found"),
                                    shadcn::card_description(format!("location={}", location)),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full())
                },
            );

        let intents_panel = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Prefetch intents"),
                        shadcn::card_description("Populated on link hover and navigation."),
                        shadcn::Button::new("Clear")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .action(act::ClearIntents)
                            .test_id(TEST_ID_BTN_CLEAR_INTENTS),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![cx;
                        ui::v_flex(|cx| {
                            if intents.is_empty() {
                                return vec![cx.text("<none>")];
                            }
                            intents
                                .into_iter()
                                .map(|i| {
                                    cx.text(format!(
                                        "{:?}: {} ({}) extra={:?}",
                                        i.route,
                                        i.location.to_url(),
                                        i.namespace,
                                        i.extra
                                    ))
                                })
                                .collect::<Vec<_>>()
                        })
                        .gap(Space::N1)
                        .test_id(TEST_ID_INTENTS_ROOT)
                    ]
                }),
            ]
        })
        .ui()
        .w_px(Px(320.0));

        let content = ui::v_flex(|cx| {
            ui::children![cx;
                header_row,
                ui::h_flex(|cx| ui::children![cx; nav, outlet, intents_panel])
                    .gap(Space::N3)
                    .items_start(),
            ]
        })
        .gap(Space::N3);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Router basics"),
                        shadcn::card_description(
                            "A tiny routing example: typed route codec, links, outlet rendering, and back/forward.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(980.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-router-basics")
        .window("cookbook-router-basics", (1040.0, 620.0))
        .config_files(false)
        .setup(fret::router::app::install)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<RouterBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
