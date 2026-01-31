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
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"))
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

fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|t| t == token))
}

fn class_has_all_tokens(node: &WebNode, tokens: &[&str]) -> bool {
    tokens.iter().all(|t| class_has_token(node, t))
}

fn web_find_chart_root<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "div" && n.attrs.get("data-slot").is_some_and(|v| v == "chart")
    })
}

fn web_find_chart_tooltip_panel<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    // Stable invariants of the upstream shadcn v4 ChartTooltipContent recipe.
    find_first(root, &|n| {
        n.tag == "div"
            && class_has_all_tokens(
                n,
                &[
                    "border-border/50",
                    "bg-background",
                    "shadow-xl",
                    "min-w-[8rem]",
                ],
            )
    })
}

const CHART_KEYS: &[&str] = &[
    "chart-area-axes",
    "chart-area-default",
    "chart-area-gradient",
    "chart-area-icons",
    "chart-area-interactive",
    "chart-area-interactive.30d",
    "chart-area-interactive.7d",
    "chart-area-legend",
    "chart-area-linear",
    "chart-area-stacked",
    "chart-area-stacked-expand",
    "chart-area-step",
    "chart-bar-active",
    "chart-bar-default",
    "chart-bar-demo",
    "chart-bar-demo-axis",
    "chart-bar-demo-grid",
    "chart-bar-demo-legend",
    "chart-bar-demo-tooltip",
    "chart-bar-horizontal",
    "chart-bar-interactive",
    "chart-bar-interactive.hover-mid",
    "chart-bar-interactive.mobile",
    "chart-bar-label",
    "chart-bar-label-custom",
    "chart-bar-mixed",
    "chart-bar-multiple",
    "chart-bar-negative",
    "chart-bar-stacked",
    "chart-line-default",
    "chart-line-dots",
    "chart-line-dots-colors",
    "chart-line-dots-custom",
    "chart-line-interactive",
    "chart-line-interactive.hover-mid",
    "chart-line-interactive.mobile",
    "chart-line-label",
    "chart-line-label-custom",
    "chart-line-linear",
    "chart-line-multiple",
    "chart-line-step",
    "chart-pie-donut",
    "chart-pie-donut-active",
    "chart-pie-donut-text",
    "chart-pie-interactive",
    "chart-pie-interactive.february",
    "chart-pie-interactive.may",
    "chart-pie-label",
    "chart-pie-label-custom",
    "chart-pie-label-list",
    "chart-pie-legend",
    "chart-pie-separator-none",
    "chart-pie-simple",
    "chart-pie-stacked",
    "chart-radar-default",
    "chart-radar-dots",
    "chart-radar-grid-circle",
    "chart-radar-grid-circle-fill",
    "chart-radar-grid-circle-no-lines",
    "chart-radar-grid-custom",
    "chart-radar-grid-fill",
    "chart-radar-grid-none",
    "chart-radar-icons",
    "chart-radar-label-custom",
    "chart-radar-legend",
    "chart-radar-lines-only",
    "chart-radar-multiple",
    "chart-radar-radius",
    "chart-radial-grid",
    "chart-radial-label",
    "chart-radial-shape",
    "chart-radial-simple",
    "chart-radial-stacked",
    "chart-radial-text",
    "chart-tooltip-advanced",
    "chart-tooltip-default",
    "chart-tooltip-demo",
    "chart-tooltip-formatter",
    "chart-tooltip-icons",
    "chart-tooltip-indicator-line",
    "chart-tooltip-indicator-none",
    "chart-tooltip-label-custom",
    "chart-tooltip-label-formatter",
    "chart-tooltip-label-none",
];

#[test]
fn shadcn_chart_goldens_are_targeted_gates() {
    for &key in CHART_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        if key == "chart-tooltip-demo" {
            web_find_chart_tooltip_panel(&theme.root)
                .expect("missing chart tooltip panel (ChartTooltipContent)");
            continue;
        }

        let chart_root = web_find_chart_root(&theme.root)
            .unwrap_or_else(|| panic!("missing chart root (data-slot=chart) in {key}"));

        find_first(chart_root, &|n| {
            n.tag == "div" && class_has_token(n, "recharts-responsive-container")
        })
        .unwrap_or_else(|| panic!("missing recharts responsive container in {key}"));

        find_first(chart_root, &|n| {
            n.tag == "svg" && class_has_token(n, "recharts-surface")
        })
        .unwrap_or_else(|| panic!("missing recharts svg surface in {key}"));
    }
}
