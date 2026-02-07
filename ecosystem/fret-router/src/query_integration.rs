use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use fret_query::QueryKey;

use crate::RouteLocation;

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
    let canonical = location.canonicalized().to_url();
    QueryKey::new(namespace, &canonical)
}

pub fn route_query_key_with<T: 'static, H: Hash + ?Sized>(
    namespace: &'static str,
    location: &RouteLocation,
    extra: &H,
) -> QueryKey<T> {
    let canonical = location.canonicalized().to_url();
    let seed = RouteQueryKeySeedWithExtra {
        canonical_url: canonical.as_str(),
        extra,
    };
    QueryKey::new(namespace, &seed)
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
        NamespaceInvalidationRule, RouteChangePolicy, collect_invalidated_namespaces,
        route_change_matches, route_query_key, route_query_key_with,
    };
    use crate::RouteLocation;

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
}
