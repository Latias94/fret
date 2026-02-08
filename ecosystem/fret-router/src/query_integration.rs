use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use fret_query::QueryKey;

use crate::{RouteLocation, RouteMatchSnapshot, RoutePrefetchIntent, RouterTransition};

fn canonical_location_for_query_key(location: &RouteLocation) -> RouteLocation {
    let mut canonical = location.canonicalized();
    canonical.fragment = None;
    canonical
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteChangePolicy {
    Always,
    AnyChanged,
    PathChanged,
    QueryChanged,
    FragmentChanged,
    PathOrQueryChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamespaceInvalidationRule {
    pub namespace: &'static str,
    pub policy: RouteChangePolicy,
}

impl NamespaceInvalidationRule {
    pub fn new(namespace: &'static str, policy: RouteChangePolicy) -> Self {
        Self { namespace, policy }
    }
}

pub fn route_query_key<T: 'static>(
    namespace: &'static str,
    location: &RouteLocation,
) -> QueryKey<T> {
    let canonical = canonical_location_for_query_key(location).to_url();
    QueryKey::new(namespace, &canonical)
}

pub fn route_query_key_with<T: 'static, H: Hash + ?Sized>(
    namespace: &'static str,
    location: &RouteLocation,
    extra: &H,
) -> QueryKey<T> {
    let canonical = canonical_location_for_query_key(location).to_url();
    let seed = RouteQueryKeySeedWithExtra {
        canonical_url: canonical.as_str(),
        extra,
    };
    QueryKey::new(namespace, &seed)
}

pub fn prefetch_intent_query_key<T: 'static, R>(intent: &RoutePrefetchIntent<R>) -> QueryKey<T> {
    if let Some(extra) = intent.extra {
        route_query_key_with(intent.namespace, &intent.location, extra)
    } else {
        route_query_key(intent.namespace, &intent.location)
    }
}

pub fn route_change_matches(
    previous: &RouteLocation,
    current: &RouteLocation,
    policy: RouteChangePolicy,
) -> bool {
    let previous = previous.canonicalized();
    let current = current.canonicalized();

    match policy {
        RouteChangePolicy::Always => true,
        RouteChangePolicy::AnyChanged => previous != current,
        RouteChangePolicy::PathChanged => previous.path != current.path,
        RouteChangePolicy::QueryChanged => previous.query != current.query,
        RouteChangePolicy::FragmentChanged => previous.fragment != current.fragment,
        RouteChangePolicy::PathOrQueryChanged => {
            previous.path != current.path || previous.query != current.query
        }
    }
}

pub fn collect_invalidated_namespaces(
    previous: &RouteLocation,
    current: &RouteLocation,
    rules: &[NamespaceInvalidationRule],
) -> Vec<&'static str> {
    let mut seen = HashSet::<&'static str>::new();
    let mut out = Vec::new();

    for rule in rules {
        if route_change_matches(previous, current, rule.policy) && seen.insert(rule.namespace) {
            out.push(rule.namespace);
        }
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RoutePrefetchRule<R> {
    pub route: R,
    pub namespace: &'static str,
    pub policy: RouteChangePolicy,
    pub extra: Option<&'static str>,
}

#[allow(dead_code)]
impl<R> RoutePrefetchRule<R> {
    pub fn new(route: R, namespace: &'static str, policy: RouteChangePolicy) -> Self {
        Self {
            route,
            namespace,
            policy,
            extra: None,
        }
    }

    pub fn with_extra(mut self, extra: &'static str) -> Self {
        self.extra = Some(extra);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RoutePrefetchPlanItem<R> {
    pub route: R,
    pub namespace: &'static str,
    pub location: RouteLocation,
    pub extra: Option<&'static str>,
}

#[allow(dead_code)]
impl<R> RoutePrefetchPlanItem<R> {
    pub fn query_key<T: 'static>(&self) -> QueryKey<T> {
        if let Some(extra) = self.extra {
            route_query_key_with(self.namespace, &self.location, &extra)
        } else {
            route_query_key(self.namespace, &self.location)
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RouteTransitionPlan<R> {
    pub invalidated_namespaces: Vec<&'static str>,
    pub prefetches: Vec<RoutePrefetchPlanItem<R>>,
}

#[allow(dead_code)]
pub fn plan_route_transition<R>(
    transition: &RouterTransition,
    next_matches: &[RouteMatchSnapshot<R>],
    invalidation_rules: &[NamespaceInvalidationRule],
    prefetch_rules: &[RoutePrefetchRule<R>],
) -> RouteTransitionPlan<R>
where
    R: Clone + Eq + Hash,
{
    let invalidated_namespaces =
        collect_invalidated_namespaces(&transition.from, &transition.to, invalidation_rules);

    let mut next_by_route = std::collections::HashMap::<&R, &RouteMatchSnapshot<R>>::new();
    for entry in next_matches {
        next_by_route.entry(&entry.route).or_insert(entry);
    }

    let mut seen = HashSet::<(String, &'static str, Option<&'static str>)>::new();
    let mut prefetches = Vec::new();

    for rule in prefetch_rules {
        if !route_change_matches(&transition.from, &transition.to, rule.policy) {
            continue;
        }

        let Some(matched) = next_by_route.get(&rule.route) else {
            continue;
        };

        let location = RouteLocation {
            path: matched.matched_path.clone(),
            query: matched.search.clone().into_pairs(),
            fragment: None,
        };
        let signature = (
            canonical_location_for_query_key(&location).to_url(),
            rule.namespace,
            rule.extra,
        );
        if !seen.insert(signature) {
            continue;
        }

        prefetches.push(RoutePrefetchPlanItem {
            route: rule.route.clone(),
            namespace: rule.namespace,
            location,
            extra: rule.extra,
        });
    }

    RouteTransitionPlan {
        invalidated_namespaces,
        prefetches,
    }
}

#[derive(Debug)]
struct RouteQueryKeySeedWithExtra<'a, H: Hash + ?Sized> {
    canonical_url: &'a str,
    extra: &'a H,
}

impl<H: Hash + ?Sized> Hash for RouteQueryKeySeedWithExtra<'_, H> {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.canonical_url.hash(state);
        self.extra.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NamespaceInvalidationRule, RouteChangePolicy, RoutePrefetchRule,
        collect_invalidated_namespaces, plan_route_transition, prefetch_intent_query_key,
        route_change_matches, route_query_key, route_query_key_with,
    };
    use crate::{
        NavigationAction, RouteLocation, RouteMatchSnapshot, RoutePrefetchIntent, RouterTransition,
        SearchMap,
    };

    #[test]
    fn route_query_key_uses_canonical_location() {
        let left = RouteLocation::parse("/users/42?b=2&a=1");
        let right = RouteLocation::parse("users///42/?a=1&b=2");

        let left_key = route_query_key::<u8>("fret.router.user.v1", &left);
        let right_key = route_query_key::<u8>("fret.router.user.v1", &right);

        assert_eq!(left_key.namespace(), right_key.namespace());
        assert_eq!(left_key.hash(), right_key.hash());
    }

    #[test]
    fn route_query_key_with_includes_extra_scope() {
        let location = RouteLocation::parse("/users/42");

        let a = route_query_key_with::<u8, _>("fret.router.user.v1", &location, &"summary");
        let b = route_query_key_with::<u8, _>("fret.router.user.v1", &location, &"detail");

        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn route_query_key_ignores_fragment() {
        let left = RouteLocation::parse("/users/42?tab=profile#section-1");
        let right = RouteLocation::parse("/users/42?tab=profile#section-2");

        let left_key = route_query_key::<u8>("fret.router.user.v1", &left);
        let right_key = route_query_key::<u8>("fret.router.user.v1", &right);

        assert_eq!(left_key.namespace(), right_key.namespace());
        assert_eq!(left_key.hash(), right_key.hash());
    }

    #[test]
    fn prefetch_intent_query_key_ignores_fragment_and_respects_extra() {
        let with_fragment = RouteLocation::parse("/users/42?b=2&a=1#tab");
        let without_fragment = RouteLocation::parse("/users/42?a=1&b=2");

        let intent_without_extra = RoutePrefetchIntent {
            route: 1u8,
            namespace: "fret.router.test.users.v1",
            location: with_fragment.clone(),
            extra: None,
        };
        let key_without_extra_a = prefetch_intent_query_key::<u8, _>(&intent_without_extra);

        let intent_without_fragment = RoutePrefetchIntent {
            location: without_fragment,
            ..intent_without_extra
        };
        let key_without_extra_b = prefetch_intent_query_key::<u8, _>(&intent_without_fragment);
        assert_eq!(key_without_extra_a.hash(), key_without_extra_b.hash());

        let intent_with_extra = RoutePrefetchIntent {
            route: 1u8,
            namespace: "fret.router.test.users.v1",
            location: with_fragment,
            extra: Some("scope"),
        };
        let key_with_extra = prefetch_intent_query_key::<u8, _>(&intent_with_extra);
        assert_ne!(key_without_extra_a.hash(), key_with_extra.hash());
    }

    #[test]
    fn route_change_policy_matches_expected_fields() {
        let previous = RouteLocation::parse("/users/42?tab=summary#section-1");
        let current = RouteLocation::parse("/users/42?tab=detail#section-2");

        assert!(route_change_matches(
            &previous,
            &current,
            RouteChangePolicy::AnyChanged,
        ));
        assert!(!route_change_matches(
            &previous,
            &current,
            RouteChangePolicy::PathChanged,
        ));
        assert!(route_change_matches(
            &previous,
            &current,
            RouteChangePolicy::QueryChanged,
        ));
        assert!(route_change_matches(
            &previous,
            &current,
            RouteChangePolicy::FragmentChanged,
        ));
        assert!(route_change_matches(
            &previous,
            &current,
            RouteChangePolicy::PathOrQueryChanged,
        ));
    }

    #[test]
    fn collect_invalidated_namespaces_is_ordered_and_deduped() {
        let previous = RouteLocation::parse("/users/42?tab=summary");
        let current = RouteLocation::parse("/users/43?tab=detail");

        let namespaces = collect_invalidated_namespaces(
            &previous,
            &current,
            &[
                NamespaceInvalidationRule::new(
                    "fret.user.detail.v1",
                    RouteChangePolicy::PathChanged,
                ),
                NamespaceInvalidationRule::new(
                    "fret.user.list.v1",
                    RouteChangePolicy::QueryChanged,
                ),
                NamespaceInvalidationRule::new("fret.user.detail.v1", RouteChangePolicy::Always),
            ],
        );

        assert_eq!(namespaces, vec!["fret.user.detail.v1", "fret.user.list.v1"]);
    }

    #[test]
    fn plan_route_transition_collects_invalidations_and_prefetches() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            User,
        }

        let transition = RouterTransition::navigate(
            NavigationAction::Push,
            RouteLocation::parse("/users/1?tab=profile"),
            RouteLocation::parse("/users/2?tab=profile"),
        );

        let next_matches = vec![
            RouteMatchSnapshot {
                route: RouteId::Root,
                matched_path: "/".to_string(),
                params: Vec::new(),
                search: SearchMap::from_location(&transition.to),
                search_error: None,
            },
            RouteMatchSnapshot {
                route: RouteId::User,
                matched_path: "/users/2".to_string(),
                params: Vec::new(),
                search: SearchMap::from_location(&transition.to),
                search_error: None,
            },
        ];

        let plan = plan_route_transition(
            &transition,
            &next_matches,
            &[NamespaceInvalidationRule::new(
                "fret.user.detail.v1",
                RouteChangePolicy::PathChanged,
            )],
            &[RoutePrefetchRule::new(
                RouteId::User,
                "fret.user.detail.v1",
                RouteChangePolicy::PathChanged,
            )],
        );

        assert_eq!(plan.invalidated_namespaces, vec!["fret.user.detail.v1"]);
        assert_eq!(plan.prefetches.len(), 1);
        assert_eq!(plan.prefetches[0].location.path, "/users/2");
        assert_eq!(plan.prefetches[0].namespace, "fret.user.detail.v1");

        let key = plan.prefetches[0].query_key::<u8>();
        assert_eq!(key.namespace(), "fret.user.detail.v1");
    }

    #[test]
    fn plan_route_transition_dedupes_prefetch_entries() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
        }

        let transition = RouterTransition::navigate(
            NavigationAction::Replace,
            RouteLocation::parse("/?a=1"),
            RouteLocation::parse("/?a=2"),
        );

        let next_matches = vec![RouteMatchSnapshot {
            route: RouteId::Root,
            matched_path: "/".to_string(),
            params: Vec::new(),
            search: SearchMap::from_location(&transition.to),
            search_error: None,
        }];

        let rule =
            RoutePrefetchRule::new(RouteId::Root, "fret.root.v1", RouteChangePolicy::AnyChanged);
        let plan = plan_route_transition(&transition, &next_matches, &[], &[rule.clone(), rule]);

        assert_eq!(plan.prefetches.len(), 1);
    }
}
