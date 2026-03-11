use crate::RouteLocation;

/// Shared typed route <-> canonical location conversion seam.
///
/// Router core still owns canonical URL policy and route matching. `RouteCodec` exists so apps and
/// higher-level ecosystem crates can centralize typed route translation instead of scattering
/// `RouteLocation` parsing/building logic.
pub trait RouteCodec: Send + Sync + 'static {
    type Route: Clone + Eq + 'static;
    type Error;

    /// Encode a typed route into a location.
    ///
    /// Implementations may return a non-canonical location; consumers should prefer
    /// `encode_canonical(...)` when they need stable href/location output.
    fn encode(&self, route: &Self::Route) -> RouteLocation;

    /// Decode a location into a typed route.
    ///
    /// Consumers should prefer `decode_canonical(...)` when they want canonical location semantics.
    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error>;

    fn encode_canonical(&self, route: &Self::Route) -> RouteLocation {
        self.encode(route).canonicalized()
    }

    fn decode_canonical(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
        self.decode(&location.canonicalized())
    }

    fn href_for(&self, route: &Self::Route) -> String {
        self.encode_canonical(route).to_url()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{PathPattern, RouteLocation};

    use super::RouteCodec;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum AppRoute {
        Home,
        User { id: Arc<str> },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DecodeError {
        NoMatch,
        MissingUserId,
    }

    struct TestCodec;

    impl RouteCodec for TestCodec {
        type Route = AppRoute;
        type Error = DecodeError;

        fn encode(&self, route: &Self::Route) -> RouteLocation {
            match route {
                AppRoute::Home => RouteLocation::parse("///"),
                AppRoute::User { id } => RouteLocation {
                    path: format!("/users/{id}"),
                    query: vec![
                        crate::QueryPair {
                            key: "b".to_string(),
                            value: Some("2".to_string()),
                        },
                        crate::QueryPair {
                            key: "a".to_string(),
                            value: Some("1".to_string()),
                        },
                    ],
                    fragment: Some(" section 1 ".to_string()),
                },
            }
        }

        fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
            if location.path == "/" {
                return Ok(AppRoute::Home);
            }

            let pattern = PathPattern::parse("/users/:id").expect("pattern should parse");
            let matched = pattern
                .match_path(location.path.as_str())
                .ok_or(DecodeError::NoMatch)?;
            let id = matched.param("id").ok_or(DecodeError::MissingUserId)?;
            Ok(AppRoute::User { id: Arc::from(id) })
        }
    }

    #[test]
    fn route_codec_encode_canonical_normalizes_output() {
        let codec = TestCodec;
        let location = codec.encode_canonical(&AppRoute::User {
            id: Arc::from("42"),
        });
        assert_eq!(location.to_url(), "/users/42?a=1&b=2#section%201");
    }

    #[test]
    fn route_codec_decode_canonical_normalizes_input_before_decode() {
        let codec = TestCodec;
        let route = codec
            .decode_canonical(&RouteLocation::parse("users///42/?b=2&a=1#ignored"))
            .expect("route should decode");
        assert_eq!(
            route,
            AppRoute::User {
                id: Arc::from("42")
            }
        );
    }

    #[test]
    fn route_codec_href_for_uses_canonical_location() {
        let codec = TestCodec;
        assert_eq!(codec.href_for(&AppRoute::Home), "/");
    }
}
