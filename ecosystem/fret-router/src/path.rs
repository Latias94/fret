use std::collections::{HashMap, HashSet};

use crate::query::{decode_component, encode_component};

pub const WILDCARD_PARAM: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    Static(String),
    Param(String),
    Wildcard(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathPattern {
    raw: String,
    segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPatternError {
    EmptyPattern,
    EmptyParamName,
    DuplicateParamName(String),
    WildcardMustBeLast,
    MultipleWildcards,
}

impl std::fmt::Display for PathPatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPattern => f.write_str("path pattern cannot be empty"),
            Self::EmptyParamName => f.write_str("path pattern contains empty param name"),
            Self::DuplicateParamName(name) => {
                write!(f, "path pattern contains duplicate param name '{name}'")
            }
            Self::WildcardMustBeLast => {
                f.write_str("path pattern wildcard must be the final segment")
            }
            Self::MultipleWildcards => f.write_str("path pattern can contain only one wildcard"),
        }
    }
}

impl std::error::Error for PathPatternError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathMatch {
    pub matched_path: String,
    pub params: Vec<PathParam>,
}

impl PathMatch {
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|param| param.name == name)
            .map(|param| param.value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry<R> {
    pub route: R,
    pub pattern: PathPattern,
}

#[derive(Debug, Clone)]
pub struct RouteTable<R> {
    entries: Vec<RouteEntry<R>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteResolution<'a, R> {
    pub route: &'a R,
    pub pattern: &'a PathPattern,
    pub params: Vec<PathParam>,
    pub matched_path: String,
    pub is_fallback: bool,
}

impl PathPattern {
    pub fn parse(pattern: &str) -> Result<Self, PathPatternError> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return Err(PathPatternError::EmptyPattern);
        }

        let raw = normalize_path(pattern);
        let raw_segments = path_segments(raw.as_str());
        let mut segments = Vec::new();
        let mut param_names = HashSet::<String>::new();
        let mut seen_wildcard = false;

        for (index, segment) in raw_segments.iter().enumerate() {
            if let Some(name) = segment.strip_prefix(':') {
                if name.is_empty() {
                    return Err(PathPatternError::EmptyParamName);
                }
                if !param_names.insert(name.to_string()) {
                    return Err(PathPatternError::DuplicateParamName(name.to_string()));
                }
                segments.push(PathSegment::Param(name.to_string()));
                continue;
            }

            if let Some(name) = segment.strip_prefix('*') {
                if seen_wildcard {
                    return Err(PathPatternError::MultipleWildcards);
                }
                if index + 1 != raw_segments.len() {
                    return Err(PathPatternError::WildcardMustBeLast);
                }
                seen_wildcard = true;

                let wildcard_name = if name.is_empty() {
                    None
                } else {
                    if !param_names.insert(name.to_string()) {
                        return Err(PathPatternError::DuplicateParamName(name.to_string()));
                    }
                    Some(name.to_string())
                };

                segments.push(PathSegment::Wildcard(wildcard_name));
                continue;
            }

            segments.push(PathSegment::Static(decode_component(segment)));
        }

        Ok(Self { raw, segments })
    }

    pub fn as_str(&self) -> &str {
        self.raw.as_str()
    }

    pub fn is_fallback(&self) -> bool {
        self.segments.len() == 1 && matches!(self.segments[0], PathSegment::Wildcard(_))
    }

    pub fn match_path(&self, path: &str) -> Option<PathMatch> {
        let normalized_path = normalize_path(path);
        let target = path_segments(normalized_path.as_str());

        let mut params = Vec::new();
        let mut cursor = 0usize;

        for segment in &self.segments {
            match segment {
                PathSegment::Static(expected) => {
                    let actual = target.get(cursor)?;
                    if decode_component(actual) != *expected {
                        return None;
                    }
                    cursor += 1;
                }
                PathSegment::Param(name) => {
                    let actual = target.get(cursor)?;
                    params.push(PathParam {
                        name: name.clone(),
                        value: decode_component(actual),
                    });
                    cursor += 1;
                }
                PathSegment::Wildcard(name) => {
                    let value = target[cursor..]
                        .iter()
                        .map(|segment| decode_component(segment))
                        .collect::<Vec<_>>()
                        .join("/");
                    params.push(PathParam {
                        name: name.clone().unwrap_or_else(|| WILDCARD_PARAM.to_string()),
                        value,
                    });
                    cursor = target.len();
                    break;
                }
            }
        }

        if cursor != target.len() {
            return None;
        }

        Some(PathMatch {
            matched_path: normalized_path,
            params,
        })
    }

    pub fn format_path(&self, params: &[PathParam]) -> Option<String> {
        let mut out = Vec::<String>::new();
        let by_name = params_by_name(params);

        for segment in &self.segments {
            match segment {
                PathSegment::Static(value) => out.push(encode_component(value)),
                PathSegment::Param(name) => {
                    let value = by_name.get(name.as_str())?;
                    if value.is_empty() {
                        return None;
                    }
                    out.push(encode_component(value));
                }
                PathSegment::Wildcard(name) => {
                    let key = name.as_deref().unwrap_or(WILDCARD_PARAM);
                    let value = by_name.get(key).copied().unwrap_or("");
                    for segment in value.split('/').filter(|segment| !segment.is_empty()) {
                        out.push(encode_component(segment));
                    }
                }
            }
        }

        if out.is_empty() {
            Some("/".to_string())
        } else {
            Some(format!("/{}", out.join("/")))
        }
    }
}

impl<R> RouteTable<R> {
    pub fn new(entries: Vec<RouteEntry<R>>) -> Self {
        Self { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn resolve(&self, path: &str) -> Option<RouteResolution<'_, R>> {
        let mut fallback: Option<(&RouteEntry<R>, PathMatch)> = None;

        for entry in &self.entries {
            let Some(matched) = entry.pattern.match_path(path) else {
                continue;
            };

            if entry.pattern.is_fallback() {
                if fallback.is_none() {
                    fallback = Some((entry, matched));
                }
                continue;
            }

            return Some(RouteResolution {
                route: &entry.route,
                pattern: &entry.pattern,
                params: matched.params,
                matched_path: matched.matched_path,
                is_fallback: false,
            });
        }

        let (entry, matched) = fallback?;
        Some(RouteResolution {
            route: &entry.route,
            pattern: &entry.pattern,
            params: matched.params,
            matched_path: matched.matched_path,
            is_fallback: true,
        })
    }
}

pub fn normalize_path(path: &str) -> String {
    let without_hash = path.trim().split('#').next().unwrap_or_default();
    let without_query = without_hash.split('?').next().unwrap_or_default();
    let segments = path_segments(without_query);

    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn params_by_name<'a>(params: &'a [PathParam]) -> HashMap<&'a str, &'a str> {
    let mut out = HashMap::new();
    for param in params {
        out.insert(param.name.as_str(), param.value.as_str());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        PathParam, PathPattern, PathPatternError, RouteEntry, RouteTable, WILDCARD_PARAM,
        normalize_path,
    };

    #[test]
    fn normalize_path_strips_query_hash_and_extra_slashes() {
        assert_eq!(
            normalize_path("users///42/?tab=profile#anchor"),
            "/users/42"
        );
        assert_eq!(normalize_path(""), "/");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn path_pattern_extracts_param_values() {
        let pattern = PathPattern::parse("/users/:id/settings").expect("pattern should parse");
        let matched = pattern
            .match_path("/users/42/settings")
            .expect("path should match");

        assert_eq!(matched.param("id"), Some("42"));
        assert_eq!(matched.matched_path, "/users/42/settings");
    }

    #[test]
    fn path_pattern_wildcard_captures_remaining_segments() {
        let pattern = PathPattern::parse("/docs/*rest").expect("pattern should parse");
        let matched = pattern
            .match_path("/docs/guides/getting-started")
            .expect("path should match");

        assert_eq!(matched.param("rest"), Some("guides/getting-started"));
    }

    #[test]
    fn route_table_uses_fallback_when_no_route_matches() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Home,
            User,
            NotFound,
        }

        let table = RouteTable::new(vec![
            RouteEntry {
                route: RouteId::Home,
                pattern: PathPattern::parse("/").expect("pattern should parse"),
            },
            RouteEntry {
                route: RouteId::User,
                pattern: PathPattern::parse("/users/:id").expect("pattern should parse"),
            },
            RouteEntry {
                route: RouteId::NotFound,
                pattern: PathPattern::parse("/*").expect("pattern should parse"),
            },
        ]);

        let matched_user = table.resolve("/users/7").expect("route should resolve");
        assert_eq!(*matched_user.route, RouteId::User);
        assert_eq!(matched_user.params[0].name, "id");
        assert_eq!(matched_user.params[0].value, "7");
        assert!(!matched_user.is_fallback);

        let matched_fallback = table
            .resolve("/unknown/feature")
            .expect("fallback should resolve");
        assert_eq!(*matched_fallback.route, RouteId::NotFound);
        assert_eq!(matched_fallback.params[0].name, WILDCARD_PARAM);
        assert_eq!(matched_fallback.params[0].value, "unknown/feature");
        assert!(matched_fallback.is_fallback);
    }

    #[test]
    fn path_pattern_format_roundtrip() {
        let pattern = PathPattern::parse("/users/:id/*rest").expect("pattern should parse");
        let path = pattern
            .format_path(&[
                PathParam {
                    name: "id".to_string(),
                    value: "42".to_string(),
                },
                PathParam {
                    name: "rest".to_string(),
                    value: "settings/profile".to_string(),
                },
            ])
            .expect("path should format");

        assert_eq!(path, "/users/42/settings/profile");
        let matched = pattern
            .match_path(path.as_str())
            .expect("path should match");
        assert_eq!(matched.param("id"), Some("42"));
        assert_eq!(matched.param("rest"), Some("settings/profile"));
    }

    #[test]
    fn path_pattern_rejects_invalid_wildcard_position() {
        let error = PathPattern::parse("/*rest/child").expect_err("pattern should fail");
        assert_eq!(error, PathPatternError::WildcardMustBeLast);
    }

    #[test]
    fn path_pattern_rejects_duplicate_param_names() {
        let error = PathPattern::parse("/users/:id/:id").expect_err("pattern should fail");
        assert_eq!(
            error,
            PathPatternError::DuplicateParamName("id".to_string())
        );
    }
}
