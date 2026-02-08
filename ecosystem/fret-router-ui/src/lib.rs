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
    HistoryAdapter, NavigationAction, RouteLocation, RouteMatchSnapshot, Router, RouterEvent,
    RouterTransition, RouterUpdate, RouterUpdateWithPrefetchIntents,
};
use fret_runtime::Model;
use fret_runtime::{CommandId, Effect};
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
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
    H: HistoryAdapter,
{
    window: AppWindowId,
    router: Router<R, H>,
    snapshot: Model<RouterUiSnapshot<R>>,
}

impl<R, H> RouterUiStore<R, H>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter,
{
    pub fn new(app: &mut App, window: AppWindowId, router: Router<R, H>) -> Self {
        let snapshot = RouterUiSnapshot::from_state(router.state());
        let snapshot = app.models_mut().insert(snapshot);
        Self {
            window,
            router,
            snapshot,
        }
    }

    pub fn snapshot_model(&self) -> Model<RouterUiSnapshot<R>> {
        self.snapshot.clone()
    }

    pub fn router(&self) -> &Router<R, H> {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut Router<R, H> {
        &mut self.router
    }

    pub fn state(&self) -> &fret_router::RouterState<R> {
        self.router.state()
    }

    pub fn history(&self) -> &H {
        self.router.history()
    }

    pub fn history_mut(&mut self) -> &mut H {
        self.router.history_mut()
    }

    pub fn take_events(&mut self) -> Vec<RouterEvent<R>> {
        self.router.take_events()
    }

    pub fn apply_update(&mut self, app: &mut App, update: &RouterUpdate) {
        if !update.changed() {
            return;
        }

        let next = RouterUiSnapshot::from_state(self.router.state());
        let _ = app.models_mut().update(&self.snapshot, |v| *v = next);
        app.request_redraw(self.window);
    }

    pub fn navigate(
        &mut self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdate, fret_router::RouteSearchValidationFailure> {
        let update = self.router.navigate(action, target)?;
        self.apply_update(app, &update);
        Ok(update)
    }

    pub fn navigate_with_prefetch_intents(
        &mut self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, fret_router::RouteSearchValidationFailure> {
        let update = self.router.navigate_with_prefetch_intents(action, target)?;
        self.apply_update(app, &update.update);
        Ok(update)
    }

    pub fn sync_with_prefetch_intents(
        &mut self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, fret_router::RouteSearchValidationFailure> {
        let update = self.router.sync_with_prefetch_intents()?;
        self.apply_update(app, &update.update);
        Ok(update)
    }

    pub fn init_with_prefetch_intents(
        &mut self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, fret_router::RouteSearchValidationFailure> {
        let update = self.router.init_with_prefetch_intents()?;
        self.apply_update(app, &update.update);
        Ok(update)
    }

    pub fn link_to(
        &self,
        action: NavigationAction,
        route: &R,
        params: &[fret_router::PathParam],
        search: fret_router::SearchMap,
        fragment: Option<String>,
    ) -> Result<RouterLink, fret_router::RouterBuildLocationError> {
        let to = self
            .router
            .build_location(route, params, search, fragment)?;
        let href: Arc<str> = Arc::from(to.to_url());
        Ok(RouterLink { action, href, to })
    }

    pub fn link_to_location(&self, action: NavigationAction, mut to: RouteLocation) -> RouterLink {
        to.canonicalize();
        let href: Arc<str> = Arc::from(to.to_url());
        RouterLink { action, href, to }
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
        let mut store = RouterUiStore::new(&mut app, window, router);

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
        let mut store = RouterUiStore::new(&mut app, window, router);

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
