use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
struct DirectTextCounts {
    ui_text: usize,
    ui_rich_text: usize,
    ui_raw_text: usize,
    ui_text_block: usize,
    cx_text: usize,
    cx_text_props: usize,
    text_props_new: usize,
    text_props_literal: usize,
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
        count.ui_text += source.matches("ui::text(").count();
        count.ui_rich_text += source.matches("ui::rich_text(").count();
        count.ui_raw_text += source.matches("ui::raw_text(").count();
        count.ui_text_block += source.matches("ui::text_block(").count();
        count.cx_text += source.matches("cx.text(").count();
        count.cx_text_props += source.matches("cx.text_props(").count();
        count.text_props_new += source.matches("TextProps::new(").count();
        count.text_props_literal += source.matches("cx.text_props(TextProps {").count()
            + source
                .matches("cx.text_props(fret_ui::element::TextProps {")
                .count();
    }

    counts.retain(|_, count| {
        count.ui_text > 0
            || count.ui_rich_text > 0
            || count.ui_raw_text > 0
            || count.ui_text_block > 0
            || count.cx_text > 0
            || count.cx_text_props > 0
            || count.text_props_new > 0
            || count.text_props_literal > 0
    });
    counts
}

#[test]
fn remaining_bare_text_in_fret_examples_is_explicit_capability_surface() {
    let expected = BTreeMap::from([
        (
            "src/components_gallery.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 4,
                cx_text_props: 3,
                text_props_new: 3,
                text_props_literal: 0,
            },
        ),
        (
            "src/cjk_conformance_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 7,
                text_props_new: 0,
                text_props_literal: 7,
            },
        ),
        (
            "src/custom_effect_v2_glass_chrome_web_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 3,
                text_props_new: 0,
                text_props_literal: 3,
            },
        ),
        (
            "src/custom_effect_v2_identity_web_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 3,
                text_props_new: 0,
                text_props_literal: 3,
            },
        ),
        (
            "src/custom_effect_v2_lut_web_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 3,
                text_props_new: 0,
                text_props_literal: 3,
            },
        ),
        (
            "src/custom_effect_v3_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 1,
                text_props_new: 0,
                text_props_literal: 1,
            },
        ),
        (
            "src/emoji_conformance_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 5,
                text_props_new: 0,
                text_props_literal: 5,
            },
        ),
        (
            "src/hello_counter_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 1,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 0,
                text_props_new: 0,
                text_props_literal: 0,
            },
        ),
        (
            "src/hello_world_compare_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 1,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 0,
                text_props_new: 0,
                text_props_literal: 0,
            },
        ),
        (
            "src/ime_smoke_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 8,
                cx_text_props: 0,
                text_props_new: 0,
                text_props_literal: 0,
            },
        ),
        (
            "src/liquid_glass_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 2,
                text_props_new: 0,
                text_props_literal: 2,
            },
        ),
        (
            "src/markdown_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 2,
                text_props_new: 0,
                text_props_literal: 2,
            },
        ),
        (
            "src/postprocess_theme_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 2,
                text_props_new: 0,
                text_props_literal: 2,
            },
        ),
        (
            "src/text_heavy_memory_demo.rs".to_string(),
            DirectTextCounts {
                ui_text: 0,
                ui_rich_text: 0,
                ui_raw_text: 0,
                ui_text_block: 0,
                cx_text: 0,
                cx_text_props: 1,
                text_props_new: 0,
                text_props_literal: 1,
            },
        ),
    ]);

    assert_eq!(
        direct_text_counts(),
        expected,
        "remaining direct text construction in fret-examples must stay limited to explicit text/IME/rendering capability proofs",
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

    let hello_counter = include_str!("../src/hello_counter_demo.rs");
    for needle in [
        "ui::text(count.to_string())",
        ".text_size_px(Px(72.0))",
        ".font_bold()",
        ".text_align(fret_core::TextAlign::Center)",
        ".nowrap()",
        ".test_id(TEST_ID_COUNT)",
    ] {
        assert!(
            hello_counter.contains(needle),
            "hello_counter_demo residual ui::text should remain the explicit large numeric display; missing `{needle}`",
        );
    }

    let hello_world_compare = include_str!("../src/hello_world_compare_demo.rs");
    for needle in [
        "FRET_HELLO_WORLD_COMPARE_NO_TEXT",
        "FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE",
        "ui::text(\"Hello, World!\")",
        ".text_size_px(Px(24.0))",
        ".font_semibold()",
        ".text_align(TextAlign::Center)",
        ".nowrap()",
        ".test_id(TEST_ID_TITLE)",
    ] {
        assert!(
            hello_world_compare.contains(needle),
            "hello_world_compare_demo residual ui::text should remain the explicit GPUI/Fret comparison payload; missing `{needle}`",
        );
    }
}
