use std::cell::RefCell;
use std::sync::OnceLock;

use crate::HIGHLIGHT_NAMES;
use tree_sitter_highlight::{Highlight, HighlightConfiguration};

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

#[allow(clippy::vec_init_then_push)]
pub fn supported_languages() -> &'static [&'static str] {
    static SUPPORTED: OnceLock<Vec<&'static str>> = OnceLock::new();
    SUPPORTED.get_or_init(|| {
        #[allow(unused_mut)]
        let mut out = Vec::new();
        #[cfg(feature = "lang-bash")]
        out.push("bash");
        #[cfg(feature = "lang-c")]
        out.push("c");
        #[cfg(feature = "lang-cpp")]
        out.push("cpp");
        #[cfg(feature = "lang-c-sharp")]
        out.push("csharp");
        #[cfg(feature = "lang-cmake")]
        out.push("cmake");
        #[cfg(feature = "lang-css")]
        out.push("css");
        #[cfg(feature = "lang-dart")]
        out.push("dart");
        #[cfg(feature = "lang-diff")]
        out.push("diff");
        #[cfg(feature = "lang-elixir")]
        out.push("elixir");
        #[cfg(feature = "lang-embedded-template")]
        out.push("embedded-template");
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
        #[cfg(feature = "lang-graphql")]
        out.push("graphql");
        #[cfg(feature = "lang-java")]
        out.push("java");
        #[cfg(feature = "lang-kotlin")]
        out.push("kotlin");
        #[cfg(feature = "lang-lua")]
        out.push("lua");
        #[cfg(feature = "lang-make")]
        out.push("make");
        #[cfg(feature = "lang-php")]
        out.push("php");
        #[cfg(feature = "lang-proto")]
        out.push("proto");
        #[cfg(feature = "lang-ruby")]
        out.push("ruby");
        #[cfg(feature = "lang-scala")]
        out.push("scala");
        #[cfg(feature = "lang-sql")]
        out.push("sql");
        #[cfg(feature = "lang-swift")]
        out.push("swift");
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
    } else if s.eq_ignore_ascii_case("sh") || s.eq_ignore_ascii_case("shell") {
        "bash"
    } else if s.eq_ignore_ascii_case("js") || s.eq_ignore_ascii_case("jsx") {
        "javascript"
    } else if s.eq_ignore_ascii_case("ts") || s.eq_ignore_ascii_case("tsx") {
        "typescript"
    } else if s.eq_ignore_ascii_case("md") {
        "markdown"
    } else if s.eq_ignore_ascii_case("yml") {
        "yaml"
    } else if s.eq_ignore_ascii_case("kt") || s.eq_ignore_ascii_case("kts") {
        "kotlin"
    } else if s.eq_ignore_ascii_case("py") {
        "python"
    } else if s.eq_ignore_ascii_case("rb") {
        "ruby"
    } else if s.eq_ignore_ascii_case("jsonc") {
        "json"
    } else if s.eq_ignore_ascii_case("c++") {
        "cpp"
    } else if s.eq_ignore_ascii_case("c#")
        || s.eq_ignore_ascii_case("csharp")
        || s.eq_ignore_ascii_case("c-sharp")
        || s.eq_ignore_ascii_case("cs")
    {
        "csharp"
    } else if s.eq_ignore_ascii_case("makefile") {
        "make"
    } else if s.eq_ignore_ascii_case("protobuf") {
        "proto"
    } else {
        s
    }
}

pub fn config_for(language: &str) -> Option<&'static HighlightConfiguration> {
    match normalize_language_name(language) {
        #[cfg(feature = "lang-bash")]
        "bash" => Some(bash_config()),
        #[cfg(feature = "lang-c")]
        "c" => Some(c_config()),
        #[cfg(feature = "lang-cpp")]
        "cpp" => Some(cpp_config()),
        #[cfg(feature = "lang-c-sharp")]
        "csharp" => Some(csharp_config()),
        #[cfg(feature = "lang-cmake")]
        "cmake" => Some(cmake_config()),
        #[cfg(feature = "lang-css")]
        "css" => Some(css_config()),
        #[cfg(feature = "lang-dart")]
        "dart" => Some(dart_config()),
        #[cfg(feature = "lang-diff")]
        "diff" => Some(diff_config()),
        #[cfg(feature = "lang-elixir")]
        "elixir" => Some(elixir_config()),
        #[cfg(feature = "lang-embedded-template")]
        "embedded-template" => Some(embedded_template_config()),
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
        #[cfg(feature = "lang-graphql")]
        "graphql" => Some(graphql_config()),
        #[cfg(feature = "lang-java")]
        "java" => Some(java_config()),
        #[cfg(feature = "lang-kotlin")]
        "kotlin" => Some(kotlin_config()),
        #[cfg(feature = "lang-lua")]
        "lua" => Some(lua_config()),
        #[cfg(feature = "lang-make")]
        "make" => Some(make_config()),
        #[cfg(feature = "lang-zig")]
        "zig" => Some(zig_config()),
        #[cfg(feature = "lang-md")]
        "markdown" => Some(markdown_config()),
        #[cfg(feature = "lang-php")]
        "php" => Some(php_config()),
        #[cfg(feature = "lang-proto")]
        "proto" => Some(proto_config()),
        #[cfg(feature = "lang-python")]
        "python" => Some(python_config()),
        #[cfg(feature = "lang-ruby")]
        "ruby" => Some(ruby_config()),
        #[cfg(feature = "lang-scala")]
        "scala" => Some(scala_config()),
        #[cfg(feature = "lang-sql")]
        "sql" => Some(sql_config()),
        #[cfg(feature = "lang-swift")]
        "swift" => Some(swift_config()),
        #[cfg(feature = "lang-yaml")]
        "yaml" => Some(yaml_config()),
        #[cfg(feature = "lang-toml")]
        "toml" => Some(toml_config()),
        _ => None,
    }
}

#[cfg(feature = "lang-bash")]
fn bash_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_bash::LANGUAGE.into(),
            "bash",
            include_str!("../languages/bash/highlights.scm"),
            "",
            "",
        )
        .expect("valid bash highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-c")]
fn c_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_c::LANGUAGE.into(),
            "c",
            include_str!("../languages/c/highlights.scm"),
            "",
            "",
        )
        .expect("valid c highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-cpp")]
fn cpp_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_cpp::LANGUAGE.into(),
            "cpp",
            include_str!("../languages/cpp/highlights.scm"),
            include_str!("../languages/cpp/injections.scm"),
            "",
        )
        .expect("valid cpp highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-c-sharp")]
fn csharp_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_c_sharp::LANGUAGE.into(),
            "csharp",
            include_str!("../languages/csharp/highlights.scm"),
            "",
            "",
        )
        .expect("valid csharp highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-cmake")]
fn cmake_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_cmake::LANGUAGE.into(),
            "cmake",
            include_str!("../languages/cmake/highlights.scm"),
            include_str!("../languages/cmake/injections.scm"),
            "",
        )
        .expect("valid cmake highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-css")]
fn css_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_css::LANGUAGE.into(),
            "css",
            include_str!("../languages/css/highlights.scm"),
            "",
            "",
        )
        .expect("valid css highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-dart")]
fn dart_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_dart_orchard::LANGUAGE.into(),
            "dart",
            include_str!("../languages/dart/highlights.scm"),
            include_str!("../languages/dart/injections.scm"),
            "",
        )
        .expect("valid dart highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-diff")]
fn diff_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_diff::LANGUAGE.into(),
            "diff",
            include_str!("../languages/diff/highlights.scm"),
            "",
            "",
        )
        .expect("valid diff highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-elixir")]
fn elixir_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_elixir::LANGUAGE.into(),
            "elixir",
            include_str!("../languages/elixir/highlights.scm"),
            include_str!("../languages/elixir/injections.scm"),
            "",
        )
        .expect("valid elixir highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-embedded-template")]
fn embedded_template_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_embedded_template::LANGUAGE.into(),
            "embedded-template",
            include_str!("../languages/embedded-template/highlights.scm"),
            "",
            "",
        )
        .expect("valid embedded-template highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
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
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-graphql")]
fn graphql_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_graphql::LANGUAGE.into(),
            "graphql",
            include_str!("../languages/graphql/highlights.scm"),
            include_str!("../languages/graphql/injections.scm"),
            "",
        )
        .expect("valid graphql highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-java")]
fn java_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_java::LANGUAGE.into(),
            "java",
            include_str!("../languages/java/highlights.scm"),
            "",
            "",
        )
        .expect("valid java highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-kotlin")]
fn kotlin_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_kotlin_ng::LANGUAGE.into(),
            "kotlin",
            include_str!("../languages/kotlin/highlights.scm"),
            include_str!("../languages/kotlin/injections.scm"),
            "",
        )
        .expect("valid kotlin highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-lua")]
fn lua_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_lua::LANGUAGE.into(),
            "lua",
            include_str!("../languages/lua/highlights.scm"),
            include_str!("../languages/lua/injections.scm"),
            "",
        )
        .expect("valid lua highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-make")]
fn make_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_make::LANGUAGE.into(),
            "make",
            include_str!("../languages/make/highlights.scm"),
            "",
            "",
        )
        .expect("valid make highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
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
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-md")]
fn markdown_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_md::LANGUAGE.into(),
            "markdown",
            include_str!("../languages/markdown/highlights.scm"),
            include_str!("../languages/markdown/injections.scm"),
            "",
        )
        .expect("valid markdown highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-php")]
fn php_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_php::LANGUAGE_PHP.into(),
            "php",
            include_str!("../languages/php/highlights.scm"),
            include_str!("../languages/php/injections.scm"),
            "",
        )
        .expect("valid php highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-proto")]
fn proto_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_proto::LANGUAGE.into(),
            "proto",
            include_str!("../languages/proto/highlights.scm"),
            "",
            "",
        )
        .expect("valid proto highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-python")]
fn python_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_python::LANGUAGE.into(),
            "python",
            include_str!("../languages/python/highlights.scm"),
            "",
            "",
        )
        .expect("valid python highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-ruby")]
fn ruby_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_ruby::LANGUAGE.into(),
            "ruby",
            include_str!("../languages/ruby/highlights.scm"),
            "",
            "",
        )
        .expect("valid ruby highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-scala")]
fn scala_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_scala::LANGUAGE.into(),
            "scala",
            include_str!("../languages/scala/highlights.scm"),
            "",
            "",
        )
        .expect("valid scala highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-sql")]
fn sql_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_sequel::LANGUAGE.into(),
            "sql",
            include_str!("../languages/sql/highlights.scm"),
            "",
            "",
        )
        .expect("valid sql highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-swift")]
fn swift_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_swift::LANGUAGE.into(),
            "swift",
            include_str!("../languages/swift/highlights.scm"),
            include_str!("../languages/swift/injections.scm"),
            "",
        )
        .expect("valid swift highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-yaml")]
fn yaml_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_yaml::LANGUAGE.into(),
            "yaml",
            include_str!("../languages/yaml/highlights.scm"),
            "",
            "",
        )
        .expect("valid yaml highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}

#[cfg(feature = "lang-toml")]
fn toml_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_toml_ng::LANGUAGE.into(),
            "toml",
            include_str!("../languages/toml/highlights.scm"),
            "",
            "",
        )
        .expect("valid toml highlight queries");
        cfg.configure(HIGHLIGHT_NAMES);
        cfg
    })
}
