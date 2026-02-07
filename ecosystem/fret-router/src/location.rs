use crate::base_path::{apply_base_path, strip_base_path};
use crate::path::{PathParam, PathPattern, normalize_path};
use crate::query::{
    QueryPair, decode_path_component, encode_component, format_query_pairs, parse_query_pairs,
    query_values,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteLocation {
    pub path: String,
    pub query: Vec<QueryPair>,
    pub fragment: Option<String>,
}

impl Default for RouteLocation {
    fn default() -> Self {
        Self {
            path: "/".to_string(),
            query: Vec::new(),
            fragment: None,
        }
    }
}

impl RouteLocation {
    pub fn parse(url: &str) -> Self {
        let url = url.trim();
        if url.is_empty() {
            return Self::default();
        }

        let mut path_part = url;
        let mut fragment = None::<String>;

        if let Some((before_hash, after_hash)) = path_part.split_once('#') {
            path_part = before_hash;
            if !after_hash.is_empty() {
                fragment = Some(decode_path_component(after_hash));
            }
        }

        let (path, query_raw) = if let Some((path_only, query)) = path_part.split_once('?') {
            (path_only, query)
        } else {
            (path_part, "")
        };

        Self {
            path: normalize_path(path),
            query: parse_query_pairs(query_raw),
            fragment,
        }
    }

    pub fn from_path(path: impl AsRef<str>) -> Self {
        Self {
            path: normalize_path(path.as_ref()),
            ..Default::default()
        }
    }

    pub fn to_url(&self) -> String {
        let canonical = self.canonicalized();

        let mut url = canonical.path;
        url.push_str(format_query_pairs(canonical.query.as_slice()).as_str());

        if let Some(fragment) = canonical
            .fragment
            .as_deref()
            .filter(|fragment| !fragment.is_empty())
        {
            if fragment.starts_with('#') {
                url.push_str(fragment);
            } else {
                url.push('#');
                url.push_str(fragment);
            }
        }

        url
    }

    pub fn query_value(&self, key: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|pair| pair.key == key)
            .and_then(|pair| pair.value.as_deref())
    }

    pub fn query_values(&self, key: &str) -> Vec<Option<String>> {
        query_values(format_query_pairs(self.query.as_slice()).as_str(), key)
    }

    pub fn set_query_value(&mut self, key: impl Into<String>, value: Option<String>) {
        let key = key.into();
        self.query.retain(|pair| pair.key != key);

        self.query.push(QueryPair { key, value });
        canonicalize_query_pairs(&mut self.query);
    }

    pub fn canonicalize_query(&mut self) {
        canonicalize_query_pairs(&mut self.query);
    }

    pub fn canonicalize(&mut self) {
        self.path = normalize_path(self.path.as_str());
        canonicalize_query_pairs(&mut self.query);
        self.fragment = canonicalize_fragment(self.fragment.take());
    }

    pub fn canonicalized(&self) -> Self {
        let mut location = self.clone();
        location.canonicalize();
        location
    }

    pub fn with_query_value(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        self.set_query_value(key, value);
        self
    }

    pub fn from_pattern(pattern: &PathPattern, params: &[PathParam]) -> Option<Self> {
        let path = pattern.format_path(params)?;
        Some(Self::from_path(path))
    }

    pub fn with_base_path(&self, base_path: &str) -> Self {
        let mut location = self.canonicalized();
        location.path = apply_base_path(location.path.as_str(), base_path);
        location
    }

    pub fn strip_base_path(&self, base_path: &str) -> Option<Self> {
        let mut location = self.canonicalized();
        location.path = strip_base_path(location.path.as_str(), base_path)?;
        Some(location)
    }
}

pub fn canonicalize_query_pairs(pairs: &mut Vec<QueryPair>) {
    let mut normalized = pairs
        .iter()
        .filter_map(|pair| {
            let key = pair.key.trim();
            if key.is_empty() {
                return None;
            }

            let value = pair
                .value
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            Some(QueryPair {
                key: key.to_string(),
                value,
            })
        })
        .collect::<Vec<_>>();

    normalized.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.value.cmp(&right.value))
    });

    *pairs = normalized;
}

fn canonicalize_fragment(fragment: Option<String>) -> Option<String> {
    let fragment = fragment?;
    let fragment = fragment.trim().trim_start_matches('#');
    if fragment.is_empty() {
        None
    } else {
        Some(encode_component(fragment))
    }
}

#[cfg(test)]
mod tests {
    use crate::path::{PathParam, PathPattern};

    use super::{QueryPair, RouteLocation};

    #[test]
    fn route_location_parse_and_to_url_roundtrip() {
        let location = RouteLocation::parse("/users/42?tab=profile&lang=zh#section-1");

        assert_eq!(location.path, "/users/42");
        assert_eq!(location.query_value("tab"), Some("profile"));
        assert_eq!(location.fragment.as_deref(), Some("section-1"));
        assert_eq!(location.to_url(), "/users/42?lang=zh&tab=profile#section-1");
    }

    #[test]
    fn route_location_decodes_and_reencodes_fragment() {
        let location = RouteLocation::parse("/docs#section%201");

        assert_eq!(location.fragment.as_deref(), Some("section 1"));
        assert_eq!(location.to_url(), "/docs#section%201");
    }

    #[test]
    fn route_location_to_url_is_canonical() {
        let location = RouteLocation::parse("users///42/?b=2&a=1&empty=# section 1 ");

        assert_eq!(location.to_url(), "/users/42?a=1&b=2&empty#section%201");
    }

    #[test]
    fn route_location_canonicalizes_query_order() {
        let mut location = RouteLocation {
            path: "/".to_string(),
            query: vec![
                QueryPair {
                    key: "z".to_string(),
                    value: Some("9".to_string()),
                },
                QueryPair {
                    key: "a".to_string(),
                    value: Some("1".to_string()),
                },
            ],
            fragment: None,
        };

        location.canonicalize_query();
        assert_eq!(location.to_url(), "/?a=1&z=9");
    }

    #[test]
    fn route_location_builds_from_path_pattern() {
        let pattern = PathPattern::parse("/docs/:lang/:slug").expect("pattern should parse");
        let location = RouteLocation::from_pattern(
            &pattern,
            &[
                PathParam {
                    name: "lang".to_string(),
                    value: "en".to_string(),
                },
                PathParam {
                    name: "slug".to_string(),
                    value: "getting-started".to_string(),
                },
            ],
        )
        .expect("location should build");

        assert_eq!(location.to_url(), "/docs/en/getting-started");
    }

    #[test]
    fn route_location_query_values_keep_duplicates() {
        let location = RouteLocation::parse("/search?tag=rust&tag=wasm&tag");
        assert_eq!(
            location.query_values("tag"),
            vec![Some("rust".to_string()), Some("wasm".to_string()), None]
        );
    }

    #[test]
    fn canonicalize_query_pairs_drops_empty_keys_and_values() {
        let mut pairs = vec![
            QueryPair {
                key: " ".to_string(),
                value: Some("1".to_string()),
            },
            QueryPair {
                key: "a".to_string(),
                value: Some(" ".to_string()),
            },
            QueryPair {
                key: "b".to_string(),
                value: Some("2".to_string()),
            },
        ];

        super::canonicalize_query_pairs(&mut pairs);

        assert_eq!(
            pairs,
            vec![
                QueryPair {
                    key: "a".to_string(),
                    value: None,
                },
                QueryPair {
                    key: "b".to_string(),
                    value: Some("2".to_string()),
                }
            ]
        );
    }

    #[test]
    fn route_location_can_apply_and_strip_base_path() {
        let location = RouteLocation::parse("/users/42?tab=profile#section-1");
        let with_base = location.with_base_path("/app");

        assert_eq!(with_base.to_url(), "/app/users/42?tab=profile#section-1");

        let stripped = with_base
            .strip_base_path("/app")
            .expect("base path should strip");
        assert_eq!(stripped.to_url(), "/users/42?tab=profile#section-1");
    }

    #[test]
    fn route_location_strip_base_path_returns_none_on_mismatch() {
        let location = RouteLocation::parse("/other/users/42");
        assert!(location.strip_base_path("/app").is_none());
    }
}
