use std::io::BufReader;
use std::path::PathBuf;

#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

fn shadcn_web_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
}

fn assert_finite_rect(label: &str, rect: WebRect) {
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

fn validate_rects_in_dom_tree(label: &str, node: &WebNode) {
    assert!(!node.tag.is_empty(), "missing tag for {label}");
    assert_finite_rect(label, node.rect);
    for child in &node.children {
        validate_rects_in_dom_tree(label, child);
    }
}

#[test]
fn shadcn_web_goldens_smoke_parse_and_rects_valid() {
    let dir = shadcn_web_dir();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|err| {
        panic!(
            "missing shadcn web goldens dir: {}\nerror: {err}",
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

        let file = std::fs::File::open(&path).unwrap_or_else(|err| {
            panic!(
                "failed to read shadcn web golden: {}\nerror: {err}",
                path.display()
            )
        });
        let golden: WebGolden =
            serde_json::from_reader(BufReader::new(file)).unwrap_or_else(|err| {
                panic!(
                    "failed to parse shadcn web golden: {}\nerror: {err}",
                    path.display()
                )
            });

        assert!(!golden.themes.is_empty(), "missing themes in {name}");
        let theme = golden
            .themes
            .get("light")
            .or_else(|| golden.themes.get("dark"))
            .unwrap_or_else(|| panic!("missing light/dark theme in {name}"));

        assert!(
            theme.viewport.w.is_finite() && theme.viewport.w > 0.0,
            "{name} viewport.w"
        );
        assert!(
            theme.viewport.h.is_finite() && theme.viewport.h > 0.0,
            "{name} viewport.h"
        );

        validate_rects_in_dom_tree(name, &theme.root);

        found += 1;
    }

    assert!(
        found >= 100,
        "expected many shadcn web goldens in {}",
        dir.display()
    );
}
