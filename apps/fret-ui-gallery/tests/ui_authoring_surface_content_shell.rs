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

#[test]
fn gallery_content_preview_panel_keeps_page_preview_as_a_vec_anyelement_boundary() {
    let normalized = assert_normalized_markers_present(
        "src/ui/content.rs",
        &[
            "let preview_panel_content = page_preview(cx, theme, selected, models);",
            "let preview_panel = cx.semantics(",
            "move |_cx| preview_panel_content",
            "fn page_preview(",
            ") -> Vec<AnyElement>",
            "vec![shadcn::Card::new(vec![",
        ],
    );

    assert!(
        !normalized.contains("return preview_inspector_torture(cx, theme)"),
        "gallery content preview panel should not regress to an inspector-specific single-element return path",
    );
    assert!(
        normalized.contains("PAGE_INSPECTOR_TORTURE => preview_inspector_torture(cx,theme),")
            || normalized.contains("PAGE_INSPECTOR_TORTURE=>preview_inspector_torture(cx,theme),"),
        "gallery content preview panel should still route inspector_torture through the shared preview boundary",
    );
}

#[test]
fn gallery_content_scroll_area_keeps_the_page_level_handle_without_a_nested_page_key() {
    let normalized = assert_normalized_markers_present(
        "src/ui/content.rs",
        &[
            "let scroll_handle = cx.slot_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());",
            "shadcn::ScrollArea::new([preview_panel])",
            ".scroll_handle(scroll_handle.clone())",
            ".test_id(\"ui-gallery-content-scroll\")",
        ],
    );

    assert!(
        !normalized.contains("ui_gallery.content_scroll_area"),
        "gallery content should not keep the nested page-keyed scroll wrapper",
    );
}

#[test]
fn gallery_content_scroll_area_gives_code_view_torture_a_known_preview_height() {
    let normalized = assert_normalized_markers_present(
        "src/ui/content.rs",
        &[
            "if selected == PAGE_CODE_VIEW_TORTURE {",
            "scroll=scroll.viewport_known_content_size(fret_core::Size::new(Px(0.0),CODE_VIEW_TORTURE_PREVIEW_HEIGHT));",
        ],
    );

    assert!(
        normalized.contains("scroll=scroll.viewport_focus_ring(false);"),
        "code-view torture should keep the viewport focus-ring opt-out alongside the known content size",
    );
}

#[test]
fn gallery_sidebar_view_does_not_keep_an_extra_keyed_shell_wrapper() {
    let normalized = assert_normalized_markers_present(
        "src/driver/shell.rs",
        &[
            "let selected = cx",
            "let query = cx",
            "ui::sidebar_view(",
            "if (bisect & BISECT_SIMPLE_SIDEBAR) != 0",
        ],
    );

    assert!(
        !normalized.contains("cx.keyed(\"ui_gallery.sidebar\""),
        "gallery sidebar should not keep an extra keyed shell wrapper around the nav view",
    );
}

#[test]
fn gallery_sidebar_view_keeps_chrome_and_layout_on_the_flex_root() {
    let normalized = assert_normalized_markers_present(
        "src/ui/nav.rs",
        &[
            "ui::v_flex(|_cx| [title_row, query_input, nav_scroll])",
            ".bg(ColorRef::Color(",
            ".p(Space::N4)",
            ".layout(",
            ".gap(Space::N4)",
            ".into_element(cx)",
        ],
    );

    assert!(
        !normalized.contains("let container = cx.container("),
        "gallery sidebar should not keep a dedicated container wrapper around the nav root",
    );
    assert!(
        !normalized.contains("decl_style::container_props("),
        "gallery sidebar should not route chrome/layout through a separate container props wrapper",
    );
}
