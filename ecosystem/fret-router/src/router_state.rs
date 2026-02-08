use std::hash::Hash;
use std::sync::Arc;

use crate::{
    MemoryHistory, NavigationAction, RouteLocation, RouteMatchResultWithSearch, RouteSearchTable,
    RouteSearchValidationFailure, RouteTree, SearchValidationMode,
};

const DEFAULT_MAX_REDIRECT_HOPS: usize = 4;

const REDIRECT_LOOP_REASON: &str = "redirect loop detected";
const REDIRECT_HOP_LIMIT_REASON: &str = "redirect hop limit exceeded";

#[derive(Debug, Clone)]
pub struct RoutePrefetchIntent<R> {
    pub route: R,
    pub namespace: &'static str,
    pub location: RouteLocation,
    pub extra: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub enum RouterBuildLocationError {
    UnknownRoute,
    MissingPathParams,
    SearchValidation(RouteSearchValidationFailure),
}

impl std::fmt::Display for RouterBuildLocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRoute => f.write_str("unknown route"),
            Self::MissingPathParams => f.write_str("missing path params"),
            Self::SearchValidation(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for RouterBuildLocationError {}

#[derive(Debug, Clone)]
pub enum RouterNavigateToError {
    BuildLocation(RouterBuildLocationError),
    SearchValidation(RouteSearchValidationFailure),
}

impl std::fmt::Display for RouterNavigateToError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildLocation(err) => write!(f, "build location failed: {err}"),
            Self::SearchValidation(err) => write!(f, "route search validation failed: {err}"),
        }
    }
}

impl std::error::Error for RouterNavigateToError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BuildLocation(err) => Some(err),
            Self::SearchValidation(err) => Some(err),
        }
    }
}

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
    Init,
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
pub struct RouteHookContext<'a, R> {
    pub cause: RouterTransitionCause,
    pub from: &'a RouteLocation,
    pub to: &'a RouteLocation,
    pub from_state: &'a RouterState<R>,
    pub next_state: &'a RouterState<R>,
    pub match_index: usize,
    pub matched: &'a RouteMatchSnapshot<R>,
}

pub type RouteBeforeLoadFn<R> =
    Arc<dyn for<'a> Fn(&RouteHookContext<'a, R>) -> RouterGuardDecision + Send + Sync>;

pub type RouteLoaderFn<R> =
    Arc<dyn for<'a> Fn(&RouteHookContext<'a, R>) -> Vec<RoutePrefetchIntent<R>> + Send + Sync>;

#[derive(Clone)]
pub struct RouteHooksTable<R> {
    hooks: std::collections::HashMap<R, RouteHooks<R>>,
}

impl<R> Default for RouteHooksTable<R> {
    fn default() -> Self {
        Self {
            hooks: std::collections::HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct RouteHooks<R> {
    pub before_load: Option<RouteBeforeLoadFn<R>>,
    pub loader: Option<RouteLoaderFn<R>>,
}

impl<R> Default for RouteHooks<R> {
    fn default() -> Self {
        Self {
            before_load: None,
            loader: None,
        }
    }
}

impl<R> RouteHooksTable<R>
where
    R: Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, route: R, hooks: RouteHooks<R>) -> Option<RouteHooks<R>> {
        self.hooks.insert(route, hooks)
    }

    pub fn hooks_for(&self, route: &R) -> Option<&RouteHooks<R>> {
        self.hooks.get(route)
    }
}

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
            RouterTransitionCause::Init => None,
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
pub struct RouterUpdateWithPrefetchIntents<R> {
    pub update: RouterUpdate,
    pub intents: Vec<RoutePrefetchIntent<R>>,
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
    route_hooks: RouteHooksTable<R>,
    prefetch_intents: Vec<RoutePrefetchIntent<R>>,
    max_redirect_hops: usize,
}

impl<R, H> Router<R, H>
where
    R: Clone + Hash + Eq,
    H: HistoryAdapter,
{
    fn route_chain_nodes<'a>(
        node: &'a crate::RouteNode<R>,
        target: &R,
        out: &mut Vec<&'a crate::RouteNode<R>>,
    ) -> bool {
        out.push(node);
        if &node.route == target {
            return true;
        }

        for child in &node.children {
            if Self::route_chain_nodes(child, target, out) {
                return true;
            }
        }

        out.pop();
        false
    }

    fn build_path_for_chain(
        chain: &[&crate::RouteNode<R>],
        params: &[crate::PathParam],
        matched_paths_out: &mut Vec<String>,
    ) -> Result<String, RouterBuildLocationError> {
        let mut segments: Vec<String> = Vec::new();
        matched_paths_out.clear();

        for node in chain {
            let formatted = node
                .pattern
                .format_path(params)
                .ok_or(RouterBuildLocationError::MissingPathParams)?;
            for seg in formatted.split('/').filter(|seg| !seg.is_empty()) {
                segments.push(seg.to_string());
            }

            let matched = if segments.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", segments.join("/"))
            };
            matched_paths_out.push(matched);
        }

        Ok(if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        })
    }

    pub fn build_location(
        &self,
        route: &R,
        params: &[crate::PathParam],
        search: crate::SearchMap,
        fragment: Option<String>,
    ) -> Result<RouteLocation, RouterBuildLocationError> {
        let mut chain: Vec<&crate::RouteNode<R>> = Vec::new();
        if !Self::route_chain_nodes(&self.tree.root, route, &mut chain) {
            return Err(RouterBuildLocationError::UnknownRoute);
        }

        let mut matched_paths: Vec<String> = Vec::with_capacity(chain.len());
        let path = Self::build_path_for_chain(&chain, params, &mut matched_paths)?;

        let mut location = RouteLocation {
            path,
            query: search.into_pairs(),
            fragment,
        };
        location.canonicalize();

        let mut accumulated = crate::SearchMap::from_location(&location);
        for (index, node) in chain.iter().enumerate() {
            let Some(validator) = self.search_table.validator_for(&node.route) else {
                continue;
            };
            match validator(&location, &accumulated) {
                Ok(next) => {
                    accumulated = next;
                }
                Err(err) => {
                    if self.search_mode == SearchValidationMode::Strict {
                        return Err(RouterBuildLocationError::SearchValidation(
                            RouteSearchValidationFailure {
                                match_index: index,
                                matched_path: matched_paths
                                    .get(index)
                                    .cloned()
                                    .unwrap_or_else(|| "/".to_string()),
                                error: err,
                            },
                        ));
                    }
                }
            }
        }

        location.query = accumulated.into_pairs();
        location.canonicalize();
        Ok(location)
    }

    pub fn build_location_for_route(
        &self,
        route: &R,
        params: &[crate::PathParam],
    ) -> Result<RouteLocation, RouterBuildLocationError> {
        self.build_location(route, params, crate::SearchMap::new(), None)
    }

    pub fn href_to(
        &self,
        route: &R,
        params: &[crate::PathParam],
        search: crate::SearchMap,
        fragment: Option<String>,
    ) -> Result<String, RouterBuildLocationError> {
        Ok(self
            .build_location(route, params, search, fragment)?
            .to_url())
    }

    pub fn href_to_route(
        &self,
        route: &R,
        params: &[crate::PathParam],
    ) -> Result<String, RouterBuildLocationError> {
        Ok(self.build_location_for_route(route, params)?.to_url())
    }

    fn route_before_load_decision(
        &self,
        cause: RouterTransitionCause,
        from: &RouteLocation,
        to: &RouteLocation,
        next_state: &RouterState<R>,
    ) -> RouterGuardDecision {
        for (match_index, matched) in next_state.matches.iter().enumerate() {
            let Some(hooks) = self.route_hooks.hooks_for(&matched.route) else {
                continue;
            };

            let Some(before_load) = hooks.before_load.as_ref() else {
                continue;
            };

            let ctx = RouteHookContext {
                cause,
                from,
                to,
                from_state: &self.state,
                next_state,
                match_index,
                matched,
            };

            match before_load(&ctx) {
                RouterGuardDecision::Allow => {}
                other => return other,
            }
        }

        RouterGuardDecision::Allow
    }

    fn collect_loader_intents(
        &mut self,
        from_state: &RouterState<R>,
        transition: &RouterTransition,
    ) {
        let to = &transition.to;
        for (match_index, matched) in self.state.matches.iter().enumerate() {
            let Some(hooks) = self.route_hooks.hooks_for(&matched.route) else {
                continue;
            };

            let Some(loader) = hooks.loader.as_ref() else {
                continue;
            };

            let ctx = RouteHookContext {
                cause: transition.cause,
                from: &transition.from,
                to,
                from_state,
                next_state: &self.state,
                match_index,
                matched,
            };

            self.prefetch_intents.extend(loader(&ctx));
        }
    }

    fn resolve_guard_chain(
        &self,
        initial_cause: RouterTransitionCause,
        original_action: NavigationAction,
        from: &RouteLocation,
        initial_target: RouteLocation,
    ) -> Result<RouterGuardResolution, RouteSearchValidationFailure> {
        let force_replace_redirects = matches!(
            initial_cause,
            RouterTransitionCause::Sync | RouterTransitionCause::Init
        );
        let mut redirect_chain = Vec::<RouteLocation>::new();
        let mut seen = std::collections::HashSet::<String>::new();
        let mut current_action = original_action;
        let mut current_cause = initial_cause;
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

            if let Some(guard) = self.guard.as_ref() {
                match guard(&ctx) {
                    RouterGuardDecision::Allow => {}
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
                        current_action = if force_replace_redirects {
                            NavigationAction::Replace
                        } else {
                            normalize_redirect_action(redirect_action)
                        };
                        current_cause = RouterTransitionCause::Redirect {
                            action: current_action,
                        };
                        current_target = redirect_to.canonicalized();
                        continue;
                    }
                }
            }

            match self.route_before_load_decision(current_cause, from, &current_target, &next_state)
            {
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
                    current_action = if force_replace_redirects {
                        NavigationAction::Replace
                    } else {
                        normalize_redirect_action(redirect_action)
                    };
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
        let from_state = self.state.clone();
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
        self.collect_loader_intents(&from_state, &transition);

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
            route_hooks: RouteHooksTable::new(),
            prefetch_intents: Vec::new(),
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

    pub fn route_hooks(&self) -> &RouteHooksTable<R> {
        &self.route_hooks
    }

    pub fn route_hooks_mut(&mut self) -> &mut RouteHooksTable<R> {
        &mut self.route_hooks
    }

    pub fn take_prefetch_intents(&mut self) -> Vec<RoutePrefetchIntent<R>> {
        std::mem::take(&mut self.prefetch_intents)
    }

    pub fn sync_with_prefetch_intents(
        &mut self,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        match self.sync() {
            Ok(update) => Ok(RouterUpdateWithPrefetchIntents {
                update,
                intents: self.take_prefetch_intents(),
            }),
            Err(err) => {
                let _ = self.take_prefetch_intents();
                Err(err)
            }
        }
    }

    pub fn init_with_prefetch_intents(
        &mut self,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        match self.init() {
            Ok(update) => Ok(RouterUpdateWithPrefetchIntents {
                update,
                intents: self.take_prefetch_intents(),
            }),
            Err(err) => {
                let _ = self.take_prefetch_intents();
                Err(err)
            }
        }
    }

    pub fn navigate_with_prefetch_intents(
        &mut self,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        match self.navigate(action, target) {
            Ok(update) => Ok(RouterUpdateWithPrefetchIntents {
                update,
                intents: self.take_prefetch_intents(),
            }),
            Err(err) => {
                let _ = self.take_prefetch_intents();
                Err(err)
            }
        }
    }

    pub fn navigate_to(
        &mut self,
        action: NavigationAction,
        route: &R,
        params: &[crate::PathParam],
        search: crate::SearchMap,
        fragment: Option<String>,
    ) -> Result<RouterUpdate, RouterNavigateToError> {
        let location = self
            .build_location(route, params, search, fragment)
            .map_err(RouterNavigateToError::BuildLocation)?;
        self.navigate(action, Some(location))
            .map_err(RouterNavigateToError::SearchValidation)
    }

    pub fn navigate_to_with_prefetch_intents(
        &mut self,
        action: NavigationAction,
        route: &R,
        params: &[crate::PathParam],
        search: crate::SearchMap,
        fragment: Option<String>,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouterNavigateToError> {
        let location = self
            .build_location(route, params, search, fragment)
            .map_err(RouterNavigateToError::BuildLocation)?;

        self.navigate_with_prefetch_intents(action, Some(location))
            .map_err(RouterNavigateToError::SearchValidation)
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

    pub fn init(&mut self) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let from = self.history.current().clone();
        let from_state = self.state.clone();

        match self.resolve_guard_chain(
            RouterTransitionCause::Init,
            NavigationAction::Replace,
            &from,
            from.canonicalized(),
        )? {
            RouterGuardResolution::Allow {
                cause,
                action: nav_action,
                to,
                redirect_chain,
            } => {
                if matches!(cause, RouterTransitionCause::Redirect { .. })
                    || !redirect_chain.is_empty()
                    || to != from
                {
                    if !self.history.navigate(nav_action, Some(to.clone())) {
                        return Ok(RouterUpdate::NoChange);
                    }

                    return self.commit_changed_transition(cause, from, redirect_chain);
                }

                let transition = RouterTransition {
                    cause: RouterTransitionCause::Init,
                    from: self.state.location.clone(),
                    to: self.state.location.clone(),
                    redirect_chain: Vec::new(),
                    blocked_by: None,
                };
                self.collect_loader_intents(&from_state, &transition);
                Ok(RouterUpdate::NoChange)
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

                Ok(RouterUpdate::Blocked(transition))
            }
        }
    }

    pub fn sync(&mut self) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let next_location = self.history.current().clone();
        if next_location == self.state.location {
            return Ok(RouterUpdate::NoChange);
        }

        let from = self.state.location.clone();
        let from_state = self.state.clone();

        match self.resolve_guard_chain(
            RouterTransitionCause::Sync,
            NavigationAction::Replace,
            &from,
            next_location.canonicalized(),
        )? {
            RouterGuardResolution::Allow {
                cause,
                action: nav_action,
                to,
                redirect_chain,
            } => {
                if matches!(cause, RouterTransitionCause::Redirect { .. })
                    || !redirect_chain.is_empty()
                {
                    if !self.history.navigate(nav_action, Some(to.clone())) {
                        return Ok(RouterUpdate::NoChange);
                    }
                    return self.commit_changed_transition(cause, from, redirect_chain);
                }

                let next_state =
                    RouterState::from_match_result(self.tree.match_routes_with_search(
                        &next_location,
                        self.search_table.as_ref(),
                        self.search_mode,
                    )?);

                let transition = RouterTransition::sync(
                    self.state.location.clone(),
                    next_state.location.clone(),
                );
                self.state = next_state;
                self.state.last_transition = Some(transition.clone());
                self.events.push(RouterEvent::Transitioned {
                    transition: transition.clone(),
                    state: self.state.clone(),
                });
                self.collect_loader_intents(&from_state, &transition);

                Ok(RouterUpdate::Changed(transition))
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

                Ok(RouterUpdate::Blocked(transition))
            }
        }
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
                match self.resolve_guard_chain(
                    RouterTransitionCause::Navigate { action },
                    action,
                    &from,
                    proposed_location,
                )? {
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
                        match self.resolve_guard_chain(
                            RouterTransitionCause::Navigate { action },
                            action,
                            &from,
                            attempted,
                        )? {
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
                    match self.resolve_guard_chain(
                        RouterTransitionCause::Navigate { action },
                        action,
                        &from,
                        attempted.clone(),
                    )? {
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
    use super::{
        RouteHooks, Router, RouterBlockReason, RouterBuildLocationError, RouterGuardDecision,
        RouterUpdate, RouterUpdateWithPrefetchIntents,
    };
    use crate::{
        MemoryHistory, NavigationAction, PathParam, RouteLocation, RouteNode, RouteSearchTable,
        RouteTree, SearchMap, SearchValidationError, SearchValidationMode,
    };
    use std::sync::{Arc, Mutex};

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
    fn router_init_collects_loader_intents_without_location_change() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
        }

        let tree = Arc::new(RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap()));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.route_hooks_mut().insert(
            RouteId::Root,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    assert!(matches!(ctx.cause, super::RouterTransitionCause::Init));
                    vec![super::RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "test.ns",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let update = router
            .init_with_prefetch_intents()
            .expect("init should succeed");
        assert!(matches!(update.update, RouterUpdate::NoChange));
        assert_eq!(update.intents.len(), 1);
    }

    #[test]
    fn router_sync_runs_before_load_and_can_redirect() {
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

        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: Some(Arc::new(|ctx| {
                    if ctx.to.path == "/settings" {
                        RouterGuardDecision::Redirect {
                            action: NavigationAction::Replace,
                            to: RouteLocation::parse("/"),
                        }
                    } else {
                        RouterGuardDecision::Allow
                    }
                })),
                loader: None,
            },
        );

        assert!(router.history_mut().push(RouteLocation::parse("/settings")));
        let update = router.sync().expect("sync should succeed");
        assert!(update.changed());
        assert_eq!(router.state().location.to_url(), "/");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Redirect { .. }
        ));
    }

    #[test]
    fn router_navigate_to_builds_location_and_navigates() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
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
                .with_children(vec![RouteNode::new(RouteId::User, "users/:id").unwrap()]),
        ));

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        let search_table = Arc::new(search_table);

        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        let update = router
            .navigate_to(
                NavigationAction::Push,
                &RouteId::User,
                &[PathParam {
                    name: "id".to_string(),
                    value: "42".to_string(),
                }],
                SearchMap::new(),
                None,
            )
            .expect("navigate_to should succeed");
        assert!(update.changed());
        assert_eq!(router.state().location.to_url(), "/users/42?lang=en");
    }

    #[test]
    fn router_navigate_to_errors_when_params_missing() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::User, "users/:id").unwrap()]),
        ));

        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        let err = router
            .navigate_to(
                NavigationAction::Push,
                &RouteId::User,
                &[],
                SearchMap::new(),
                None,
            )
            .expect_err("expected navigate_to to fail");
        assert!(matches!(
            err,
            super::RouterNavigateToError::BuildLocation(
                super::RouterBuildLocationError::MissingPathParams
            )
        ));
    }

    #[test]
    fn router_href_to_formats_path_and_stabilizes_search() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
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
                .with_children(vec![RouteNode::new(RouteId::User, "users/:id").unwrap()]),
        ));

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        let search_table = Arc::new(search_table);

        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        let href = router
            .href_to(
                &RouteId::User,
                &[PathParam {
                    name: "id".to_string(),
                    value: "42".to_string(),
                }],
                SearchMap::new().with_typed("debug", Some(true)),
                Some("section-1".to_string()),
            )
            .expect("href_to should succeed");

        assert_eq!(href, "/users/42?debug=true&lang=en#section-1");
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
    fn navigate_with_prefetch_intents_returns_update_scoped_intents() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Gallery,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Gallery, "gallery").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.route_hooks_mut().insert(
            RouteId::Gallery,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    vec![super::RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "tests.intent",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let RouterUpdateWithPrefetchIntents { update, intents } = router
            .navigate_with_prefetch_intents(
                NavigationAction::Push,
                Some(RouteLocation::parse("/gallery")),
            )
            .expect("navigate should succeed");

        assert!(update.changed());
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].location.to_url(), "/gallery");
        assert!(router.take_prefetch_intents().is_empty());
    }

    #[test]
    fn navigate_with_prefetch_intents_clears_stale_intents_on_search_failure() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Gallery,
        }

        fn validate_root(
            _location: &RouteLocation,
            search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            if search.first("bad").is_some() {
                Err(SearchValidationError::new("bad search key not allowed"))
            } else {
                Ok(search.clone())
            }
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
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        router.route_hooks_mut().insert(
            RouteId::Gallery,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    vec![super::RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "tests.intent",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/gallery")),
            )
            .expect("navigate should succeed");
        assert!(update.changed());
        assert_eq!(
            router.prefetch_intents.len(),
            1,
            "expected the first transition to queue a prefetch intent"
        );

        let update = router.navigate_with_prefetch_intents(
            NavigationAction::Push,
            Some(RouteLocation::parse("/gallery?bad=1")),
        );
        assert!(update.is_err());
        assert!(
            router.prefetch_intents.is_empty(),
            "expected navigate_with_prefetch_intents() to drain stale intents on failure"
        );
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
    fn route_before_load_runs_root_to_leaf_and_can_block() {
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

        let calls = Arc::new(Mutex::new(Vec::<RouteId>::new()));

        let root_calls = calls.clone();
        router.route_hooks_mut().insert(
            RouteId::Root,
            RouteHooks {
                before_load: Some(Arc::new(move |ctx| {
                    root_calls.lock().expect("lock").push(ctx.matched.route);
                    RouterGuardDecision::Allow
                })),
                loader: None,
            },
        );

        let settings_calls = calls.clone();
        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: Some(Arc::new(move |ctx| {
                    settings_calls.lock().expect("lock").push(ctx.matched.route);
                    RouterGuardDecision::Block {
                        reason: RouterBlockReason::new("blocked by before_load"),
                    }
                })),
                loader: None,
            },
        );

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(matches!(update, RouterUpdate::Blocked(_)));
        assert_eq!(router.state().location.to_url(), "/");

        let calls = calls.lock().expect("lock").clone();
        assert_eq!(calls, vec![RouteId::Root, RouteId::Settings]);
    }

    #[test]
    fn route_before_load_can_redirect() {
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

        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: Some(Arc::new(|ctx| {
                    if ctx.to.path == "/settings" {
                        RouterGuardDecision::redirect_replace(RouteLocation::parse("/login"))
                    } else {
                        RouterGuardDecision::Allow
                    }
                })),
                loader: None,
            },
        );

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(matches!(update, RouterUpdate::Changed(_)));
        assert_eq!(router.state().location.to_url(), "/login");

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Redirect {
                action: NavigationAction::Replace
            }
        ));
        assert_eq!(transition.redirect_chain.len(), 1);
        assert_eq!(transition.redirect_chain[0].to_url(), "/settings");
    }

    #[test]
    fn route_loader_collects_prefetch_intents_with_context() {
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

        let observed = Arc::new(Mutex::new(Vec::<(usize, RouteId, String, String)>::new()));

        let root_observed = observed.clone();
        router.route_hooks_mut().insert(
            RouteId::Root,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(move |ctx| {
                    root_observed.lock().expect("lock").push((
                        ctx.match_index,
                        ctx.matched.route,
                        ctx.from_state.location.to_url(),
                        ctx.next_state.location.to_url(),
                    ));
                    vec![super::RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "fret.router.test.root.v1",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let settings_observed = observed.clone();
        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(move |ctx| {
                    settings_observed.lock().expect("lock").push((
                        ctx.match_index,
                        ctx.matched.route,
                        ctx.from_state.location.to_url(),
                        ctx.next_state.location.to_url(),
                    ));
                    vec![super::RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "fret.router.test.settings.v1",
                        location: ctx.to.clone(),
                        extra: Some("scope"),
                    }]
                })),
            },
        );

        let update = router
            .navigate(
                NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(matches!(update, RouterUpdate::Changed(_)));

        let intents = router.take_prefetch_intents();
        assert_eq!(intents.len(), 2);
        assert!(router.take_prefetch_intents().is_empty());

        let observed = observed.lock().expect("lock").clone();
        assert_eq!(observed.len(), 2);
        assert_eq!(observed[0].0, 0);
        assert_eq!(observed[0].1, RouteId::Root);
        assert_eq!(observed[0].2, "/");
        assert_eq!(observed[0].3, "/settings");
        assert_eq!(observed[1].0, 1);
        assert_eq!(observed[1].1, RouteId::Settings);
        assert_eq!(observed[1].2, "/");
        assert_eq!(observed[1].3, "/settings");
    }

    #[test]
    fn memory_history_adapter_is_usable_via_trait() {
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.navigate(NavigationAction::Push, Some(RouteLocation::parse("/next"))));
        assert_eq!(history.current().to_url(), "/next");
    }

    #[test]
    fn build_location_formats_path_and_stabilizes_search() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
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
                .with_children(vec![RouteNode::new(RouteId::User, "users/:id").unwrap()]),
        ));

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        let search_table = Arc::new(search_table);

        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        let location = router
            .build_location(
                &RouteId::User,
                &[crate::PathParam {
                    name: "id".to_string(),
                    value: "42".to_string(),
                }],
                SearchMap::new(),
                None,
            )
            .expect("build_location should succeed");

        assert_eq!(location.to_url(), "/users/42?lang=en");
    }

    #[test]
    fn build_location_errors_when_params_missing() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::User, "users/:id").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        let err = router
            .build_location_for_route(&RouteId::User, &[])
            .expect_err("expected build_location_for_route to fail");
        assert!(matches!(err, RouterBuildLocationError::MissingPathParams));
    }
}
