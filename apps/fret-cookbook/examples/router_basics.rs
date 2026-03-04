use std::sync::Arc;

use fret::prelude::*;
use fret_router::{
    HistoryAdapter as _,
    MemoryHistory, NavigationAction, PathParam, RouteHooks, RouteLocation, RouteNode,
    RoutePrefetchIntent, RouteSearchTable, RouteTree, Router, SearchMap, SearchValidationMode,
};
use fret_router_ui::{RouterOutlet, RouterUiSnapshot, RouterUiStore, router_link_to_with_test_id};
use fret_ui::{CommandAvailability, Invalidation};

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

struct RouterBasicsView {
    store: RouterUiStore<RouteId, MemoryHistory>,
}

impl View for RouterBasicsView {
    fn init(app: &mut App, window: AppWindowId) -> Self {
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

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let window = cx.window;

        let router_model = self.store.router_model().downgrade();
        let snapshot_model = self.store.snapshot_model();
        let snapshot_weak = snapshot_model.downgrade();
        let intents_model = self.store.intents_model();
        let intents_weak = intents_model.downgrade();
        let intents_weak_for_clear = intents_weak.clone();

        let navigate_history_action = {
            let intents_weak = intents_weak.clone();
            let router_model = router_model.clone();
            let snapshot = snapshot_weak.clone();
            move |action: NavigationAction| {
                let router_model = router_model.clone();
                let snapshot = snapshot.clone();
                let intents = intents_weak.clone();

                move |host: &mut dyn fret_ui::action::UiFocusActionHost, _acx| {
                    let Some(router) = router_model.upgrade() else {
                        return true;
                    };
                    let Some(snapshot) = snapshot.upgrade() else {
                        return true;
                    };
                    let Some(intents_model) = intents.upgrade() else {
                        return true;
                    };

                    let result = host.models_mut().update(&router, |router| {
                        let update = router.navigate_with_prefetch_intents(action, None)?;
                        let next_snapshot = RouterUiSnapshot::from_state(router.state());
                        let history = router.history();
                        let can_back = history.can_navigate(NavigationAction::Back);
                        let can_forward = history.can_navigate(NavigationAction::Forward);
                        Ok::<_, fret_router::RouteSearchValidationFailure>((
                            update,
                            next_snapshot,
                            can_back,
                            can_forward,
                        ))
                    });

                    let Ok(result) = result else {
                        return true;
                    };
                    let Ok((update, next_snapshot, can_back, can_forward)) = result else {
                        host.request_redraw(window);
                        return true;
                    };
                    if !update.update.changed() {
                        return true;
                    }

                    let _ = host.models_mut().update(&snapshot, |v| *v = next_snapshot);
                    let _ = host
                        .models_mut()
                        .update(&intents_model, |v| *v = update.intents);
                    host.set_router_command_availability(window, can_back, can_forward);
                    host.request_redraw(window);
                    true
                }
            }
        };

        cx.on_action::<act::RouterBack>(navigate_history_action(NavigationAction::Back));
        cx.on_action::<act::RouterForward>(navigate_history_action(NavigationAction::Forward));
        cx.on_action::<act::ClearIntents>({
            let intents = intents_weak_for_clear;
            move |host, _acx| {
                let Some(intents) = intents.upgrade() else {
                    return true;
                };
                let _ = host.models_mut().update(&intents, |v| v.clear());
                host.request_redraw(window);
                true
            }
        });

        cx.on_action_availability::<act::RouterBack>(|_host, _acx| CommandAvailability::Available);
        cx.on_action_availability::<act::RouterForward>(|_host, _acx| {
            CommandAvailability::Available
        });
        cx.on_action_availability::<act::ClearIntents>(|_host, _acx| {
            CommandAvailability::Available
        });

        let snapshot = cx
            .get_model_cloned(&snapshot_model, Invalidation::Layout)
            .expect("router snapshot should be readable");
        let intents = cx.watch_model(&intents_model).layout().cloned_or_default();

        let location_label: Arc<str> = Arc::from(snapshot.location.to_url());
        let can_back = self.store.can_navigate(cx.app, NavigationAction::Back);
        let can_forward = self.store.can_navigate(cx.app, NavigationAction::Forward);

        let back = shadcn::Button::new("Back")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(!can_back)
            .action(act::RouterBack)
            .into_element(cx)
            .test_id(TEST_ID_BTN_BACK);

        let forward = shadcn::Button::new("Forward")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(!can_forward)
            .action(act::RouterForward)
            .into_element(cx)
            .test_id(TEST_ID_BTN_FORWARD);

        let location = cx.text(location_label).test_id(TEST_ID_LOCATION_LABEL);

        let header_row = ui::h_flex(cx, |_cx| [back, forward, location])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let home_label = cx.text("Home");
        let settings_label = cx.text("Settings");
        let user_label = cx.text("User 42");
        let missing_label = cx.text("Missing");

        let home_link = router_link_to_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &RouteId::Home,
            &[],
            SearchMap::default(),
            None,
            TEST_ID_LINK_HOME,
            [home_label],
        )
        .unwrap_or_else(|err| cx.text(format!("link error: {err}")));

        let settings_link = router_link_to_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &RouteId::Settings,
            &[],
            SearchMap::default(),
            None,
            TEST_ID_LINK_SETTINGS,
            [settings_label],
        )
        .unwrap_or_else(|err| cx.text(format!("link error: {err}")));

        let user_link = router_link_to_with_test_id(
            cx,
            &self.store,
            NavigationAction::Push,
            &RouteId::User,
            &[PathParam {
                name: "id".to_string(),
                value: "42".to_string(),
            }],
            SearchMap::default(),
            None,
            TEST_ID_LINK_USER_42,
            [user_label],
        )
        .unwrap_or_else(|err| cx.text(format!("link error: {err}")));

        let missing_link = {
            let link = self
                .store
                .link_to_location(NavigationAction::Push, RouteLocation::parse("/missing"));
            fret_router_ui::router_link_with_test_id(
                cx,
                &self.store,
                link,
                TEST_ID_LINK_MISSING,
                [missing_label],
            )
        };

        let nav = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Nav").into_element(cx),
                shadcn::CardDescription::new("Router links (hover prefetch + click navigate).")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([ui::v_flex(cx, |_cx| {
                [home_link, settings_link, user_link, missing_link]
            })
            .gap(Space::N2)
            .into_element(cx)])
            .into_element(cx),
        ])
        .ui()
        .w_px(Px(200.0))
        .into_element(cx);

        let outlet = RouterOutlet::new(snapshot_model.clone())
            .test_id(TEST_ID_OUTLET)
            .into_element_by_leaf(
                cx,
                |cx, route, snap| {
                    let leaf = snap.leaf_match().expect("leaf match should exist");
                    let matched_path = leaf.matched_path.clone();
                    let params = leaf.params.clone();

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

                    shadcn::Card::new([
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new(title).into_element(cx),
                            shadcn::CardDescription::new(format!(
                                "matched_path={matched_path}"
                            ))
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new([
                            cx.text(params_line),
                            cx.text(format!("depth={}", snap.match_depth())),
                        ])
                        .into_element(cx),
                    ])
                    .ui()
                    .w_full()
                    .into_element(cx)
                },
                |cx, snap| {
                    shadcn::Card::new([
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new("Not found").into_element(cx),
                            shadcn::CardDescription::new(format!(
                                "location={}",
                                snap.location.to_url()
                            ))
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .ui()
                    .w_full()
                    .into_element(cx)
                },
            );

        let intents_panel = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Prefetch intents").into_element(cx),
                shadcn::CardDescription::new("Populated on link hover and navigation.")
                    .into_element(cx),
                shadcn::Button::new("Clear")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .action(act::ClearIntents)
                    .into_element(cx)
                    .test_id(TEST_ID_BTN_CLEAR_INTENTS),
            ])
            .into_element(cx),
            shadcn::CardContent::new([ui::v_flex(cx, |cx| {
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
            .into_element(cx)
            .test_id(TEST_ID_INTENTS_ROOT)])
            .into_element(cx),
        ])
        .ui()
        .w_px(Px(320.0))
        .into_element(cx);

        let content = ui::v_flex(cx, |cx| {
            [
                header_row,
                ui::h_flex(cx, |_cx| [nav, outlet, intents_panel])
                    .gap(Space::N3)
                    .items_start()
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .into_element(cx);

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Router basics").into_element(cx),
                shadcn::CardDescription::new(
                    "A tiny routing example: links, outlet rendering, and back/forward.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(980.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn install_router_commands(app: &mut App) {
    fret_router_ui::register_router_commands(app.commands_mut());
    fret_app::install_command_default_keybindings_into_keymap(app);
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-router-basics")
        .window("cookbook-router-basics", (1040.0, 620.0))
        .config_files(false)
        .install_app(install_router_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<RouterBasicsView>()
        .map_err(anyhow::Error::from)
}
