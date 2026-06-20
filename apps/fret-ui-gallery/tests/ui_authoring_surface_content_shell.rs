mod support;

use support::{manifest_path, read_path};

fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return next;
        }
        canonical = next;
    }
}

fn assert_normalized_markers_present(relative_path: &str, required_markers: &[&str]) -> String {
    let path = manifest_path(relative_path);
    let source = read_path(&path);
    let normalized = canonicalize_rust_fragment(&source);

    for marker in required_markers {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    normalized
}

#[test]
fn gallery_content_header_keeps_semantics_on_the_existing_header_root() {
    let normalized = assert_normalized_markers_present(
        "src/ui/content.rs",
        &[
            "let header = header_content.attach_semantics(",
            "SemanticsDecoration::default()",
            ".role(fret_core::SemanticsRole::Group)",
            ".test_id(Arc::from(\"ui-gallery-content-header\"))",
        ],
    );

    assert!(
        !normalized.contains(
            "let header = cx.semantics(fret_ui::element::SemanticsProps{layout:header_semantics_layout,role:fret_core::SemanticsRole::Group,test_id:Some(Arc::from(\"ui-gallery-content-header\")),..Default::default()},|_cx|[header_content],)"
        ),
        "gallery content header should not regress to a dedicated wrapper semantics node",
    );
}
