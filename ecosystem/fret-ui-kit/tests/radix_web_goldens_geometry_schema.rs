use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct TimelineGolden {
    version: u32,
    base: String,
    theme: String,
    #[serde(default)]
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
struct Step {
    snapshot: Snapshot,
}

#[derive(Debug, Clone, Deserialize)]
struct Snapshot {
    dom: DomNode,
}

#[derive(Debug, Clone, Deserialize)]
struct DomRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct DomNode {
    tag: String,
    #[allow(dead_code)]
    path: Vec<usize>,
    #[allow(dead_code)]
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    rect: Option<DomRect>,
    #[serde(default)]
    children: Vec<DomNode>,
}

impl DomNode {
    fn validate_optional_rects(&self) {
        if let Some(r) = &self.rect {
            assert!(r.x.is_finite(), "rect.x for {}", self.tag);
            assert!(r.y.is_finite(), "rect.y for {}", self.tag);
            assert!(r.w.is_finite(), "rect.w for {}", self.tag);
            assert!(r.h.is_finite(), "rect.h for {}", self.tag);
            assert!(r.w >= 0.0, "rect.w >= 0 for {}", self.tag);
            assert!(r.h >= 0.0, "rect.h >= 0 for {}", self.tag);
        }
        for child in &self.children {
            child.validate_optional_rects();
        }
    }
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn radix_web_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("radix-web")
        .join("v4")
        .join("radix-vega")
}

#[test]
fn radix_web_goldens_optional_geometry_schema() {
    let dir = radix_web_dir();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|err| {
        panic!(
            "missing radix web golden dir: {}\nerror: {err}",
            dir.display()
        )
    });

    let mut count = 0usize;
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        count += 1;

        let text = std::fs::read_to_string(&path).expect("read golden file");
        let golden: TimelineGolden = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        assert!(golden.version >= 1, "version");
        assert_eq!(golden.base, "radix");
        assert_eq!(golden.theme, "light");
        assert!(!golden.steps.is_empty(), "steps");

        // Rects are optional so existing checked-in goldens remain valid; when present, they must
        // have sane numeric values so downstream layout conformance tests can rely on them.
        golden.steps[0].snapshot.dom.validate_optional_rects();
    }

    assert!(count >= 10, "expected some radix web goldens");
}
