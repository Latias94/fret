const DOCS_INDEX: &str = include_str!("../../../docs/README.md");
const EXAMPLES_README: &str = include_str!("../../../docs/examples/README.md");
const AUTHORING_GOLDEN_PATH: &str = include_str!("../../../docs/authoring-golden-path-v2.md");
const FIRST_HOUR: &str = include_str!("../../../docs/first-hour.md");
const TODO_APP_GOLDEN_PATH: &str = include_str!("../../../docs/examples/todo-app-golden-path.md");
const ADR_0319: &str =
    include_str!("../../../docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md");
const SCAFFOLD_TEMPLATES: &str =
    include_str!("../../../crates/fretboard/src/scaffold/templates.rs");

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.split_whitespace().collect::<Vec<_>>().join(" ");
    let needle = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    haystack.contains(&needle)
}

#[test]
fn default_docs_freeze_local_state_as_the_only_first_contact_story() {
    assert!(
        AUTHORING_GOLDEN_PATH.contains("This is the only blessed first-contact local-state story.")
    );
    assert!(
        FIRST_HOUR
            .contains("It is **LocalState + view runtime + typed actions + keyed lists** only.")
    );
    assert!(EXAMPLES_README.contains(
        "They all teach the same small authoring model first: `LocalState` for view-owned state,"
    ));
    assert!(EXAMPLES_README.contains("This is the only blessed first-contact local-state story."));
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "This document teaches one default path only: LocalState-first app code on the `fret::app`"
    ));
    assert!(
        DOCS_INDEX.contains("first-contact state/identity: `LocalState<T>` for view-owned state,")
    );
    assert!(ADR_0319.contains(
        "### D2 — `LocalState<T>` remains the only blessed first-contact local-state story"
    ));
    assert!(ADR_0319.contains(
        "First-contact docs/templates/examples should not teach `LocalState::from_model(...)`,"
    ));
    assert!(SCAFFOLD_TEMPLATES.contains("State: LocalState-first"));
    assert!(SCAFFOLD_TEMPLATES.contains("prefer `LocalState<Vec<_>>` + payload actions"));
}

#[test]
fn default_docs_freeze_keyed_identity_for_dynamic_lists() {
    assert!(
        AUTHORING_GOLDEN_PATH
            .contains("**Identity**: keyed lists via `ui::for_each_keyed(...)` by default.")
    );
    assert!(
        AUTHORING_GOLDEN_PATH.contains(
            "For dynamic lists/subtrees, keyed identity is the only default teaching rule;"
        )
    );
    assert!(FIRST_HOUR.contains("If the list can change shape over time, assume it needs keys."));
    assert!(
        TODO_APP_GOLDEN_PATH.contains("If a list can insert/remove/reorder, assume it needs keys.")
    );
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "Keep unkeyed iteration as an explicit exception for static, never-reordered lists"
    ));
    assert!(
        EXAMPLES_README
            .contains("`simple-todo` (template) — view runtime + typed actions + keyed lists")
    );
    assert!(contains_normalized(
        EXAMPLES_README,
        "For dynamic lists/subtrees, teach keyed identity first (`ui::for_each_keyed(...)` or `ui.id(key, ...)`);"
    ));
    assert!(ADR_0319.contains(
        "On the default app lane, dynamic lists/subtrees should teach `ui::for_each_keyed(...)`,"
    ));
    assert!(ADR_0319.contains(
        "Unkeyed iteration remains an explicit exception only for static, order-stable collections."
    ));
    assert!(SCAFFOLD_TEMPLATES.contains(
        "When rendering dynamic lists, prefer `ui::for_each_keyed(cx, items, |item| id, |item| ...)`"
    ));
    assert!(
        SCAFFOLD_TEMPLATES
            .contains("For keyed dynamic lists, prefer `LocalState<Vec<_>>` + payload actions")
    );
}
