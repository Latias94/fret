const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
const RELEASE_CHECKLIST: &str = include_str!("../../../docs/release/v0.1.0-release-checklist.md");
const RELEASE_CLOSEOUT: &str = include_str!(
    "../../../docs/workstreams/uicx-compat-alias-release-retirement-v1/CLOSEOUT_AUDIT_2026-04-20.md"
);

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.split_whitespace().collect::<Vec<_>>().join(" ");
    let needle = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    haystack.contains(&needle)
}

#[test]
fn changelog_carries_the_uicx_breaking_change_note() {
    assert!(CHANGELOG.contains("### Breaking Changes"));
    assert!(CHANGELOG.contains("UiCx*"));
    assert!(CHANGELOG.contains("AppComponentCx<'a>"));
    assert!(CHANGELOG.contains("AppRenderCx<'a>"));
    assert!(CHANGELOG.contains("AppRenderContext<'a>"));
    assert!(CHANGELOG.contains("UiCxActionsExt"));
    assert!(CHANGELOG.contains("UiCxDataExt"));
}

#[test]
fn release_checklist_requires_the_uicx_breaking_change_note() {
    assert!(RELEASE_CHECKLIST.contains("UiCx*"));
    assert!(RELEASE_CHECKLIST.contains("CHANGELOG.md"));
    assert!(RELEASE_CHECKLIST.contains("GitHub Release notes"));
    assert!(contains_normalized(
        RELEASE_CHECKLIST,
        "the release-facing breaking-change note is present for the deleted `UiCx*` compatibility aliases on `fret`, and it names the canonical replacements"
    ));
}

#[test]
fn closeout_points_at_repo_managed_release_note_surfaces() {
    assert!(RELEASE_CLOSEOUT.contains("CHANGELOG.md"));
    assert!(RELEASE_CLOSEOUT.contains("docs/release/v0.1.0-release-checklist.md"));
    assert!(RELEASE_CLOSEOUT.contains(
        "an explicit breaking-change note that `UiCx*` compatibility aliases were removed"
    ));
}
