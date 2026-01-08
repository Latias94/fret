use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct TimelineGolden {
    version: u32,
    base: String,
    style: String,
    #[serde(rename = "baseColor")]
    base_color: String,
    theme: String,
    item: String,
    primitive: String,
    scenario: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
struct Step {
    #[allow(dead_code)]
    action: Action,
    snapshot: Snapshot,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
enum Action {
    #[allow(dead_code)]
    #[serde(rename = "load")]
    Load { url: String },
    #[allow(dead_code)]
    #[serde(rename = "click")]
    Click { target: String },
    #[allow(dead_code)]
    #[serde(rename = "press")]
    Press { key: String },
    #[allow(dead_code)]
    #[serde(rename = "hover")]
    Hover { target: String },
}

#[derive(Debug, Clone, Deserialize)]
struct Snapshot {
    #[allow(dead_code)]
    focus: Option<DomFocus>,
    dom: DomNode,
    #[allow(dead_code)]
    ax: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomFocus {
    #[allow(dead_code)]
    tag: String,
    #[allow(dead_code)]
    path: Vec<usize>,
    #[allow(dead_code)]
    attrs: BTreeMap<String, String>,
    #[allow(dead_code)]
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomNode {
    tag: String,
    path: Vec<usize>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<DomNode>,
}

impl DomNode {
    fn contains_role(&self, role: &str) -> bool {
        if self.attrs.get("role").is_some_and(|v| v == role) {
            return true;
        }
        self.children.iter().any(|c| c.contains_role(role))
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

fn read_timeline(file_stem: &str) -> TimelineGolden {
    let path = radix_web_dir().join(format!("{file_stem}.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing radix web golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/radix-web/scripts/extract-behavior.mts --all --update --baseUrl=http://localhost:4020\n\nDocs:\n  goldens/radix-web/README.md",
            path.display()
        )
    });
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse radix web golden: {}\nerror: {err}", path.display()))
}

#[test]
fn radix_web_goldens_smoke() {
    let dir = radix_web_dir();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("missing radix web golden dir: {}\nerror: {err}", dir.display()));

    let mut count = 0usize;
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        count += 1;

        let text = std::fs::read_to_string(&path).expect("read golden file");
        let golden: TimelineGolden =
            serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));

        assert!(golden.version >= 1, "version");
        assert_eq!(golden.base, "radix");
        assert_eq!(golden.theme, "light");
        assert!(!golden.style.is_empty());
        assert!(!golden.base_color.is_empty());
        assert!(!golden.item.is_empty());
        assert!(!golden.primitive.is_empty());
        assert!(!golden.scenario.is_empty());
        assert!(!golden.steps.is_empty());
        assert!(!golden.steps[0].snapshot.dom.tag.is_empty());
        let _ = &golden.steps[0].snapshot.dom.path;
        let _ = &golden.steps[0].snapshot.dom.text;
    }

    assert!(
        count >= 10,
        "expected at least 10 radix web goldens under {}",
        dir.display()
    );
}

#[test]
fn radix_web_dialog_open_close_contract() {
    let golden = read_timeline("dialog-example.dialog.open-close.light");
    assert_eq!(golden.item, "dialog-example");
    assert_eq!(golden.primitive, "dialog");
    assert_eq!(golden.scenario, "open-close");
    assert!(golden.steps.len() >= 3);

    assert!(!golden.steps[0].snapshot.dom.contains_role("dialog"));
    assert!(golden.steps[1].snapshot.dom.contains_role("dialog"));
    assert!(!golden.steps[2].snapshot.dom.contains_role("dialog"));
}

#[test]
fn radix_web_select_open_close_contract() {
    let golden = read_timeline("select-example.select.open-navigate-select.light");
    assert_eq!(golden.item, "select-example");
    assert_eq!(golden.primitive, "select");
    assert_eq!(golden.scenario, "open-navigate-select");
    assert!(golden.steps.len() >= 3);

    assert!(!golden.steps[0].snapshot.dom.contains_role("listbox"));
    assert!(golden.steps[1].snapshot.dom.contains_role("listbox"));
    assert!(!golden.steps[2].snapshot.dom.contains_role("listbox"));
}
