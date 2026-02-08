//! UI adoption helpers for `fret-router` (desktop-first).
//!
//! This crate intentionally provides a thin layer:
//! - a window-scoped `RouterUiStore` that owns a router and a snapshot `Model`
//! - typed navigation helpers that update the snapshot model
//!
//! Policy-heavy behavior remains in apps and higher-level ecosystem crates.

use std::hash::Hash;
use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_router::{
    HistoryAdapter, NavigationAction, PathParam, RouteLocation, RouteMatchSnapshot,
    RoutePrefetchIntent, RouteSearchValidationFailure, Router, RouterBuildLocationError,
    RouterEvent, RouterTransition, RouterUpdate, RouterUpdateWithPrefetchIntents, SearchMap,
};
use fret_runtime::{CommandId, Effect};
use fret_runtime::{Model, WeakModel};
use fret_ui::action::{OnActivate, OnHoverChange};
use fret_ui::element::AnyElement;
use fret_ui::element::PressableProps;
use fret_ui::{ElementContext, Invalidation};

#[derive(Debug, Clone)]
pub struct RouterUiSnapshot<R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatchSnapshot<R>>,
    pub is_not_found: bool,
    pub last_transition: Option<RouterTransition>,
}

impl<R> RouterUiSnapshot<R>
where
    R: Clone,
{
    pub fn from_state(state: &fret_router::RouterState<R>) -> Self {
        Self {
            location: state.location.clone(),
            matches: state.matches.clone(),
            is_not_found: state.is_not_found,
            last_transition: state.last_transition.clone(),
        }
    }

    pub fn leaf_match(&self) -> Option<&RouteMatchSnapshot<R>> {
        self.matches.last()
    }

    pub fn leaf_route(&self) -> Option<&R> {
        self.leaf_match().map(|m| &m.route)
    }
}

pub struct RouterUiStore<R, H>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    window: AppWindowId,
    router: Model<Router<R, H>>,
    snapshot: Model<RouterUiSnapshot<R>>,
    intents: Model<Vec<RoutePrefetchIntent<R>>>,
}

impl<R, H> RouterUiStore<R, H>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    pub fn new(app: &mut App, window: AppWindowId, router: Router<R, H>) -> Self {
        let snapshot = RouterUiSnapshot::from_state(router.state());
        let router = app.models_mut().insert(router);
        let snapshot = app.models_mut().insert(snapshot);
        let intents = app.models_mut().insert(Vec::new());
        Self {
            window,
            router,
            snapshot,
            intents,
        }
    }

    pub fn snapshot_model(&self) -> Model<RouterUiSnapshot<R>> {
        self.snapshot.clone()
    }

    pub fn router_model(&self) -> Model<Router<R, H>> {
        self.router.clone()
    }

    pub fn intents_model(&self) -> Model<Vec<RoutePrefetchIntent<R>>> {
        self.intents.clone()
    }

    pub fn build_location(
        &self,
        app: &App,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<RouteLocation, RouterBuildLocationError> {
        app.models()
            .read(&self.router, |router| {
                router.build_location(route, params, search, fragment)
            })
            .expect("router model should be readable")
    }

    pub fn href_to(
        &self,
        app: &App,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<String, RouterBuildLocationError> {
        app.models()
            .read(&self.router, |router| {
                router.href_to(route, params, search, fragment)
            })
            .expect("router model should be readable")
    }

    pub fn link_to(
        &self,
        app: &App,
        action: NavigationAction,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<RouterLink, RouterBuildLocationError> {
        let to = self.build_location(app, route, params, search, fragment)?;
        Ok(self.link_to_location(action, to))
    }

    pub fn link_to_location(&self, action: NavigationAction, mut to: RouteLocation) -> RouterLink {
        to.canonicalize();
        let href: Arc<str> = Arc::from(to.to_url());
        RouterLink { action, href, to }
    }

    pub fn take_events(&self, app: &mut App) -> Vec<RouterEvent<R>> {
        app.models_mut()
            .update(&self.router, |router| router.take_events())
            .expect("router model should be updatable")
    }

    fn apply_update(
        &self,
        app: &mut App,
        update: &RouterUpdate,
        intents: &[RoutePrefetchIntent<R>],
    ) {
        if !update.changed() {
            return;
        }

        let next = app
            .models()
            .read(&self.router, |router| {
                RouterUiSnapshot::from_state(router.state())
            })
            .expect("router model should be readable");
        let _ = app.models_mut().update(&self.snapshot, |v| *v = next);
        let intents = intents.to_vec();
        let _ = app.models_mut().update(&self.intents, |v| *v = intents);
        app.request_redraw(self.window);
    }

    pub fn navigate(
        &self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.navigate(action, target))
            .expect("router model should be updatable")?;
        self.apply_update(app, &update, &[]);
        Ok(update)
    }

    pub fn navigate_with_prefetch_intents(
        &self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| {
                router.navigate_with_prefetch_intents(action, target)
            })
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn sync_with_prefetch_intents(
        &self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.sync_with_prefetch_intents())
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn init_with_prefetch_intents(
        &self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.init_with_prefetch_intents())
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn navigate_link_on_activate(&self, link: RouterLink) -> OnActivate {
        let window = self.window;
        let router: WeakModel<Router<R, H>> = self.router.downgrade();
        let snapshot: WeakModel<RouterUiSnapshot<R>> = self.snapshot.downgrade();
        let intents: WeakModel<Vec<RoutePrefetchIntent<R>>> = self.intents.downgrade();
        Arc::new(move |host, _cx, _reason| {
            let Some(router) = router.upgrade() else {
                return;
            };
            let Some(snapshot) = snapshot.upgrade() else {
                return;
            };
            let Some(intents_model) = intents.upgrade() else {
                return;
            };

            let action = link.action;
            let to = link.to.clone();

            let result = host.models_mut().update(&router, |router| {
                let update = router.navigate_with_prefetch_intents(action, Some(to))?;
                let snapshot = RouterUiSnapshot::from_state(router.state());
                Ok::<_, RouteSearchValidationFailure>((update, snapshot))
            });

            let Ok(result) = result else {
                return;
            };
            let Ok((update, next_snapshot)) = result else {
                host.request_redraw(window);
                return;
            };

            if !update.update.changed() {
                return;
            }

            let _ = host.models_mut().update(&snapshot, |v| *v = next_snapshot);
            let _ = host
                .models_mut()
                .update(&intents_model, |v| *v = update.intents.clone());
            host.request_redraw(window);
        })
    }

    pub fn prefetch_link_on_hover_change(&self, link: RouterLink) -> OnHoverChange {
        let window = self.window;
        let router: WeakModel<Router<R, H>> = self.router.downgrade();
        let intents: WeakModel<Vec<RoutePrefetchIntent<R>>> = self.intents.downgrade();
        Arc::new(move |host, _cx, hovered| {
            if !hovered {
                return;
            }

            let Some(router) = router.upgrade() else {
                return;
            };
            let Some(intents_model) = intents.upgrade() else {
                return;
            };

            let to = link.to.clone();
            let result = host
                .models_mut()
                .update(&router, |router| router.prefetch_intents_for_location(&to));

            let Ok(result) = result else {
                return;
            };
            let Ok(intents) = result else {
                host.request_redraw(window);
                return;
            };

            let _ = host.models_mut().update(&intents_model, |v| *v = intents);
            host.request_redraw(window);
        })
    }
}

#[derive(Debug, Clone)]
pub struct RouterLink {
    pub action: NavigationAction,
    pub href: Arc<str>,
    pub to: RouteLocation,
}

impl RouterLink {
    pub fn copy_href_on_activate(href: impl Into<Arc<str>>) -> OnActivate {
        let href: Arc<str> = href.into();
        Arc::new(move |host, _cx, _reason| {
            host.push_effect(Effect::ClipboardSetText {
                text: href.to_string(),
            });
        })
    }

    pub fn dispatch_command_on_activate(
        window: Option<AppWindowId>,
        command: impl Into<CommandId>,
    ) -> OnActivate {
        let command = command.into();
        Arc::new(move |host, _cx, _reason| {
            host.dispatch_command(window, command.clone());
        })
    }
}

pub fn router_outlet<R>(
    cx: &mut ElementContext<'_, App>,
    snapshot: &Model<RouterUiSnapshot<R>>,
    render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
) -> AnyElement
where
    R: Clone + 'static,
{
    let snap = cx
        .get_model_cloned(snapshot, Invalidation::Layout)
        .expect("router snapshot model should be readable");
    render(cx, &snap)
}

pub fn router_link<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    let on_activate = store.navigate_link_on_activate(link.clone());
    let on_hover = store.prefetch_link_on_hover_change(link);
    let children: Vec<AnyElement> = children.into_iter().collect();

    cx.pressable_with_id_props(|cx, _state, _id| {
        cx.pressable_on_activate(on_activate);
        cx.pressable_on_hover_change(on_hover);
        (PressableProps::default(), children)
    })
}

#[cfg(test)]
mod tests {
    use super::RouterUiStore;
    use fret_app::App;
    use fret_router::{
        MemoryHistory, RouteHooks, RouteLocation, RouteNode, RoutePrefetchIntent, RouteSearchTable,
        RouteTree, Router,
    };
    use std::sync::Arc;

    #[test]
    fn router_ui_store_updates_snapshot_on_navigation() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);

        let update = store
            .navigate(
                &mut app,
                fret_router::NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(update.changed());

        let snapshot = app
            .models()
            .get_cloned(&store.snapshot_model())
            .expect("snapshot should be readable");
        assert_eq!(snapshot.location.to_url(), "/settings");
        assert_eq!(snapshot.matches.len(), 2);
    }

    #[test]
    fn router_ui_store_init_returns_prefetch_intents() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/settings"));
        let mut router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    vec![RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "fret-router-ui.tests.init_prefetch",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);

        let update = store
            .init_with_prefetch_intents(&mut app)
            .expect("init should succeed");
        assert_eq!(update.intents.len(), 1);

        let snapshot = app
            .models()
            .get_cloned(&store.snapshot_model())
            .expect("snapshot should be readable");
        assert_eq!(snapshot.location.to_url(), "/settings");
        assert_eq!(snapshot.matches.len(), 2);
    }
}
