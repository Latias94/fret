use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
struct DirectTextCounts {
    cx_text: usize,
    text_props_new: usize,
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn direct_text_counts() -> BTreeMap<String, DirectTextCounts> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut counts = BTreeMap::<String, DirectTextCounts>::new();
    for path in files {
        let source = std::fs::read_to_string(&path).expect("read source file");
        let rel = path
            .strip_prefix(manifest_dir)
            .expect("source path should be under manifest dir")
            .to_string_lossy()
            .replace('\\', "/");

        let count = counts.entry(rel).or_default();
        count.cx_text += source.matches("cx.text(").count();
        count.text_props_new += source.matches("TextProps::new(").count();
    }

    counts.retain(|_, count| count.cx_text > 0 || count.text_props_new > 0);
    counts
}

#[test]
fn remaining_bare_text_in_fret_examples_is_explicit_capability_surface() {
    let expected = BTreeMap::from([
        (
            "src/components_gallery.rs".to_string(),
            DirectTextCounts {
                cx_text: 4,
                text_props_new: 3,
            },
        ),
        (
            "src/ime_smoke_demo.rs".to_string(),
            DirectTextCounts {
                cx_text: 8,
                text_props_new: 0,
            },
        ),
    ]);

    assert_eq!(
        direct_text_counts(),
        expected,
        "remaining bare text in fret-examples must stay limited to explicit text/IME capability proofs",
    );

    let gallery = include_str!("../src/components_gallery.rs");
    for needle in [
        "text_smoke_title",
        "text_smoke_lines",
        "gallery.text_smoke.fonts.load",
        "Forced UI font: {name}",
        "Forced emoji font: {name}",
        "Emoji (forced): {}",
        "TextProps::new(line.clone())",
        "out.push(cx.text(line));",
    ] {
        assert!(
            gallery.contains(needle),
            "components_gallery residual bare text should remain scoped to text smoke/font probes; missing `{needle}`",
        );
    }

    let ime = include_str!("../src/ime_smoke_demo.rs");
    for needle in [
        "IME smoke",
        "Microsoft Pinyin",
        "inline preedit",
        "cx.text(last)",
        "Single-line input",
        "Multiline textarea",
        "focus traversal",
    ] {
        assert!(
            ime.contains(needle),
            "ime_smoke_demo residual bare text should remain scoped to IME behavior probes; missing `{needle}`",
        );
    }
}
