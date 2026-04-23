# ImUi Collection Second Proof Surface v1 - TODO

- [x] Freeze the lane-opening rationale and assumptions-first baseline for the next non-multi-window second proof-surface follow-on after command-package closeout.
- [x] Freeze the current proof roster in docs/source-policy with `editor_notes_demo.rs` as the primary candidate and `workspace_shell_demo.rs` as supporting evidence.
- [x] Land one materially different shell-mounted collection surface on an existing demo, starting from `editor_notes_demo.rs`.
      Result: `M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md` records the landed
      `Scene collection` left-rail surface in `editor_notes_demo.rs`, with stable collection
      summary/list test ids and app-owned row labels over the existing selection actions.
- [x] Keep the supporting `workspace_shell_demo.rs` evidence explicit without turning it into the only second proof candidate.
      Result: the M2 gate floor still includes `workspace_shell_pane_proof_surface` and
      `workspace_shell_editor_rail_surface`, while the landed collection surface lives in the
      smaller primary candidate `editor_notes_demo.rs`.
- [x] Revisit any shared collection helper widening only after the second proof surface lands, and only in a different narrow lane if stronger evidence still exists.
      Result: `CLOSEOUT_AUDIT_2026-04-23.md` now closes this lane on a no-helper-widening verdict:
      the second proof surface exists, but it does not yet prove that both collection proof surfaces
      need the same shared helper or that explicit app-owned code is now unreasonable.
