use std::hash::Hash;
use std::sync::Arc;

use crate::{
    MemoryHistory, NavigationAction, RouteLocation, RouteMatchResultWithSearch, RouteSearchTable,
    RouteSearchValidationFailure, RouteTree, SearchValidationMode,
};

pub trait HistoryAdapter {
    fn current(&self) -> &RouteLocation;
    fn refresh(&mut self) {}
    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool;
}

impl HistoryAdapter for MemoryHistory {
    fn current(&self) -> &RouteLocation {
        MemoryHistory::current(self)
    }

    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        MemoryHistory::navigate(self, action, target)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RouterTransitionCause {
    Navigate { action: NavigationAction },
    Redirect { action: NavigationAction },
    Sync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RouterBlockReason {
    pub message: String,
}

impl RouterBlockReason {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RouterGuardContext<'a, R> {
    pub cause: RouterTransitionCause,
    pub from: &'a RouteLocation,
    pub to: &'a RouteLocation,
    pub from_state: &'a RouterState<R>,
    pub next_state: &'a RouterState<R>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RouterGuardDecision {
    Allow,
    Block {
        reason: RouterBlockReason,
    },
    Redirect {
        action: NavigationAction,
        to: RouteLocation,
    },
}

impl RouterGuardDecision {
    pub fn redirect_replace(to: RouteLocation) -> Self {
        Self::Redirect {
            action: NavigationAction::Replace,
            to,
        }
    }
}

pub type RouterGuardFn<R> =
    Arc<dyn for<'a> Fn(&RouterGuardContext<'a, R>) -> RouterGuardDecision + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterTransition {
    pub cause: RouterTransitionCause,
    pub from: RouteLocation,
    pub to: RouteLocation,
    pub redirect_chain: Vec<RouteLocation>,
    pub blocked_by: Option<RouterBlockReason>,
}

impl RouterTransition {
    pub fn navigate(action: NavigationAction, from: RouteLocation, to: RouteLocation) -> Self {
        Self {
            cause: RouterTransitionCause::Navigate { action },
            from,
            to,
            redirect_chain: Vec::new(),
            blocked_by: None,
        }
    }

    pub fn redirect(
        action: NavigationAction,
        from: RouteLocation,
        attempted: RouteLocation,
        to: RouteLocation,
    ) -> Self {
        Self {
            cause: RouterTransitionCause::Redirect { action },
            from,
            to,
            redirect_chain: vec![attempted],
            blocked_by: None,
        }
    }

    pub fn sync(from: RouteLocation, to: RouteLocation) -> Self {
        Self {
            cause: RouterTransitionCause::Sync,
            from,
            to,
            redirect_chain: Vec::new(),
            blocked_by: None,
        }
    }

    pub fn action(&self) -> Option<NavigationAction> {
        match self.cause {
            RouterTransitionCause::Navigate { action } => Some(action),
            RouterTransitionCause::Redirect { action } => Some(action),
            RouterTransitionCause::Sync => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterUpdate {
    NoChange,
    Changed(RouterTransition),
    Blocked(RouterTransition),
}

impl RouterUpdate {
    pub fn changed(&self) -> bool {
        match self {
            Self::NoChange => false,
            Self::Changed(_) => true,
            Self::Blocked(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RouterEvent<R> {
    Transitioned {
        transition: RouterTransition,
        state: RouterState<R>,
    },
    Blocked {
        transition: RouterTransition,
        state: RouterState<R>,
    },
}

#[derive(Debug, Clone)]
pub struct RouterState<R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatchSnapshot<R>>,
    pub is_not_found: bool,
    pub last_transition: Option<RouterTransition>,
}

#[derive(Debug, Clone)]
pub struct RouteMatchSnapshot<R> {
    pub route: R,
    pub matched_path: String,
    pub params: Vec<crate::PathParam>,
    pub search: crate::SearchMap,
    pub search_error: Option<crate::SearchValidationError>,
}

impl<R> RouterState<R> {
    fn from_match_result(result: RouteMatchResultWithSearch<'_, R>) -> Self
    where
        R: Clone,
    {
        Self {
            location: result.location,
            matches: result
                .matches
                .into_iter()
                .map(|entry| RouteMatchSnapshot {
                    route: entry.route.clone(),
                    matched_path: entry.matched_path,
                    params: entry.params,
                    search: entry.search,
                    search_error: entry.search_error,
                })
                .collect(),
            is_not_found: result.is_not_found,
            last_transition: None,
        }
    }
}

pub struct Router<R, H> {
    tree: Arc<RouteTree<R>>,
    search_table: Arc<RouteSearchTable<R>>,
    search_mode: SearchValidationMode,
    history: H,
    state: RouterState<R>,
    events: Vec<RouterEvent<R>>,
    guard: Option<RouterGuardFn<R>>,
}

impl<R, H> Router<R, H>
where
    R: Clone + Hash + Eq,
    H: HistoryAdapter,
{
    pub fn new(
        tree: Arc<RouteTree<R>>,
        search_table: Arc<RouteSearchTable<R>>,
        search_mode: SearchValidationMode,
        mut history: H,
    ) -> Result<Self, RouteSearchValidationFailure> {
        history.refresh();
        let initial_location = history.current().clone();
        let state = RouterState::from_match_result(tree.match_routes_with_search(
            &initial_location,
            search_table.as_ref(),
            search_mode,
        )?);

        Ok(Self {
            tree,
            search_table,
            search_mode,
            history,
            state,
            events: Vec::new(),
            guard: None,
        })
    }

    pub fn state(&self) -> &RouterState<R> {
        &self.state
    }

    pub fn history(&self) -> &H {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut H {
        &mut self.history
    }

    pub fn set_guard(&mut self, guard: Option<RouterGuardFn<R>>) {
        self.guard = guard;
    }

    pub fn take_events(&mut self) -> Vec<RouterEvent<R>> {
        std::mem::take(&mut self.events)
    }

    pub fn sync(&mut self) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let next_location = self.history.current().clone();
        if next_location == self.state.location {
            return Ok(RouterUpdate::NoChange);
        }

        let next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
            &next_location,
            self.search_table.as_ref(),
            self.search_mode,
        )?);

        let transition =
            RouterTransition::sync(self.state.location.clone(), next_state.location.clone());
        self.state = next_state;
        self.state.last_transition = Some(transition.clone());
        self.events.push(RouterEvent::Transitioned {
            transition: transition.clone(),
            state: self.state.clone(),
        });

        Ok(RouterUpdate::Changed(transition))
    }

    pub fn navigate(
        &mut self,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let from = self.history.current().clone();

        let did_navigate = match action {
            NavigationAction::Push | NavigationAction::Replace => {
                let Some(target) = target else {
                    return Ok(RouterUpdate::NoChange);
                };

                let proposed_location = target.canonicalized();

                if let Some(guard) = self.guard.as_ref() {
                    let next_state =
                        RouterState::from_match_result(self.tree.match_routes_with_search(
                            &proposed_location,
                            self.search_table.as_ref(),
                            self.search_mode,
                        )?);
                    let ctx = RouterGuardContext {
                        cause: RouterTransitionCause::Navigate { action },
                        from: &from,
                        to: &proposed_location,
                        from_state: &self.state,
                        next_state: &next_state,
                    };
                    match guard(&ctx) {
                        RouterGuardDecision::Allow => {}
                        RouterGuardDecision::Block { reason } => {
                            let mut transition =
                                RouterTransition::navigate(action, from.clone(), proposed_location);
                            transition.blocked_by = Some(reason);

                            self.state.last_transition = Some(transition.clone());
                            self.events.push(RouterEvent::Blocked {
                                transition: transition.clone(),
                                state: self.state.clone(),
                            });

                            return Ok(RouterUpdate::Blocked(transition));
                        }
                        RouterGuardDecision::Redirect {
                            action: redirect_action,
                            to: redirect_to,
                        } => {
                            let redirect_to = redirect_to.canonicalized();
                            let changed = self
                                .history
                                .navigate(redirect_action, Some(redirect_to.clone()));
                            if !changed {
                                return Ok(RouterUpdate::NoChange);
                            }

                            self.history.refresh();
                            let to = self.history.current().clone();
                            let mut next_state = RouterState::from_match_result(
                                self.tree.match_routes_with_search(
                                    &to,
                                    self.search_table.as_ref(),
                                    self.search_mode,
                                )?,
                            );
                            let transition = RouterTransition::redirect(
                                redirect_action,
                                from,
                                proposed_location,
                                to,
                            );
                            next_state.last_transition = Some(transition.clone());

                            self.state = next_state;
                            self.events.push(RouterEvent::Transitioned {
                                transition: transition.clone(),
                                state: self.state.clone(),
                            });

                            return Ok(RouterUpdate::Changed(transition));
                        }
                    }
                }

                self.history
                    .navigate(action, Some(proposed_location.clone()))
            }
            NavigationAction::Back | NavigationAction::Forward => {
                self.history.navigate(action, None)
            }
        };

        if !did_navigate {
            return Ok(RouterUpdate::NoChange);
        }

        self.history.refresh();
        let to = self.history.current().clone();
        let mut next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
            &to,
            self.search_table.as_ref(),
            self.search_mode,
        )?);
        let transition = RouterTransition::navigate(action, from, to);
        next_state.last_transition = Some(transition.clone());

        self.state = next_state;
        self.events.push(RouterEvent::Transitioned {
            transition: transition.clone(),
            state: self.state.clone(),
        });

        Ok(RouterUpdate::Changed(transition))
    }
}

#[cfg(test)]
mod tests {
    use super::{Router, RouterGuardDecision};
    use crate::{
        MemoryHistory, NavigationAction, RouteLocation, RouteNode, RouteSearchTable, RouteTree,
        SearchMap, SearchValidationError, SearchValidationMode,
    };
    use std::sync::Arc;

    #[test]
    fn router_navigate_updates_state_and_transition() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Gallery,
        }

        fn validate_root(
            _location: &RouteLocation,
            search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            Ok(search.clone().with("lang", Some("en".to_string())))
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Gallery, "gallery").unwrap()]),
        ));

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        let search_table = Arc::new(search_table);

        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Lenient, history)
            .expect("router should build");

        assert_eq!(router.state().location.to_url(), "/");
        assert!(
            router
                .navigate(
                    NavigationAction::Push,
                    Some(RouteLocation::parse("/gallery?lang=zh"))
                )
                .expect("navigate should succeed")
                .changed()
        );

        assert_eq!(router.state().location.to_url(), "/gallery?lang=zh");
        assert_eq!(router.state().matches.len(), 2);
        assert_eq!(router.state().matches[0].search.first("lang"), Some("en"));

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert_eq!(transition.action(), Some(NavigationAction::Push));
        assert_eq!(transition.from.to_url(), "/");
        assert_eq!(transition.to.to_url(), "/gallery?lang=zh");

        let events = router.take_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn router_sync_records_transition_and_event_when_location_changes() {
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
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        assert!(router.history_mut().push(RouteLocation::parse("/settings")));
        let update = router.sync().expect("sync should succeed");
        assert!(update.changed());

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Sync
        ));

        let events = router.take_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn router_guard_can_block_navigation_attempt() {
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
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.set_guard(Some(Arc::new(|ctx| {
            if ctx.to.path == "/settings" {
                RouterGuardDecision::Block {
                    reason: super::RouterBlockReason::new("blocked settings"),
                }
            } else {
                RouterGuardDecision::Allow
            }
        })));

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(matches!(update, super::RouterUpdate::Blocked(_)));
        assert_eq!(router.state().location.to_url(), "/");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert_eq!(transition.to.to_url(), "/settings");
        assert!(transition.blocked_by.is_some());

        let events = router.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::RouterEvent::Blocked { .. }));
    }

    #[test]
    fn router_guard_can_redirect_navigation_attempt() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
            Login,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![
                    RouteNode::new(RouteId::Settings, "settings").unwrap(),
                    RouteNode::new(RouteId::Login, "login").unwrap(),
                ]),
        ));

        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.set_guard(Some(Arc::new(|ctx| {
            if ctx.to.path == "/settings" {
                RouterGuardDecision::redirect_replace(RouteLocation::parse("/login"))
            } else {
                RouterGuardDecision::Allow
            }
        })));

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(matches!(update, super::RouterUpdate::Changed(_)));
        assert_eq!(router.state().location.to_url(), "/login");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Redirect { .. }
        ));
        assert_eq!(transition.redirect_chain.len(), 1);
        assert_eq!(transition.redirect_chain[0].to_url(), "/settings");

        let events = router.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::RouterEvent::Transitioned { .. }));
    }

    #[test]
    fn memory_history_adapter_is_usable_via_trait() {
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.navigate(NavigationAction::Push, Some(RouteLocation::parse("/next"))));
        assert_eq!(history.current().to_url(), "/next");
    }
}
