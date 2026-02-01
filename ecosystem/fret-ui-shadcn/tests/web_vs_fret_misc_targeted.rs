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

fn class_contains(node: &WebNode, needle: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.contains(needle))
}

fn attrs_has(node: &WebNode, key: &str, value: &str) -> bool {
    node.attrs.get(key).is_some_and(|v| v == value)
}

#[test]
fn shadcn_misc_goldens_are_targeted_gates() {
    // Accordion
    {
        let key = "accordion-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "animate-accordion-down")
        })
        .expect("missing accordion animation marker");
        find_first(&theme.root, &|n| class_contains(n, "lucide-chevron-down"))
            .expect("missing accordion chevron icon");
    }

    // AspectRatio
    {
        let key = "aspect-ratio-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| class_has_token(n, "object-cover"))
            .expect("missing aspect ratio image (object-cover)");
        find_first(&theme.root, &|n| {
            class_has_token(n, "bg-muted") && class_has_token(n, "rounded-lg")
        })
        .expect("missing aspect ratio wrapper (bg-muted rounded-lg)");
    }

    // Avatar
    {
        let key = "avatar-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "data-[slot=avatar]:ring-2")
        })
        .expect("missing avatar stack recipe marker");
        find_first(&theme.root, &|n| {
            n.tag == "img" && class_has_token(n, "aspect-square")
        })
        .expect("missing avatar image");
    }

    // Button (as child)
    {
        let key = "button-as-child";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "a" && class_has_token(n, "bg-primary")
        })
        .expect("missing <a> button-as-child recipe node");
    }

    // Card (with form)
    {
        let key = "card-with-form";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "@container/card-header")
        })
        .expect("missing card header container marker");
        find_first(&theme.root, &|n| {
            class_has_token(n, "bg-card") && class_has_token(n, "rounded-xl")
        })
        .expect("missing card root recipe node");
    }

    // Checkbox
    for &key in &["checkbox-disabled", "checkbox-with-text"] {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "data-[state=checked]:bg-primary")
        })
        .expect("missing checkbox checked-state marker");
        find_first(&theme.root, &|n| class_contains(n, "rounded-[4px]"))
            .expect("missing checkbox corner marker");
    }

    // Collapsible
    {
        let key = "collapsible-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "lucide-chevrons-up-down")
        })
        .expect("missing collapsible icon");
        find_first(&theme.root, &|n| attrs_has(n, "data-state", "closed"))
            .expect("missing collapsible data-state=closed marker");
    }

    // Command
    {
        let key = "command-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| class_contains(n, "cmdk-group-heading"))
            .expect("missing cmdk group-heading styling marker");
        find_first(&theme.root, &|n| class_contains(n, "lucide-search"))
            .expect("missing command search icon");
    }

    // Dashboard template
    {
        let key = "dashboard-01";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| class_contains(n, "group/sidebar-wrapper"))
            .expect("missing sidebar wrapper marker");
        find_first(&theme.root, &|n| class_contains(n, "peer/menu-button"))
            .expect("missing sidebar menu button marker");
    }

    // Data table
    {
        let key = "data-table-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "table" && class_contains(n, "caption-bottom")
        })
        .expect("missing data table <table> node");
        find_first(&theme.root, &|n| class_contains(n, "overflow-x-auto"))
            .expect("missing data table overflow wrapper");
    }
    {
        let key = "data-table-demo.empty";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "table" && class_contains(n, "caption-bottom")
        })
        .expect("missing data table <table> node");
        find_first(&theme.root, &|n| {
            class_contains(n, "h-24") && class_contains(n, "text-center")
        })
        .expect("missing data table empty-state cell");
    }

    // Dialog close button page (closed state)
    {
        let key = "dialog-close-button";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "button" && attrs_has(n, "data-state", "closed")
        })
        .expect("missing dialog trigger with data-state=closed");
    }

    // Drawer pages (closed state)
    for &key in &["drawer-demo", "drawer-dialog"] {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "button" && attrs_has(n, "data-state", "closed")
        })
        .expect("missing drawer trigger with data-state=closed");
    }

    // Label demo
    {
        let key = "label-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "label" && class_contains(n, "peer-disabled:cursor-not-allowed")
        })
        .expect("missing label peer-disabled marker");
    }

    // Pagination
    {
        let key = "pagination-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "nav" && attrs_has(n, "aria-label", "pagination")
        })
        .expect("missing pagination nav aria-label");
        find_first(&theme.root, &|n| class_contains(n, "lucide-chevron-left"))
            .expect("missing pagination previous icon");
        find_first(&theme.root, &|n| class_contains(n, "lucide-chevron-right"))
            .expect("missing pagination next icon");
    }

    // Skeleton
    for &key in &["skeleton-demo", "skeleton-card"] {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| class_has_token(n, "animate-pulse"))
            .expect("missing skeleton animate-pulse marker");
    }

    // Sonner pages (static trigger-only goldens)
    for &key in &["sonner-demo", "sonner-types"] {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "button" && class_contains(n, "shadow-xs")
        })
        .expect("missing sonner trigger button recipe node");
    }

    // Table
    {
        let key = "table-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            n.tag == "table" && class_contains(n, "caption-bottom")
        })
        .expect("missing <table> caption-bottom recipe node");
    }

    // Tabs
    {
        let key = "tabs-demo";
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");
        find_first(&theme.root, &|n| {
            class_contains(n, "data-[state=active]:bg-background")
        })
        .expect("missing tabs active-state recipe marker");
    }
}
