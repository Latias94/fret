# Material3 Interaction Regression Family Split v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Family Ownership Split

Exit criteria:

- Navigation tests are isolated in `material3_navigation_interactions.rs`.
- Overlay tests are isolated in `material3_overlay_interactions.rs`.
- Choice/action tests are isolated in `material3_choice_action_interactions.rs`.
- `material3_interaction_regressions.rs` contains only explicitly deferred residual families.

Status: Complete.

## M2: Harness Hygiene

Exit criteria:

- Each family binary has narrowed imports.
- No broad helper/import warnings remain under focused `cargo check`.
- The split does not create new shared production APIs.

Status: Complete.

## M3: Closeout Evidence

Exit criteria:

- Focused nextest gates pass for the three new family binaries and the residual binary.
- Package-level test check and clippy gates pass.
- Workstream catalog, layering, JSON, and diff hygiene gates pass.

Status: Complete.
