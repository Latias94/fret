use crate::path::PathSpecificity;
use crate::{
    PathParam, PathPattern, PathPatternError, RouteLocation, RouteSearchTable, SearchMap,
    SearchValidationError, SearchValidationMode,
};

#[derive(Debug)]
pub enum RouteTreeError {
    InvalidPathPattern {
        path: String,
        source: PathPatternError,
    },
}

impl std::fmt::Display for RouteTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPathPattern { path, source } => {
                write!(f, "invalid route path pattern '{path}': {source}")
            }
        }
    }
}

impl std::error::Error for RouteTreeError {}

#[derive(Debug, Clone)]
pub struct RouteNode<R> {
    pub route: R,
    pub pattern: PathPattern,
    pub children: Vec<RouteNode<R>>,
}

impl<R> RouteNode<R> {
    pub fn new(route: R, path: impl AsRef<str>) -> Result<Self, RouteTreeError> {
        let path = path.as_ref().trim();
        let path = if path.is_empty() { "/" } else { path };
        let pattern =
            PathPattern::parse(path).map_err(|source| RouteTreeError::InvalidPathPattern {
                path: path.to_string(),
                source,
            })?;

        Ok(Self {
            route,
            pattern,
            children: Vec::new(),
        })
    }

    pub fn with_children(mut self, children: Vec<RouteNode<R>>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RouteTree<R> {
    pub root: RouteNode<R>,
}

impl<R> RouteTree<R> {
    pub fn new(root: RouteNode<R>) -> Self {
        Self { root }
    }

    pub fn match_routes<'a>(&'a self, location: &RouteLocation) -> RouteMatchResult<'a, R> {
        let canonical = location.canonicalized();
        let segments = split_segments_owned(canonical.path.as_str());
        let segment_refs = segments
            .iter()
            .map(|segment| segment.as_str())
            .collect::<Vec<_>>();

        let Some(root_prefix) = self
            .root
            .pattern
            .match_prefix_segments(segment_refs.as_slice())
        else {
            return RouteMatchResult {
                location: canonical,
                matches: Vec::new(),
                is_not_found: true,
            };
        };

        let root_matched_path =
            extend_matched_path("/", &segment_refs[..root_prefix.consumed_segments]);
        let remaining_segments = &segment_refs[root_prefix.consumed_segments..];

        let root_match = RouteMatch {
            route: &self.root.route,
            pattern: &self.root.pattern,
            matched_path: root_matched_path,
            params: root_prefix.params.clone(),
        };

        let mut root_rank = RouteRank::default();
        root_rank.add_specificity(self.root.pattern.specificity());
        root_rank.depth = 1;

        let base = PartialMatch {
            chain: vec![root_match.clone()],
            params: root_prefix.params,
            remaining_segments,
            rank: root_rank,
        };

        let best = best_match(&self.root, base);
        let is_not_found = if best.is_full { best.is_fallback } else { true };

        RouteMatchResult {
            location: canonical,
            matches: best.chain,
            is_not_found,
        }
    }

    pub fn diagnostics(&self) -> RouteTreeDiagnostics {
        let mut out = RouteTreeDiagnostics::default();

        let mut leaves = Vec::<LeafSignature>::new();
        collect_leaf_signatures(&self.root, &mut Vec::new(), &mut Vec::new(), &mut leaves);

        leaves.sort_by(|a, b| a.shape_key.cmp(&b.shape_key));

        let mut cursor = 0usize;
        while cursor < leaves.len() {
            let key = leaves[cursor].shape_key.clone();
            let start = cursor;
            cursor += 1;
            while cursor < leaves.len() && leaves[cursor].shape_key == key {
                cursor += 1;
            }

            if cursor - start > 1 {
                out.ambiguities.push(RouteAmbiguity {
                    shape_key: key,
                    patterns: leaves[start..cursor]
                        .iter()
                        .map(|leaf| leaf.pattern.clone())
                        .collect(),
                });
            }
        }

        out
    }

    pub fn match_routes_with_search<'a>(
        &'a self,
        location: &RouteLocation,
        search_table: &RouteSearchTable<R>,
        mode: SearchValidationMode,
    ) -> Result<RouteMatchResultWithSearch<'a, R>, RouteSearchValidationFailure>
    where
        R: std::hash::Hash + Eq,
    {
        let base = self.match_routes(location);

        let canonical_location = base.location.canonicalized();
        let mut accumulated = SearchMap::from_location(&canonical_location);

        let mut matches = Vec::with_capacity(base.matches.len());
        for (index, entry) in base.matches.into_iter().enumerate() {
            let mut error: Option<SearchValidationError> = None;

            if let Some(validator) = search_table.validator_for(entry.route) {
                match validator(&canonical_location, &accumulated) {
                    Ok(next) => {
                        accumulated = next;
                    }
                    Err(err) => {
                        if mode == SearchValidationMode::Strict {
                            return Err(RouteSearchValidationFailure {
                                match_index: index,
                                matched_path: entry.matched_path,
                                error: err,
                            });
                        }
                        error = Some(err);
                    }
                }
            }

            matches.push(RouteMatchWithSearch {
                route: entry.route,
                pattern: entry.pattern,
                matched_path: entry.matched_path,
                params: entry.params,
                search: accumulated.clone(),
                search_error: error,
            });
        }

        Ok(RouteMatchResultWithSearch {
            location: canonical_location,
            matches,
            is_not_found: base.is_not_found,
        })
    }
}

#[derive(Debug)]
pub struct RouteMatch<'a, R> {
    pub route: &'a R,
    pub pattern: &'a PathPattern,
    pub matched_path: String,
    pub params: Vec<PathParam>,
}

impl<R> Clone for RouteMatch<'_, R> {
    fn clone(&self) -> Self {
        Self {
            route: self.route,
            pattern: self.pattern,
            matched_path: self.matched_path.clone(),
            params: self.params.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteMatchResult<'a, R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatch<'a, R>>,
    pub is_not_found: bool,
}

#[derive(Debug, Clone)]
pub struct RouteMatchWithSearch<'a, R> {
    pub route: &'a R,
    pub pattern: &'a PathPattern,
    pub matched_path: String,
    pub params: Vec<PathParam>,
    pub search: SearchMap,
    pub search_error: Option<SearchValidationError>,
}

#[derive(Debug, Clone)]
pub struct RouteMatchResultWithSearch<'a, R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatchWithSearch<'a, R>>,
    pub is_not_found: bool,
}

#[derive(Debug, Clone)]
pub struct RouteSearchValidationFailure {
    pub match_index: usize,
    pub matched_path: String,
    pub error: SearchValidationError,
}

impl std::fmt::Display for RouteSearchValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "search validation failed at match {} ('{}'): {}",
            self.match_index, self.matched_path, self.error
        )
    }
}

impl std::error::Error for RouteSearchValidationFailure {}

#[derive(Debug, Default, Clone)]
pub struct RouteTreeDiagnostics {
    pub ambiguities: Vec<RouteAmbiguity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteAmbiguity {
    pub shape_key: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone)]
struct PartialMatch<'r, 's, R> {
    chain: Vec<RouteMatch<'r, R>>,
    params: Vec<PathParam>,
    remaining_segments: &'s [&'s str],
    rank: RouteRank,
}

#[derive(Debug, Clone)]
struct BestMatch<'r, R> {
    chain: Vec<RouteMatch<'r, R>>,
    rank: RouteRank,
    is_full: bool,
    is_fallback: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct RouteRank {
    static_segments: usize,
    param_segments: usize,
    wildcard_segments: usize,
    total_segments: usize,
    depth: usize,
}

impl RouteRank {
    fn add_specificity(&mut self, specificity: PathSpecificity) {
        self.static_segments += specificity.static_segments;
        self.param_segments += specificity.param_segments;
        self.wildcard_segments += specificity.wildcard_segments;
        self.total_segments += specificity.total_segments;
    }
}

impl Ord for RouteRank {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.static_segments
            .cmp(&other.static_segments)
            .then_with(|| self.param_segments.cmp(&other.param_segments))
            .then_with(|| other.wildcard_segments.cmp(&self.wildcard_segments))
            .then_with(|| self.total_segments.cmp(&other.total_segments))
            .then_with(|| self.depth.cmp(&other.depth))
    }
}

impl PartialOrd for RouteRank {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn best_match<'r, 's, R>(
    node: &'r RouteNode<R>,
    base: PartialMatch<'r, 's, R>,
) -> BestMatch<'r, R> {
    let mut best = BestMatch {
        chain: base.chain.clone(),
        rank: base.rank,
        is_full: base.remaining_segments.is_empty(),
        is_fallback: base.remaining_segments.is_empty() && node.pattern.is_fallback(),
    };

    for child in &node.children {
        let Some(prefix) = child.pattern.match_prefix_segments(base.remaining_segments) else {
            continue;
        };

        let consumed = prefix.consumed_segments;
        let next_remaining = &base.remaining_segments[consumed..];

        let mut next_params = base.params.clone();
        next_params.extend(prefix.params);

        let mut next_rank = base.rank;
        next_rank.add_specificity(child.pattern.specificity());
        next_rank.depth = base.chain.len() + 1;

        let matched_path = extend_matched_path(
            base.chain
                .last()
                .map(|entry| entry.matched_path.as_str())
                .unwrap_or("/"),
            &base.remaining_segments[..consumed],
        );

        let mut next_chain = base.chain.clone();
        next_chain.push(RouteMatch {
            route: &child.route,
            pattern: &child.pattern,
            matched_path,
            params: next_params.clone(),
        });

        let next_base = PartialMatch {
            chain: next_chain,
            params: next_params,
            remaining_segments: next_remaining,
            rank: next_rank,
        };

        let child_best = best_match(child, next_base);
        best = pick_best(best, child_best);
    }

    best
}

fn pick_best<'r, R>(left: BestMatch<'r, R>, right: BestMatch<'r, R>) -> BestMatch<'r, R> {
    if left.is_full != right.is_full {
        if left.is_full { left } else { right }
    } else if right.rank > left.rank {
        right
    } else {
        left
    }
}

fn split_segments_owned(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn extend_matched_path(parent: &str, segments: &[&str]) -> String {
    if segments.is_empty() {
        parent.to_string()
    } else {
        let suffix = segments.join("/");
        if parent == "/" {
            format!("/{suffix}")
        } else {
            format!("{parent}/{suffix}")
        }
    }
}

#[derive(Debug, Clone)]
struct LeafSignature {
    shape_key: String,
    pattern: String,
}

fn collect_leaf_signatures<R>(
    node: &RouteNode<R>,
    shape_prefix: &mut Vec<String>,
    raw_prefix: &mut Vec<String>,
    out: &mut Vec<LeafSignature>,
) {
    let raw_segments = node
        .pattern
        .as_str()
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let shape_segments = raw_segments
        .iter()
        .map(|segment| shape_segment(segment.as_str()))
        .collect::<Vec<_>>();

    let shape_start_len = shape_prefix.len();
    let raw_start_len = raw_prefix.len();
    shape_prefix.extend(shape_segments);
    raw_prefix.extend(raw_segments);

    if node.children.is_empty() {
        let shape_key = if shape_prefix.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", shape_prefix.join("/"))
        };
        let full_pattern = if raw_prefix.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", raw_prefix.join("/"))
        };
        out.push(LeafSignature {
            shape_key,
            pattern: full_pattern,
        });
    } else {
        for child in &node.children {
            collect_leaf_signatures(child, shape_prefix, raw_prefix, out);
        }
    }

    shape_prefix.truncate(shape_start_len);
    raw_prefix.truncate(raw_start_len);
}

fn shape_segment(segment: &str) -> String {
    if segment.starts_with(':') {
        ":".to_string()
    } else if segment.starts_with('*') {
        "*".to_string()
    } else {
        segment.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{RouteNode, RouteTree};
    use crate::{
        RouteLocation, RouteSearchTable, SearchMap, SearchValidationError, SearchValidationMode,
    };

    #[test]
    fn nested_matches_return_chain_with_accumulated_params() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
            Users,
            User,
            Settings,
        }

        let tree = RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap().with_children(
            vec![RouteNode::new(RouteId::Users, "users")
                    .unwrap()
                    .with_children(vec![RouteNode::new(RouteId::User, ":id")
                        .unwrap()
                        .with_children(vec![RouteNode::new(RouteId::Settings, "settings")
                            .unwrap()
                            .with_children(vec![])])])],
        ));

        let result = tree.match_routes(&RouteLocation::parse("/users/42/settings"));
        assert!(!result.is_not_found);
        assert_eq!(result.matches.len(), 4);
        assert_eq!(*result.matches[0].route, RouteId::Root);
        assert_eq!(result.matches[0].matched_path, "/");
        assert_eq!(*result.matches[1].route, RouteId::Users);
        assert_eq!(result.matches[1].matched_path, "/users");
        assert_eq!(*result.matches[2].route, RouteId::User);
        assert_eq!(result.matches[2].matched_path, "/users/42");
        assert_eq!(result.matches[2].params[0].name, "id");
        assert_eq!(result.matches[2].params[0].value, "42");
        assert_eq!(*result.matches[3].route, RouteId::Settings);
        assert_eq!(result.matches[3].matched_path, "/users/42/settings");
    }

    #[test]
    fn index_routes_are_preferred_over_shallower_matches() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
            Home,
        }

        let tree = RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Home, "/").unwrap()]),
        );

        let result = tree.match_routes(&RouteLocation::parse("/"));
        assert!(!result.is_not_found);
        assert_eq!(result.matches.len(), 2);
        assert_eq!(*result.matches[1].route, RouteId::Home);
    }

    #[test]
    fn unknown_paths_return_root_only_when_no_fallback_exists() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
        }

        let tree = RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap());
        let result = tree.match_routes(&RouteLocation::parse("/unknown"));

        assert!(result.is_not_found);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(*result.matches[0].route, RouteId::Root);
    }

    #[test]
    fn fallback_route_is_selected_when_present() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
            NotFound,
        }

        let tree = RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::NotFound, "/*").unwrap()]),
        );

        let result = tree.match_routes(&RouteLocation::parse("/unknown/feature"));
        assert!(result.is_not_found);
        assert_eq!(result.matches.len(), 2);
        assert_eq!(*result.matches[1].route, RouteId::NotFound);
    }

    #[test]
    fn specificity_prefers_static_over_param_siblings() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
            Users,
            UserById,
            UsersSettings,
        }

        let tree = RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap().with_children(
            vec![RouteNode::new(RouteId::Users, "users")
                    .unwrap()
                    .with_children(vec![
                        RouteNode::new(RouteId::UserById, ":id").unwrap(),
                        RouteNode::new(RouteId::UsersSettings, "settings").unwrap(),
                    ])],
        ));

        let result = tree.match_routes(&RouteLocation::parse("/users/settings"));
        assert!(!result.is_not_found);
        assert_eq!(
            *result.matches.last().unwrap().route,
            RouteId::UsersSettings
        );
    }

    #[test]
    fn diagnostics_detects_param_name_collisions() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum RouteId {
            Root,
            Users,
            ById,
            ByName,
        }

        let tree = RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap().with_children(
            vec![RouteNode::new(RouteId::Users, "users")
                    .unwrap()
                    .with_children(vec![
                        RouteNode::new(RouteId::ById, ":id").unwrap(),
                        RouteNode::new(RouteId::ByName, ":name").unwrap(),
                    ])],
        ));

        let diagnostics = tree.diagnostics();
        assert_eq!(diagnostics.ambiguities.len(), 1);
        assert_eq!(diagnostics.ambiguities[0].shape_key, "/users/:");
    }

    #[test]
    fn search_validation_accumulates_through_match_chain() {
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

        fn validate_gallery(
            _location: &RouteLocation,
            search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            let lang = search.first("lang").unwrap_or("en");
            Ok(search
                .clone()
                .with("title", Some(format!("gallery:{lang}"))))
        }

        let tree = RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Gallery, "gallery").unwrap()]),
        );

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        search_table.insert(RouteId::Gallery, validate_gallery);

        let result = tree
            .match_routes_with_search(
                &RouteLocation::parse("/gallery?lang=zh"),
                &search_table,
                SearchValidationMode::Strict,
            )
            .expect("validation should succeed");

        assert_eq!(result.matches.len(), 2);
        assert_eq!(result.matches[0].search.first("lang"), Some("en"));
        assert_eq!(result.matches[1].search.first("title"), Some("gallery:en"));
    }

    #[test]
    fn search_validation_lenient_mode_records_error_and_continues() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Child,
        }

        fn bad_validate(
            _location: &RouteLocation,
            _search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            Err(SearchValidationError::new("bad search"))
        }

        fn ok_validate(
            _location: &RouteLocation,
            search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            Ok(search.clone().with("ok", Some("1".to_string())))
        }

        let tree = RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Child, "child").unwrap()]),
        );

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, bad_validate);
        search_table.insert(RouteId::Child, ok_validate);

        let result = tree
            .match_routes_with_search(
                &RouteLocation::parse("/child"),
                &search_table,
                SearchValidationMode::Lenient,
            )
            .expect("lenient mode should not fail");

        assert!(result.matches[0].search_error.is_some());
        assert!(result.matches[1].search_error.is_none());
        assert_eq!(result.matches[1].search.first("ok"), Some("1"));
    }

    #[test]
    fn search_validation_strict_mode_fails_fast() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
        }

        fn bad_validate(
            _location: &RouteLocation,
            _search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            Err(SearchValidationError::new("bad search"))
        }

        let tree = RouteTree::new(RouteNode::new(RouteId::Root, "/").unwrap());

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, bad_validate);

        let err = tree
            .match_routes_with_search(
                &RouteLocation::parse("/"),
                &search_table,
                SearchValidationMode::Strict,
            )
            .expect_err("strict mode should fail");

        assert_eq!(err.match_index, 0);
        assert_eq!(err.error.message(), "bad search");
    }
}
