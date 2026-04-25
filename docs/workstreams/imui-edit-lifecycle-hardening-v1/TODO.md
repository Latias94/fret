# ImUi Edit Lifecycle Hardening v1 TODO

Status: active
Last updated: 2026-04-25

- [x] Keep `imui-edit-lifecycle-diag-gate-v1` closed and create this narrow follow-on.
- [x] Record a Dear ImGui-aligned baseline audit for the next value-edit lifecycle slice.
- [x] M1: audit slider, text, drag-value, and numeric input against the target invariant.
- [x] M1: classify any mismatch as private IMUI runtime state, editor control behavior, demo proof,
  or diagnostics drift before changing code.
- [x] M2: delete duplicated or misleading lifecycle plumbing if the audit finds a cleaner shared
  private kernel.
- [x] M2: stabilize retained node portal text/number input sizing through an editor-layer helper.
- [x] M2: stabilize public IMUI single-line input sizing through an IMUI-layer helper.
- [x] M2: keep runtime and authoring public contracts stable unless the audit proves a hard
  contract change is unavoidable.
- [x] M3: add rendered diagnostics proof for public IMUI single-line input bounds stability.
- [ ] M3: add or promote rendered proof for drag-value and numeric input edit outcomes.
- [x] M3: keep the response-signals and editor-proof diag suites green after hardening.
- [ ] M4: close out with evidence, gates, and residual gap routing.
