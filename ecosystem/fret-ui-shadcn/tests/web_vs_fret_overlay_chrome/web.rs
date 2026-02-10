use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebGolden {
    pub(super) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebGoldenTheme {
    #[allow(dead_code)]
    pub(super) root: WebNode,
    #[serde(default)]
    pub(super) portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    pub(super) portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    pub(super) viewport: Option<WebViewport>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(super) struct WebViewport {
    pub(super) w: f32,
    pub(super) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(super) struct WebRect {
    #[allow(dead_code)]
    pub(super) x: f32,
    #[allow(dead_code)]
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebNode {
    #[allow(dead_code)]
    pub(super) tag: String,
    #[serde(default)]
    pub(super) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) active: bool,
    #[serde(rename = "activeDescendant", default)]
    pub(super) active_descendant: bool,
    #[serde(default)]
    pub(super) text: Option<String>,
    pub(super) rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    pub(super) computed_style: BTreeMap<String, String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub(super) children: Vec<WebNode>,
}

pub(super) fn web_theme_named<'a>(golden: &'a WebGolden, name: &str) -> &'a WebGoldenTheme {
    golden
        .themes
        .get(name)
        .unwrap_or_else(|| panic!("missing {name} theme in web golden"))
}

pub(super) fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

pub(super) fn web_golden_path(file_name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(file_name)
}

pub(super) fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_path(&format!("{name}.open.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web open golden: {}\nerror: {err}",
            path.display()
        )
    })
}

pub(super) fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

pub(super) fn find_portal_by_role<'a>(
    theme: &'a WebGoldenTheme,
    role: &str,
) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("role").is_some_and(|v| v == role))
}

pub(super) fn find_portal_by_slot<'a>(
    theme: &'a WebGoldenTheme,
    slot: &str,
) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("data-slot").is_some_and(|v| v == slot))
}

pub(super) fn find_first<'a>(
    node: &'a WebNode,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Option<&'a WebNode> {
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

pub(super) fn find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
    })
}

pub(super) fn find_by_data_slot_and_state_and_text<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
            && n.text.as_deref() == Some(text)
    })
}

pub(super) fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

pub(super) fn web_border_widths_px(node: &WebNode) -> Option<[f32; 4]> {
    Some([
        node.computed_style
            .get("borderTopWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderRightWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderBottomWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderLeftWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
    ])
}

pub(super) fn web_corner_radii_effective_px(node: &WebNode) -> Option<[f32; 4]> {
    let max = node.rect.w.min(node.rect.h) * 0.5;
    let radius = |key: &str| {
        node.computed_style
            .get(key)
            .map(String::as_str)
            .and_then(parse_px)
            .map(|v| v.min(max))
    };

    Some([
        radius("borderTopLeftRadius")?,
        radius("borderTopRightRadius")?,
        radius("borderBottomRightRadius")?,
        radius("borderBottomLeftRadius")?,
    ])
}

pub(super) fn find_by_data_slot<'a>(node: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(node, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WebHighlightedNodeChrome {
    pub(super) bg: super::css_color::Rgba,
    pub(super) fg: super::css_color::Rgba,
}

pub(super) fn web_find_highlighted_menu_item_background(
    theme: &WebGoldenTheme,
) -> super::css_color::Rgba {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str().ends_with("-item"));
        if is_menuitem && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(super::parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web highlighted menuitem (data-slot ends_with '-item')");
    highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web highlighted menuitem backgroundColor")
}

pub(super) fn web_find_highlighted_listbox_option_chrome(
    theme: &WebGoldenTheme,
    item_slot: &str,
) -> WebHighlightedNodeChrome {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, item_slot: &str, out: &mut Vec<&'a WebNode>) {
        let is_option = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == item_slot);
        if is_option && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(super::parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, item_slot, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, item_slot, &mut candidates);
    }
    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .unwrap_or_else(|| panic!("web highlighted option (data-slot={item_slot})"));

    let bg = highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web highlighted option backgroundColor");
    let fg = highlighted
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web highlighted option color");

    WebHighlightedNodeChrome { bg, fg }
}

pub(super) fn web_find_highlighted_menu_item_chrome(
    theme: &WebGoldenTheme,
) -> WebHighlightedNodeChrome {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str().ends_with("-item"));
        if is_menuitem && is_item_slot {
            if let Some(bg) = node
                .computed_style
                .get("backgroundColor")
                .map(String::as_str)
                .and_then(super::parse_css_color)
                && bg.a > 0.01
            {
                out.push(node);
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    for portal in &theme.portals {
        collect(portal, &mut candidates);
    }

    let highlighted = candidates
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web highlighted menuitem (data-slot ends_with '-item')");

    let bg = highlighted
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web highlighted menuitem backgroundColor");
    let fg = highlighted
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web highlighted menuitem color");

    WebHighlightedNodeChrome { bg, fg }
}

pub(super) fn web_find_menu_item_chrome_by_slot_variant_and_text(
    theme: &WebGoldenTheme,
    item_slot: &str,
    variant: &str,
    text: &str,
) -> WebHighlightedNodeChrome {
    fn collect<'a>(
        node: &'a WebNode,
        item_slot: &str,
        variant: &str,
        text: &str,
        out: &mut Vec<&'a WebNode>,
    ) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_item_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == item_slot);
        let is_variant = node
            .attrs
            .get("data-variant")
            .is_some_and(|v| v.as_str() == variant);
        let has_text = node.text.as_deref() == Some(text);
        if is_menuitem && is_item_slot && is_variant && has_text {
            out.push(node);
        }
        for child in &node.children {
            collect(child, item_slot, variant, text, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    collect(&theme.root, item_slot, variant, text, &mut candidates);
    for portal in &theme.portals {
        collect(portal, item_slot, variant, text, &mut candidates);
    }
    for wrapper in &theme.portal_wrappers {
        collect(wrapper, item_slot, variant, text, &mut candidates);
    }

    let node = candidates.first().copied().unwrap_or_else(|| {
        panic!("web menu item not found: slot={item_slot} variant={variant:?} text={text:?}")
    });

    let bg = node
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web menu item backgroundColor");
    let fg = node
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web menu item color");

    WebHighlightedNodeChrome { bg, fg }
}

pub(super) fn web_find_open_menu_subtrigger_chrome(
    theme: &WebGoldenTheme,
    subtrigger_slot: &str,
    subtrigger_text: &str,
) -> WebHighlightedNodeChrome {
    fn collect<'a>(
        node: &'a WebNode,
        subtrigger_slot: &str,
        subtrigger_text: &str,
        out: &mut Vec<&'a WebNode>,
    ) {
        let is_menuitem = node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "menuitem");
        let is_slot = node
            .attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == subtrigger_slot);
        let is_open = node
            .attrs
            .get("data-state")
            .is_some_and(|v| v.as_str() == "open");
        let has_text = node.text.as_deref() == Some(subtrigger_text);
        if is_menuitem && is_slot && is_open && has_text {
            out.push(node);
        }
        for child in &node.children {
            collect(child, subtrigger_slot, subtrigger_text, out);
        }
    }

    let mut candidates: Vec<&WebNode> = Vec::new();
    collect(
        &theme.root,
        subtrigger_slot,
        subtrigger_text,
        &mut candidates,
    );
    for portal in &theme.portals {
        collect(portal, subtrigger_slot, subtrigger_text, &mut candidates);
    }
    for wrapper in &theme.portal_wrappers {
        collect(wrapper, subtrigger_slot, subtrigger_text, &mut candidates);
    }

    let node = candidates.first().copied().unwrap_or_else(|| {
        panic!("web menu subtrigger not found: {subtrigger_slot} text={subtrigger_text:?}")
    });

    let bg = node
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web subtrigger backgroundColor");
    let fg = node
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web subtrigger color");

    WebHighlightedNodeChrome { bg, fg }
}

pub(super) fn web_find_active_element<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    fn node_area(node: &WebNode) -> f32 {
        node.rect.w * node.rect.h
    }

    fn collect<'a>(
        node: &'a WebNode,
        active_descendants: &mut Vec<&'a WebNode>,
        actives: &mut Vec<&'a WebNode>,
    ) {
        if node.active_descendant {
            active_descendants.push(node);
        }
        if node.active {
            actives.push(node);
        }
        for child in &node.children {
            collect(child, active_descendants, actives);
        }
    }

    let mut active_descendants: Vec<&WebNode> = Vec::new();
    let mut actives: Vec<&WebNode> = Vec::new();
    collect(&theme.root, &mut active_descendants, &mut actives);
    for portal in &theme.portals {
        collect(portal, &mut active_descendants, &mut actives);
    }

    if let Some(best) = active_descendants
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
    {
        return best;
    }

    actives
        .into_iter()
        .max_by(|a, b| node_area(a).total_cmp(&node_area(b)))
        .expect("web activeElement")
}

pub(super) fn web_find_active_element_chrome(theme: &WebGoldenTheme) -> WebHighlightedNodeChrome {
    let active = web_find_active_element(theme);

    let bg = active
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web active element backgroundColor");
    let fg = active
        .computed_style
        .get("color")
        .map(String::as_str)
        .and_then(super::parse_css_color)
        .expect("web active element color");

    WebHighlightedNodeChrome { bg, fg }
}
