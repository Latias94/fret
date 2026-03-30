use fret_ui_kit::{Radius, Space};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct SpecCase {
    version: u32,
    name: String,
    classes: Vec<String>,
    expected: Expected,
}

#[derive(Debug, Clone, Deserialize)]
struct Expected {
    #[serde(default)]
    padding: Option<ExpectedEdges>,
    #[serde(default)]
    margin: Option<ExpectedEdges>,
    #[serde(default)]
    gap: Option<String>,
    #[serde(default)]
    radius: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedEdges {
    top: String,
    right: String,
    bottom: String,
    left: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EdgesOpt<T> {
    top: Option<T>,
    right: Option<T>,
    bottom: Option<T>,
    left: Option<T>,
}

impl<T> Default for EdgesOpt<T> {
    fn default() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct LayoutTokens {
    padding: EdgesOpt<Space>,
    margin: EdgesOpt<Space>,
    gap: Option<Space>,
    radius: Option<Radius>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn spec_dir() -> PathBuf {
    repo_root().join("goldens").join("tailwind-spec").join("v1")
}

fn parse_space(s: &str) -> Option<Space> {
    match s {
        "0" => Some(Space::N0),
        "0.5" => Some(Space::N0p5),
        "1" => Some(Space::N1),
        "1.5" => Some(Space::N1p5),
        "2" => Some(Space::N2),
        "2.5" => Some(Space::N2p5),
        "3" => Some(Space::N3),
        "3.5" => Some(Space::N3p5),
        "4" => Some(Space::N4),
        "5" => Some(Space::N5),
        "6" => Some(Space::N6),
        "8" => Some(Space::N8),
        "10" => Some(Space::N10),
        "11" => Some(Space::N11),
        "12" => Some(Space::N12),
        _ => None,
    }
}

fn space_to_str(space: Space) -> &'static str {
    match space {
        Space::N0 => "0",
        Space::N0p5 => "0.5",
        Space::N1 => "1",
        Space::N1p5 => "1.5",
        Space::N2 => "2",
        Space::N2p5 => "2.5",
        Space::N3 => "3",
        Space::N3p5 => "3.5",
        Space::N4 => "4",
        Space::N5 => "5",
        Space::N6 => "6",
        Space::N8 => "8",
        Space::N10 => "10",
        Space::N11 => "11",
        Space::N12 => "12",
    }
}

fn parse_radius(s: &str) -> Option<Radius> {
    match s {
        "sm" => Some(Radius::Sm),
        "md" => Some(Radius::Md),
        "lg" => Some(Radius::Lg),
        "full" => Some(Radius::Full),
        _ => None,
    }
}

fn radius_to_str(radius: Radius) -> &'static str {
    match radius {
        Radius::Sm => "sm",
        Radius::Md => "md",
        Radius::Lg => "lg",
        Radius::Full => "full",
    }
}

fn apply_edges_all(edges: &mut EdgesOpt<Space>, v: Space) {
    edges.top = Some(v);
    edges.right = Some(v);
    edges.bottom = Some(v);
    edges.left = Some(v);
}

fn parse_layout_tokens(classes: &[String]) -> LayoutTokens {
    let mut out = LayoutTokens::default();

    for class in classes {
        if let Some(v) = class.strip_prefix("p-").and_then(parse_space) {
            apply_edges_all(&mut out.padding, v);
            continue;
        }
        if let Some(v) = class.strip_prefix("px-").and_then(parse_space) {
            out.padding.left = Some(v);
            out.padding.right = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("py-").and_then(parse_space) {
            out.padding.top = Some(v);
            out.padding.bottom = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("pt-").and_then(parse_space) {
            out.padding.top = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("pr-").and_then(parse_space) {
            out.padding.right = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("pb-").and_then(parse_space) {
            out.padding.bottom = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("pl-").and_then(parse_space) {
            out.padding.left = Some(v);
            continue;
        }

        if let Some(v) = class.strip_prefix("m-").and_then(parse_space) {
            apply_edges_all(&mut out.margin, v);
            continue;
        }
        if let Some(v) = class.strip_prefix("mx-").and_then(parse_space) {
            out.margin.left = Some(v);
            out.margin.right = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("my-").and_then(parse_space) {
            out.margin.top = Some(v);
            out.margin.bottom = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("mt-").and_then(parse_space) {
            out.margin.top = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("mr-").and_then(parse_space) {
            out.margin.right = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("mb-").and_then(parse_space) {
            out.margin.bottom = Some(v);
            continue;
        }
        if let Some(v) = class.strip_prefix("ml-").and_then(parse_space) {
            out.margin.left = Some(v);
            continue;
        }

        if let Some(v) = class.strip_prefix("gap-").and_then(parse_space) {
            out.gap = Some(v);
            continue;
        }

        if let Some(v) = class.strip_prefix("rounded-").and_then(parse_radius) {
            out.radius = Some(v);
            continue;
        }
    }

    out
}

fn assert_edges_eq(actual: EdgesOpt<Space>, expected: &ExpectedEdges, label: &str) {
    let want_top = parse_space(&expected.top).unwrap_or_else(|| panic!("{label}.top"));
    let want_right = parse_space(&expected.right).unwrap_or_else(|| panic!("{label}.right"));
    let want_bottom = parse_space(&expected.bottom).unwrap_or_else(|| panic!("{label}.bottom"));
    let want_left = parse_space(&expected.left).unwrap_or_else(|| panic!("{label}.left"));

    assert_eq!(actual.top, Some(want_top), "{label}.top");
    assert_eq!(actual.right, Some(want_right), "{label}.right");
    assert_eq!(actual.bottom, Some(want_bottom), "{label}.bottom");
    assert_eq!(actual.left, Some(want_left), "{label}.left");
}

#[test]
fn tailwind_spec_v1_smoke() {
    let dir = spec_dir();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("missing tailwind-spec dir: {}\nerror: {err}", dir.display()));

    let mut count = 0usize;
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        count += 1;

        let text = std::fs::read_to_string(&path).expect("read spec file");
        let case: SpecCase = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        assert!(case.version >= 1);
        assert!(!case.name.is_empty());
        assert!(!case.classes.is_empty());

        let got = parse_layout_tokens(&case.classes);

        if let Some(padding) = &case.expected.padding {
            assert_edges_eq(got.padding, padding, "padding");
        } else {
            assert_eq!(got.padding, EdgesOpt::default());
        }

        if let Some(margin) = &case.expected.margin {
            assert_edges_eq(got.margin, margin, "margin");
        } else {
            assert_eq!(got.margin, EdgesOpt::default());
        }

        if let Some(gap) = &case.expected.gap {
            let want = parse_space(gap).unwrap_or_else(|| panic!("gap: invalid space {gap}"));
            assert_eq!(got.gap, Some(want), "gap");
            let _ = space_to_str(want);
        } else {
            assert_eq!(got.gap, None);
        }

        if let Some(radius) = &case.expected.radius {
            let want = parse_radius(radius).unwrap_or_else(|| panic!("radius: invalid {radius}"));
            assert_eq!(got.radius, Some(want), "radius");
            let _ = radius_to_str(want);
        } else {
            assert_eq!(got.radius, None);
        }
    }

    assert!(count >= 3, "expected at least 3 tailwind-spec cases");
}
