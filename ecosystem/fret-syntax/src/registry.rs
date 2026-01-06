use std::cell::RefCell;
use std::sync::OnceLock;

use tree_sitter_highlight::{Highlight, HighlightConfiguration};

pub const HIGHLIGHT_NAMES: [&str; 40] = [
    "attribute",
    "boolean",
    "comment",
    "comment.doc",
    "constant",
    "constructor",
    "embedded",
    "emphasis",
    "emphasis.strong",
    "enum",
    "function",
    "hint",
    "keyword",
    "label",
    "link_text",
    "link_uri",
    "number",
    "operator",
    "predictive",
    "preproc",
    "primary",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.list_marker",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regex",
    "string.special",
    "string.special.symbol",
    "tag",
    "tag.doctype",
    "text.literal",
    "title",
    "type",
    "variable",
    "variable.special",
    "variant",
];

thread_local! {
    static ACTIVE_HIGHLIGHTS: RefCell<Vec<Highlight>> = const { RefCell::new(Vec::new()) };
}

pub fn push_active_highlight(h: Highlight) {
    ACTIVE_HIGHLIGHTS.with(|stack| stack.borrow_mut().push(h));
}

pub fn pop_active_highlight() {
    ACTIVE_HIGHLIGHTS.with(|stack| {
        let mut stack = stack.borrow_mut();
        let _ = stack.pop();
    });
}

pub fn reset_active_highlights() {
    ACTIVE_HIGHLIGHTS.with(|stack| stack.borrow_mut().clear());
}

pub fn active_highlight_name() -> Option<&'static str> {
    ACTIVE_HIGHLIGHTS.with(|stack| {
        let stack = stack.borrow();
        let h = stack.last()?;
        HIGHLIGHT_NAMES.get(h.0).copied()
    })
}

pub fn supported_languages() -> &'static [&'static str] {
    static SUPPORTED: OnceLock<Vec<&'static str>> = OnceLock::new();
    SUPPORTED.get_or_init(|| {
        #[allow(unused_mut)]
        let mut out = Vec::new();
        #[cfg(feature = "lang-rust")]
        out.push("rust");
        #[cfg(feature = "lang-json")]
        out.push("json");
        #[cfg(feature = "lang-javascript")]
        out.push("javascript");
        #[cfg(feature = "lang-typescript")]
        out.push("typescript");
        #[cfg(feature = "lang-html")]
        out.push("html");
        #[cfg(feature = "lang-go")]
        out.push("go");
        #[cfg(feature = "lang-zig")]
        out.push("zig");
        #[cfg(feature = "lang-md")]
        out.push("markdown");
        #[cfg(feature = "lang-python")]
        out.push("python");
        #[cfg(feature = "lang-yaml")]
        out.push("yaml");
        #[cfg(feature = "lang-toml")]
        out.push("toml");
        out
    })
}

fn normalize_language_name(language: &str) -> &str {
    let s = language.trim();
    if s.eq_ignore_ascii_case("rs") {
        "rust"
    } else if s.eq_ignore_ascii_case("js") || s.eq_ignore_ascii_case("jsx") {
        "javascript"
    } else if s.eq_ignore_ascii_case("ts") || s.eq_ignore_ascii_case("tsx") {
        "typescript"
    } else if s.eq_ignore_ascii_case("md") {
        "markdown"
    } else if s.eq_ignore_ascii_case("yml") {
        "yaml"
    } else {
        s
    }
}

pub fn config_for(language: &str) -> Option<&'static HighlightConfiguration> {
    match normalize_language_name(language) {
        #[cfg(feature = "lang-rust")]
        "rust" => Some(rust_config()),
        #[cfg(feature = "lang-json")]
        "json" => Some(json_config()),
        #[cfg(feature = "lang-javascript")]
        "javascript" => Some(javascript_config()),
        #[cfg(feature = "lang-typescript")]
        "typescript" => Some(typescript_config()),
        #[cfg(feature = "lang-html")]
        "html" => Some(html_config()),
        #[cfg(feature = "lang-go")]
        "go" => Some(go_config()),
        #[cfg(feature = "lang-zig")]
        "zig" => Some(zig_config()),
        #[cfg(feature = "lang-md")]
        "markdown" => Some(markdown_config()),
        #[cfg(feature = "lang-python")]
        "python" => Some(python_config()),
        #[cfg(feature = "lang-yaml")]
        "yaml" => Some(yaml_config()),
        #[cfg(feature = "lang-toml")]
        "toml" => Some(toml_config()),
        _ => None,
    }
}

#[cfg(feature = "lang-rust")]
fn rust_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            include_str!("../languages/rust/highlights.scm"),
            include_str!("../languages/rust/injections.scm"),
            "",
        )
        .expect("valid rust highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-json")]
fn json_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_json::LANGUAGE.into(),
            "json",
            include_str!("../languages/json/highlights.scm"),
            "",
            "",
        )
        .expect("valid json highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-javascript")]
fn javascript_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "javascript",
            include_str!("../languages/javascript/highlights.scm"),
            include_str!("../languages/javascript/injections.scm"),
            "",
        )
        .expect("valid javascript highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-typescript")]
fn typescript_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "typescript",
            include_str!("../languages/typescript/highlights.scm"),
            "",
            "",
        )
        .expect("valid typescript highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-html")]
fn html_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_html::LANGUAGE.into(),
            "html",
            include_str!("../languages/html/highlights.scm"),
            include_str!("../languages/html/injections.scm"),
            "",
        )
        .expect("valid html highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-go")]
fn go_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_go::LANGUAGE.into(),
            "go",
            include_str!("../languages/go/highlights.scm"),
            "",
            "",
        )
        .expect("valid go highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-zig")]
fn zig_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_zig::LANGUAGE.into(),
            "zig",
            include_str!("../languages/zig/highlights.scm"),
            "",
            "",
        )
        .expect("valid zig highlight queries");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-md")]
fn markdown_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg =
            HighlightConfiguration::new(tree_sitter_md::LANGUAGE.into(), "markdown", "", "", "")
                .expect("valid markdown config");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-python")]
fn python_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg =
            HighlightConfiguration::new(tree_sitter_python::LANGUAGE.into(), "python", "", "", "")
                .expect("valid python config");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-yaml")]
fn yaml_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg =
            HighlightConfiguration::new(tree_sitter_yaml::LANGUAGE.into(), "yaml", "", "", "")
                .expect("valid yaml config");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-toml")]
fn toml_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg =
            HighlightConfiguration::new(tree_sitter_toml_ng::LANGUAGE.into(), "toml", "", "", "")
                .expect("valid toml config");
        cfg.configure(&HIGHLIGHT_NAMES);
        cfg
    })
}
