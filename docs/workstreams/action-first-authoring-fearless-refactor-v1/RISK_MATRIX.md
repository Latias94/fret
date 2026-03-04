# Action-First Authoring + View Runtime (Fearless Refactor v1) — Risk Matrix

Last updated: 2026-03-04

This matrix is a planning tool: it enumerates the highest-risk failure modes and the gates that
keep the refactor safe across native + wasm + future mobile targets.

This is not an ADR. If a mitigation is a hard contract, promote it to an ADR.

---

## Review pass (M0/M1) — 2026-03-03

This section records the current mitigation status and the concrete gates/evidence we rely on today.

- R1: Mostly mitigated (dispatch explainability + cross-trigger parity gates exist).
  - Evidence/gates:
    - `apps/fret-cookbook/examples/commands_keymap_basics.rs`
    - `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json`
    - `crates/fret-ui/src/tree/tests/command_dispatch_source_trace.rs`
- R2: Partially mitigated (modal barrier gate exists; mechanism-level availability tests still thin).
  - Evidence/gates:
    - `apps/fret-cookbook/examples/overlay_basics.rs`
    - `tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json`
- R3: Mostly mitigated (keyed hooks + debug rails exist; still relies on author discipline for complex loops).
  - Evidence:
    - `ecosystem/fret/src/view.rs` (hook keying + debug warnings)
    - `ecosystem/fret-selector/src/ui.rs` (`DepsBuilder` + debug warnings)
- R4: Partially mitigated (view-cache reuse has keepalive handler install; inspection/picking divergence gates still TODO).
  - Evidence:
    - `ecosystem/fret/src/view.rs` (handler keepalive under reuse)
    - `docs/ui-diagnostics-and-scripted-tests.md`
- R5: Mitigated (wasm smoke gates exist for the view runtime surface).
  - Evidence/gates:
    - `tools/gates_wasm_smoke.ps1`
- R6: Mostly mitigated (templates + docs converge on one golden path; deletion/quarantine remains for M6).
  - Evidence:
    - `apps/fretboard/src/scaffold/templates.rs`
    - `docs/README.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## R1 — Action dispatch becomes non-deterministic (focus/roots/key contexts)

Symptoms:

- the same keybinding triggers different outcomes depending on unrelated tree changes,
- palette/menus invoke a different handler than pointer-triggered UI,
- focus changes cause “lost commands” with no explanation.

Mitigation / gates:

- dispatch must follow ADR 0020 scope order (focused widget → window → app),
- add diagnostics traces:
  - keymap resolution → action id,
  - availability outcome,
  - dispatch scope resolution,
- scripted diag repro (ADR 0159): assert action dispatch under focus changes and modal barriers.

---

## R2 — Availability gating is inconsistent with Input Dispatch v2

Symptoms:

- UI shows enabled but action is blocked at dispatch,
- `prevent_default` interacts incorrectly with action availability,
- modal barriers block wrong actions.

Mitigation / gates:

- align availability queries with ADR 0218 semantics,
- add at least one mechanism test + one ecosystem integration test.

---

## R3 — View runtime introduces “hook order” footguns

Symptoms:

- derived/query/local state collides in loops,
- rendering order changes cause state to “jump” between items,
- caches become invalid silently.

Mitigation / gates:

- explicit hook keying rules in ADR 0308,
- keyed variants (`use_*_keyed`) required for loops,
- debug-only diagnostics rails (warn on multiple calls per frame or missing observed deps).

---

## R4 — View cache reuse hides correctness bugs (inspection/picking drift)

Symptoms:

- inspector/picker shows stale semantics or wrong dispatch paths,
- scripted selectors fail to resolve when caching is enabled,
- “works without diag, fails with diag” divergence.

Mitigation / gates:

- define the “inspection disables reuse” rule at cache boundaries,
- add diag scripts that run in inspection/picking modes,
- ensure diagnostics bundles record whether reuse was enabled for the frame.

---

## R5 — wasm portability regressions (desktop-only types leak)

Symptoms:

- `cargo check --target wasm32-unknown-unknown` fails for authoring crates,
- action/view runtime depends on winit/wgpu types.

Mitigation / gates:

- wasm build-only smoke gates for:
  - view runtime crate(s),
  - `fret-ui-kit`/`fret` facades that re-export the surface,
- enforce dependency policy (no backend crates in portable layers).

---

## R6 — Ecosystem drift (two golden paths survive)

Symptoms:

- templates teach MVU-only, docs mention actions/views, gallery uses string commands,
- duplicated APIs linger and confuse new contributors.

Mitigation / gates:

- treat cleanup as a milestone with exit criteria (M6),
- update templates/docs early (deprecate in docs first),
- add CI grep gates for reintroduced stringly routing patterns (keep existing checks, extend if needed).
