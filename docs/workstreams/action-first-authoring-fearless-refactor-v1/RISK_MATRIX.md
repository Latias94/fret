# Action-First Authoring + View Runtime (Fearless Refactor v1) — Risk Matrix

Last updated: 2026-03-01

This matrix is a planning tool: it enumerates the highest-risk failure modes and the gates that
keep the refactor safe across native + wasm + future mobile targets.

This is not an ADR. If a mitigation is a hard contract, promote it to an ADR.

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

