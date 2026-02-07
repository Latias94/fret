use std::hash::Hash;
use std::sync::Arc;

use crate::{
    MemoryHistory, NavigationAction, RouteLocation, RouteMatchResultWithSearch, RouteSearchTable,
    RouteSearchValidationFailure, RouteTree, SearchValidationMode,
};

const DEFAULT_MAX_REDIRECT_HOPS: usize = 4;

const REDIRECT_LOOP_REASON: &str = "redirect loop detected";
const REDIRECT_HOP_LIMIT_REASON: &str = "redirect hop limit exceeded";

pub trait HistoryAdapter {
    fn current(&self) -> &RouteLocation;
    fn refresh(&mut self) {}
    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool;
    fn peek(&mut self, _action: NavigationAction) -> Option<RouteLocation> {
        None
    }
}

impl HistoryAdapter for MemoryHistory {
    fn current(&self) -> &RouteLocation {
        MemoryHistory::current(self)
    }

    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        MemoryHistory::navigate(self, action, target)
    }

    fn peek(&mut self, action: NavigationAction) -> Option<RouteLocation> {
        match action {
            NavigationAction::Back => self
                .can_back()
                .then(|| self.entries()[self.index() - 1].clone()),
            NavigationAction::Forward => self
                .can_forward()
                .then(|| self.entries()[self.index() + 1].clone()),
            NavigationAction::Push | NavigationAction::Replace => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
enum RouterGuardResolution {
    Allow {
        cause: RouterTransitionCause,
        action: NavigationAction,
        to: RouteLocation,
        redirect_chain: Vec<RouteLocation>,
    },
    Block {
        cause: RouterTransitionCause,
        action: NavigationAction,
        attempted: RouteLocation,
        redirect_chain: Vec<RouteLocation>,
        reason: RouterBlockReason,
    },
}

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
        Self::redirect_chain(action, from, vec![attempted], to)
    }

    pub fn redirect_chain(
        action: NavigationAction,
        from: RouteLocation,
        redirect_chain: Vec<RouteLocation>,
        to: RouteLocation,
    ) -> Self {
        Self {
            cause: RouterTransitionCause::Redirect { action },
            from,
            to,
            redirect_chain,
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
    max_redirect_hops: usize,
}

impl<R, H> Router<R, H>
where
    R: Clone + Hash + Eq,
    H: HistoryAdapter,
{
    fn resolve_guard_chain(
        &self,
        original_action: NavigationAction,
        from: &RouteLocation,
        initial_target: RouteLocation,
    ) -> Result<RouterGuardResolution, RouteSearchValidationFailure> {
        let Some(guard) = self.guard.as_ref() else {
            return Ok(RouterGuardResolution::Allow {
                cause: RouterTransitionCause::Navigate {
                    action: original_action,
                },
                action: original_action,
                to: initial_target,
                redirect_chain: Vec::new(),
            });
        };

        let mut redirect_chain = Vec::<RouteLocation>::new();
        let mut seen = std::collections::HashSet::<String>::new();
        let mut current_action = original_action;
        let mut current_cause = RouterTransitionCause::Navigate {
            action: original_action,
        };
        let mut current_target = initial_target;

        loop {
            let canonical_key = current_target.canonicalized().to_url();
            if !seen.insert(canonical_key) {
                let mut chain = redirect_chain.clone();
                chain.push(current_target.clone());
                return Ok(RouterGuardResolution::Block {
                    cause: current_cause,
                    action: current_action,
                    attempted: current_target,
                    redirect_chain: chain,
                    reason: RouterBlockReason::new(REDIRECT_LOOP_REASON),
                });
            }

            let next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
                &current_target,
                self.search_table.as_ref(),
                self.search_mode,
            )?);
            let ctx = RouterGuardContext {
                cause: current_cause,
                from,
                to: &current_target,
                from_state: &self.state,
                next_state: &next_state,
            };

            match guard(&ctx) {
                RouterGuardDecision::Allow => {
                    return Ok(RouterGuardResolution::Allow {
                        cause: current_cause,
                        action: current_action,
                        to: current_target,
                        redirect_chain,
                    });
                }
                RouterGuardDecision::Block { reason } => {
                    return Ok(RouterGuardResolution::Block {
                        cause: current_cause,
                        action: current_action,
                        attempted: current_target,
                        redirect_chain,
                        reason,
                    });
                }
                RouterGuardDecision::Redirect {
                    action: redirect_action,
                    to: redirect_to,
                } => {
                    if redirect_chain.len() >= self.max_redirect_hops {
                        let mut chain = redirect_chain.clone();
                        chain.push(current_target.clone());
                        return Ok(RouterGuardResolution::Block {
                            cause: current_cause,
                            action: current_action,
                            attempted: current_target,
                            redirect_chain: chain,
                            reason: RouterBlockReason::new(REDIRECT_HOP_LIMIT_REASON),
                        });
                    }

                    redirect_chain.push(current_target);
                    current_action = normalize_redirect_action(redirect_action);
                    current_cause = RouterTransitionCause::Redirect {
                        action: current_action,
                    };
                    current_target = redirect_to.canonicalized();
                }
            }
        }
    }

    fn commit_changed_transition(
        &mut self,
        cause: RouterTransitionCause,
        from: RouteLocation,
        redirect_chain: Vec<RouteLocation>,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let to = self.history.current().clone();

        let mut next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
            &to,
            self.search_table.as_ref(),
            self.search_mode,
        )?);

        let transition = RouterTransition {
            cause,
            from,
            to,
            redirect_chain,
            blocked_by: None,
        };
        next_state.last_transition = Some(transition.clone());

        self.state = next_state;
        self.events.push(RouterEvent::Transitioned {
            transition: transition.clone(),
            state: self.state.clone(),
        });

        Ok(RouterUpdate::Changed(transition))
    }

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
            max_redirect_hops: DEFAULT_MAX_REDIRECT_HOPS,
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

    pub fn max_redirect_hops(&self) -> usize {
        self.max_redirect_hops
    }

    pub fn set_max_redirect_hops(&mut self, max_redirect_hops: usize) {
        self.max_redirect_hops = max_redirect_hops.max(1);
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

        match action {
            NavigationAction::Push | NavigationAction::Replace => {
                let Some(target) = target else {
                    return Ok(RouterUpdate::NoChange);
                };

                let proposed_location = target.canonicalized();
                match self.resolve_guard_chain(action, &from, proposed_location)? {
                    RouterGuardResolution::Allow {
                        cause,
                        action: nav_action,
                        to,
                        redirect_chain,
                    } => {
                        if !self.history.navigate(nav_action, Some(to.clone())) {
                            return Ok(RouterUpdate::NoChange);
                        }
                        return self.commit_changed_transition(cause, from, redirect_chain);
                    }
                    RouterGuardResolution::Block {
                        cause,
                        action: _nav_action,
                        attempted,
                        redirect_chain,
                        reason,
                    } => {
                        let transition = RouterTransition {
                            cause,
                            from,
                            to: attempted,
                            redirect_chain,
                            blocked_by: Some(reason),
                        };

                        self.state.last_transition = Some(transition.clone());
                        self.events.push(RouterEvent::Blocked {
                            transition: transition.clone(),
                            state: self.state.clone(),
                        });

                        return Ok(RouterUpdate::Blocked(transition));
                    }
                }
            }
            NavigationAction::Back | NavigationAction::Forward => {
                if self.guard.is_some() {
                    if let Some(attempted) = self.history.peek(action) {
                        let attempted = attempted.canonicalized();
                        match self.resolve_guard_chain(action, &from, attempted)? {
                            RouterGuardResolution::Allow {
                                cause,
                                action: nav_action,
                                to,
                                redirect_chain,
                            } => {
                                if redirect_chain.is_empty()
                                    && matches!(cause, RouterTransitionCause::Navigate { action: a } if a == action)
                                {
                                    if !self.history.navigate(action, None) {
                                        return Ok(RouterUpdate::NoChange);
                                    }
                                } else if !self.history.navigate(nav_action, Some(to.clone())) {
                                    return Ok(RouterUpdate::NoChange);
                                }

                                return self.commit_changed_transition(cause, from, redirect_chain);
                            }
                            RouterGuardResolution::Block {
                                cause,
                                action: _nav_action,
                                attempted,
                                redirect_chain,
                                reason,
                            } => {
                                let transition = RouterTransition {
                                    cause,
                                    from,
                                    to: attempted,
                                    redirect_chain,
                                    blocked_by: Some(reason),
                                };

                                self.state.last_transition = Some(transition.clone());
                                self.events.push(RouterEvent::Blocked {
                                    transition: transition.clone(),
                                    state: self.state.clone(),
                                });

                                return Ok(RouterUpdate::Blocked(transition));
                            }
                        }
                    }
                }

                if !self.history.navigate(action, None) {
                    return Ok(RouterUpdate::NoChange);
                }

                self.history.refresh();
                let attempted = self.history.current().clone();

                let mut final_cause = RouterTransitionCause::Navigate { action };
                let mut final_chain = Vec::<RouteLocation>::new();

                if self.guard.is_some() {
                    match self.resolve_guard_chain(action, &from, attempted.clone())? {
                        RouterGuardResolution::Allow {
                            cause,
                            action: nav_action,
                            to,
                            redirect_chain,
                        } => {
                            if matches!(cause, RouterTransitionCause::Redirect { .. })
                                || !redirect_chain.is_empty()
                            {
                                if self.history.navigate(nav_action, Some(to.clone())) {
                                    final_cause = cause;
                                    final_chain = redirect_chain;
                                }
                            }
                        }
                        RouterGuardResolution::Block {
                            cause,
                            action: _nav_action,
                            attempted,
                            redirect_chain,
                            reason,
                        } => {
                            let _ = self
                                .history
                                .navigate(NavigationAction::Replace, Some(from.clone()));
                            self.history.refresh();

                            let transition = RouterTransition {
                                cause,
                                from,
                                to: attempted,
                                redirect_chain,
                                blocked_by: Some(reason),
                            };

                            self.state.last_transition = Some(transition.clone());
                            self.events.push(RouterEvent::Blocked {
                                transition: transition.clone(),
                                state: self.state.clone(),
                            });

                            return Ok(RouterUpdate::Blocked(transition));
                        }
                    }
                }

                return self.commit_changed_transition(final_cause, from, final_chain);
            }
        }
    }
}

fn normalize_redirect_action(action: NavigationAction) -> NavigationAction {
    match action {
        NavigationAction::Back | NavigationAction::Forward => NavigationAction::Replace,
        NavigationAction::Push | NavigationAction::Replace => action,
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
    fn router_guard_redirect_chain_supports_multiple_hops() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            A,
            B,
            C,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![
                    RouteNode::new(RouteId::A, "a").unwrap(),
                    RouteNode::new(RouteId::B, "b").unwrap(),
                    RouteNode::new(RouteId::C, "c").unwrap(),
                ]),
        ));

        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.set_guard(Some(Arc::new(|ctx| match ctx.to.path.as_str() {
            "/a" => RouterGuardDecision::redirect_replace(RouteLocation::parse("/b")),
            "/b" => RouterGuardDecision::redirect_replace(RouteLocation::parse("/c")),
            _ => RouterGuardDecision::Allow,
        })));

        let update = router
            .navigate(NavigationAction::Push, Some(RouteLocation::parse("/a")))
            .expect("navigate should succeed");
        assert!(matches!(update, super::RouterUpdate::Changed(_)));
        assert_eq!(router.state().location.to_url(), "/c");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Redirect { .. }
        ));
        assert_eq!(transition.redirect_chain.len(), 2);
        assert_eq!(transition.redirect_chain[0].to_url(), "/a");
        assert_eq!(transition.redirect_chain[1].to_url(), "/b");
    }

    #[test]
    fn router_guard_detects_redirect_loop() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            A,
            B,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![
                    RouteNode::new(RouteId::A, "a").unwrap(),
                    RouteNode::new(RouteId::B, "b").unwrap(),
                ]),
        ));

        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.set_guard(Some(Arc::new(|ctx| match ctx.to.path.as_str() {
            "/a" => RouterGuardDecision::redirect_replace(RouteLocation::parse("/b")),
            "/b" => RouterGuardDecision::redirect_replace(RouteLocation::parse("/a")),
            _ => RouterGuardDecision::Allow,
        })));

        let update = router
            .navigate(NavigationAction::Push, Some(RouteLocation::parse("/a")))
            .expect("navigate should succeed");
        assert!(matches!(update, super::RouterUpdate::Blocked(_)));
        assert_eq!(router.state().location.to_url(), "/");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(transition.blocked_by.is_some());
        assert!(
            transition
                .blocked_by
                .as_ref()
                .unwrap()
                .message
                .contains("loop")
        );
        assert!(!transition.redirect_chain.is_empty());
    }

    #[test]
    fn router_guard_caps_redirect_hops() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            A,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::A, "a").unwrap()]),
        ));

        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");
        let max = router.max_redirect_hops();

        router.set_guard(Some(Arc::new(|ctx| {
            if ctx.to.path != "/a" {
                return RouterGuardDecision::Allow;
            }

            let n = ctx
                .to
                .query_value("n")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let next = n + 1;
            RouterGuardDecision::redirect_replace(RouteLocation::parse(&format!("/a?n={next}")))
        })));

        let update = router
            .navigate(NavigationAction::Push, Some(RouteLocation::parse("/a?n=0")))
            .expect("navigate should succeed");
        assert!(matches!(update, super::RouterUpdate::Blocked(_)));

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(transition.blocked_by.is_some());
        assert!(
            transition
                .blocked_by
                .as_ref()
                .unwrap()
                .message
                .contains("hop")
        );
        assert_eq!(transition.redirect_chain.len(), max + 1);
    }

    #[test]
    fn router_guard_can_block_back_with_pre_guard_when_adapter_can_peek() {
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

        router.set_guard(Some(Arc::new(|ctx| match ctx.cause {
            super::RouterTransitionCause::Navigate {
                action: NavigationAction::Back,
            } if ctx.to.path == "/" => RouterGuardDecision::Block {
                reason: super::RouterBlockReason::new("blocked back"),
            },
            _ => RouterGuardDecision::Allow,
        })));

        assert!(
            router
                .navigate(
                    NavigationAction::Push,
                    Some(RouteLocation::parse("/settings")),
                )
                .expect("navigate should succeed")
                .changed()
        );
        let _ = router.take_events();

        let update = router
            .navigate(NavigationAction::Back, None)
            .expect("back should succeed");
        assert!(matches!(update, super::RouterUpdate::Blocked(_)));
        assert_eq!(router.state().location.to_url(), "/settings");

        let events = router.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::RouterEvent::Blocked { .. }));
    }

    #[test]
    fn router_guard_can_block_back_with_post_guard_when_adapter_cannot_peek() {
        #[derive(Debug, Clone)]
        struct NoPeekHistory {
            inner: MemoryHistory,
        }

        impl super::HistoryAdapter for NoPeekHistory {
            fn current(&self) -> &RouteLocation {
                self.inner.current()
            }

            fn navigate(
                &mut self,
                action: NavigationAction,
                target: Option<RouteLocation>,
            ) -> bool {
                self.inner.navigate(action, target)
            }
        }

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
        let history = NoPeekHistory {
            inner: MemoryHistory::new(RouteLocation::parse("/")),
        };
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.set_guard(Some(Arc::new(|ctx| match ctx.cause {
            super::RouterTransitionCause::Navigate {
                action: NavigationAction::Back,
            } if ctx.to.path == "/" => RouterGuardDecision::Block {
                reason: super::RouterBlockReason::new("blocked back"),
            },
            _ => RouterGuardDecision::Allow,
        })));

        assert!(
            router
                .navigate(
                    NavigationAction::Push,
                    Some(RouteLocation::parse("/settings")),
                )
                .expect("navigate should succeed")
                .changed()
        );
        let _ = router.take_events();

        let update = router
            .navigate(NavigationAction::Back, None)
            .expect("back should succeed");
        assert!(matches!(update, super::RouterUpdate::Blocked(_)));
        assert_eq!(router.state().location.to_url(), "/settings");

        let events = router.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::RouterEvent::Blocked { .. }));
    }

    #[test]
    fn memory_history_adapter_is_usable_via_trait() {
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.navigate(NavigationAction::Push, Some(RouteLocation::parse("/next"))));
        assert_eq!(history.current().to_url(), "/next");
    }
}
