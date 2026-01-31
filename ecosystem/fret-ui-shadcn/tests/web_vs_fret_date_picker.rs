use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
    #[serde(default, rename = "portalWrappers")]
    portal_wrappers: Vec<WebNode>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_path(name: &str) -> PathBuf {
    let base = repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"));

    if base.exists() {
        return base;
    }

    // Open-mode goldens use an `.open` suffix, but coverage keys normalize it away.
    // Fall back so tests can reference normalized keys (e.g. `foo.bar`) while still loading
    // `foo.bar.open.json`.
    base.with_file_name(format!("{name}.open.json"))
}

fn read_web_golden(name: &str) -> WebGolden {
    let path = web_golden_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 golden:extract {name} --update\n\nDocs:\n  goldens/README.md\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn find_first<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Option<&'a WebNode> {
    if pred(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first(child, pred) {
            return Some(found);
        }
    }
    None
}

fn class_contains(node: &WebNode, needle: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.contains(needle))
}

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

fn theme_find_first<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Option<&'a WebNode> {
    find_first(&theme.root, pred)
        .or_else(|| theme.portals.iter().find_map(|n| find_first(n, pred)))
        .or_else(|| {
            theme
                .portal_wrappers
                .iter()
                .find_map(|n| find_first(n, pred))
        })
}

const DATE_PICKER_KEYS: &[&str] = &[
    "date-picker-demo",
    "date-picker-with-presets",
    "date-picker-with-presets.preset-tomorrow",
    "date-picker-with-range",
];

#[test]
fn shadcn_date_picker_goldens_are_targeted_gates() {
    for &key in DATE_PICKER_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_contains(n, "lucide-calendar"))
            .expect("missing calendar icon");
        find_first(&theme.root, &|n| {
            class_contains(n, "justify-start") && class_contains(n, "text-left")
        })
        .expect("missing date picker trigger button recipe markers");
    }
}

#[test]
fn shadcn_date_picker_with_presets_preset_tomorrow_has_selected_day_and_trigger_text() {
    let web = read_web_golden("date-picker-with-presets.preset-tomorrow");
    let theme = web.themes.get("light").expect("missing light theme");

    theme_find_first(theme, &|n| {
        n.tag == "button" && contains_text(n, "January 16th, 2026")
    })
    .expect("missing updated trigger button text");

    theme_find_first(theme, &|n| {
        n.attrs
            .get("aria-label")
            .is_some_and(|v| v.contains("January 16th, 2026") && v.contains("selected"))
    })
    .expect("missing selected day aria-label marker");
}
