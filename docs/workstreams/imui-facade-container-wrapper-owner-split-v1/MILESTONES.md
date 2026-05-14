# ImUi Facade Container Wrapper Owner Split v1 - Milestones

Status: closed
Last updated: 2026-05-13

## M0 - Baseline

Exit criteria:

- The lane records the root `facade_writer.rs` line count after the value-model split.
- The wrapper family is named explicitly.
- Non-goals exclude new behavior and runtime public APIs.

Result: done in `M0_BASELINE_AUDIT_2026-05-13.md`.

## M1 - Owner Split

Exit criteria:

- `facade_writer/container_wrappers.rs` owns the structural container inherent wrappers.
- `facade_writer.rs` declares `mod container_wrappers;`.
- Public IMUI names and re-export paths stay unchanged.
- Focused gates pass.

Result: done in `M1_CONTAINER_FACADE_OWNER_SPLIT_2026-05-13.md`.

## M2 - Closeout

Exit criteria:

- The lane is marked closed.
- The active IMUI gap-closure lane and repo-wide indexes include the new owner anchor.
- Follow-on boundaries are explicit.

Result: done in `CLOSEOUT_AUDIT_2026-05-13.md`.
