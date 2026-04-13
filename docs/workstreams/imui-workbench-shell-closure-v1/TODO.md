# ImUi Workbench Shell Closure v1 - TODO

Status: active execution lane
Last updated: 2026-04-13

## Lane setup

- [x] Create the lane as a narrow P1 follow-on under the active immediate-mode product-closure
      umbrella.
- [x] Wire the lane into the umbrella docs, `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Freeze that this lane is not the new owner of tabstrip parity or multi-window runner parity.

## M0 - Baseline and owner freeze

- [x] Write one assumptions-first baseline audit that re-reads:
      - `workspace_shell_demo`,
      - `editor_notes_demo`,
      - `imui_editor_proof_demo`,
      - the P1 proof matrix,
      - and the promoted shell diag suite.
      Result: `M0_BASELINE_AUDIT_2026-04-13.md`.
- [x] Name the smallest current shell-composition gap that still blocks the default workbench path.
      Result: `M0_BASELINE_AUDIT_2026-04-13.md` now freezes shell assembly / first-party default-path
      posture as the current narrow gap instead of another tabstrip or generic `imui` helper slice.
- [x] Freeze a short "do not reopen here" note for:
      - generic `imui` helper growth,
      - tabstrip kernel parity,
      - and runner/backend multi-window closure.
      Result: `M0_BASELINE_AUDIT_2026-04-13.md`.

## M1 - Default workbench shell closure

- [x] Audit the current first-open `workspace_shell_demo` path for shell-level friction that is
      still product-significant after the tabstrip and editor-rail closeouts.
      Result: `M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md` now freezes shell-assembly
      posture as the current narrow decision point.
- [x] Decide whether the next landable slice belongs in:
      - `ecosystem/fret-workspace`,
      - `ecosystem/fret-ui-editor`,
      - `ecosystem/fret-docking`,
      - or app/example composition.
      Result: the current owner stays app/example composition above the frozen starter set; no new
      promoted shell helper is warranted yet.
- [x] Land or freeze one bounded shell slice with:
      - one source-level gate,
      - one launched/diag gate,
      - and one evidence note.
      Result: `M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md` plus the source-policy gate
      `immediate_mode_workstream_freezes_the_p1_default_workbench_assembly_decision` now freeze the
      no-new-helper-yet verdict against the existing P1 launched shell floor.

## M2 - Follow-on management

- [ ] If the remaining work clusters around tabstrip behavior, continue the existing
      `workspace-tabstrip*` / tab-bar lanes instead of widening this folder.
- [ ] If the remaining work clusters around runner/backend hand-feel, continue
      `docking-multiwindow-imgui-parity` instead of widening this folder.
- [ ] Close this lane or split a narrower follow-on once the default workbench proof and shell gate
      package are stable enough to stop using this folder as the active execution surface.
