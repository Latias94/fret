use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[path = "support/repo_root.rs"]
mod repo_root;

use repo_root::repo_root;

#[derive(Debug, Clone, Deserialize)]
struct TimelineGolden {
    version: u32,
    base: String,
    theme: String,
    item: String,
    primitive: String,
    scenario: String,
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

#[derive(Debug, Clone, Copy, Deserialize)]
struct DomRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DomNode {
    tag: String,
    path: Vec<usize>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: Option<DomRect>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<DomNode>,
}

fn radix_web_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("radix-web")
        .join("v4")
        .join("radix-vega")
}

fn assert_dom_rect(label: &str, rect: DomRect) {
    assert!(
        rect.x.is_finite()
            && rect.y.is_finite()
            && rect.w.is_finite()
            && rect.h.is_finite()
            && rect.w >= 0.0
            && rect.h >= 0.0,
        "invalid rect for {label}: {rect:?}"
    );
}

fn validate_rects_in_dom_tree(name: &str, node: &DomNode) -> usize {
    let mut count = 0usize;
    if let Some(rect) = node.rect {
        assert_dom_rect(name, rect);
        count += 1;
    }
    for child in &node.children {
        count += validate_rects_in_dom_tree(name, child);
    }
    count
}

fn requires_rects(primitive: &str) -> bool {
    matches!(
        primitive,
        "alert-dialog"
            | "context-menu"
            | "dialog"
            | "dropdown-menu"
            | "hover-card"
            | "menubar"
            | "navigation-menu"
            | "popover"
            | "select"
            | "tooltip"
    )
}

#[test]
fn radix_web_goldens_smoke_parse_and_rects_present() {
    let dir = radix_web_dir();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|err| {
        panic!(
            "missing radix web goldens dir: {}\nerror: {err}",
            dir.display()
        )
    });

    let mut found = 0usize;
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!(
                "failed to read radix web golden: {}\nerror: {err}",
                path.display()
            )
        });
        let golden: TimelineGolden = serde_json::from_str(&text).unwrap_or_else(|err| {
            panic!(
                "failed to parse radix web golden: {}\nerror: {err}",
                path.display()
            )
        });

        assert!(golden.version >= 1, "golden.version: {name}");
        assert_eq!(golden.base, "radix", "golden.base: {name}");
        assert!(!golden.theme.is_empty(), "golden.theme: {name}");
        assert!(!golden.item.is_empty(), "golden.item: {name}");
        assert!(!golden.primitive.is_empty(), "golden.primitive: {name}");
        assert!(!golden.scenario.is_empty(), "golden.scenario: {name}");
        assert!(!golden.steps.is_empty(), "golden.steps: {name}");

        let mut rects_total = 0usize;
        for (idx, step) in golden.steps.iter().enumerate() {
            assert!(
                !step.snapshot.dom.tag.is_empty(),
                "step {idx} dom.tag: {name}"
            );
            rects_total += validate_rects_in_dom_tree(name, &step.snapshot.dom);
        }

        if requires_rects(&golden.primitive) {
            assert!(
                rects_total > 0,
                "expected at least one rect somewhere in {name} (primitive={})",
                golden.primitive
            );
        }

        found += 1;
    }

    assert!(
        found >= 10,
        "expected radix web goldens in {}",
        dir.display()
    );
}
