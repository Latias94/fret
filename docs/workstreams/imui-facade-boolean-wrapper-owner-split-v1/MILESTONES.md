# ImUi Facade Boolean Wrapper Owner Split v1 - Milestones

Status: closed
Last updated: 2026-05-13

## M0 - Baseline

Exit criteria:

- The source owner risk is scoped to the checkbox/radio/switch wrapper cluster.
- The before line count is recorded.
- Public/runtime non-goals are explicit.

Result: done in `M0_BASELINE_AUDIT_2026-05-13.md`.

## M1 - Owner Split

Exit criteria:

- `facade_writer/boolean_wrappers.rs` owns the moved inherent wrappers.
- `facade_writer.rs` no longer owns those wrapper bodies.
- Public methods remain callable through `ImUiFacade`.
- Focused IMUI smoke and source gates pass.

Result: done in `M1_BOOLEAN_FACADE_OWNER_SPLIT_2026-05-13.md`.

## Closeout

Exit criteria:

- `WORKSTREAM.json` is marked `closed` / `stay_closed`.
- The closeout audit records the no-public-API and no-runtime-contract verdict.
- Future wider work is pointed to separate follow-ons.
