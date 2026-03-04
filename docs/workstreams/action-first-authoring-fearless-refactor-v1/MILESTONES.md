# Action-First Authoring + View Runtime (Fearless Refactor v1) — Milestones

Last updated: 2026-03-04

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`

---

## Current status snapshot (as of 2026-03-04)

- **M0**: Met (ADRs 0307/0308 are accepted; ADR index jump table is updated).
- **M1**: Met (ActionId identity + typed unit actions + converged metadata via command registry; dispatch diagnostics include handler scope and driver-handled classification).
- **M2**: Met (View runtime v1 + hooks + view-cache keepalive + gates exist; cookbook adoption landed).
- **M3**: Met (Declarative + imui + GenUI converge on the same ActionId dispatch and stable selectors; cross-frontend diag gate exists).
- **M4**: Met (ui-gallery includes an action-first view runtime snippet; templates + docs converge on View+actions).
- **M5**: Met (workspace shell demo tab strip uses action-first pointer dispatch hooks; scripted diag gate asserts pointer dispatch trace exists).
- **M6**: Met (legacy MVU authoring is quarantined; golden path is action-first + view runtime).
  - Status (as of 2026-03-04): MVU remains available as compat under `fret::legacy::prelude::*` while cookbook/templates stay MVU-free.
- **M6 evidence** (as of 2026-03-04): `apps/fret-examples/src/todo_demo.rs`, `apps/fret-examples/src/query_demo.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`, `apps/fret-examples/src/hello_counter_demo.rs`, `apps/fret-examples/src/async_playground_demo.rs`, `apps/fret-examples/src/embedded_viewport_demo.rs`, `apps/fret-examples/src/drop_shadow_demo.rs`, `apps/fret-examples/src/postprocess_theme_demo.rs`, `apps/fret-examples/src/custom_effect_v1_demo.rs`, `apps/fret-examples/src/custom_effect_v2_demo.rs`, `apps/fret-examples/src/custom_effect_v3_demo.rs`, `apps/fret-examples/src/liquid_glass_demo.rs`, `apps/fret-examples/src/genui_demo.rs`, `apps/fret-examples/src/markdown_demo.rs` are view runtime + typed actions (legacy MVU versions are opt-in where present).
- **M7**: Met (payload actions v2 contract + prototype landed; at least one in-tree demo uses it with a scripted diag gate).
- **M8**: Met (in-tree) (MVU is opt-in behind a legacy feature and surfaces are compile-time deprecated; in-tree legacy demos explicitly opt in).
- **M9**: Planned (in-tree) (remaining MVU demos migrated; legacy MVU feature + modules removed).

Hardening follow-up (post-M1):

- Key-context aware `when` evaluation (`keyctx.*`) is aligned across keymap matching, menus/palette gating, shortcut display, and diagnostics (see TODO `AFA-actions-019`).
- Embedded viewport interop has a view-runtime demo proving `record_engine_frame` composition (see TODO `AFA-adopt-044`).
- Authoring ergonomics: semantics/test IDs/key contexts can be attached before `into_element(cx)` and cookbook demos demonstrate the pattern (see TODO “Reduce authoring noise”).

---

## M0 — Decision gates locked (ADRs accepted)

Exit criteria:

- ADR 0307 and ADR 0308 exist as the canonical contract references (Status: Accepted).
- Keymap strategy (ActionId vs CommandId) is explicit and stable for v1.
- `docs/adr/README.md` jump table points to the new ADRs.

---

## M1 — Action system v1 landed (additive)

Exit criteria:

- Action IDs are stable and debuggable.
- UI can bind to actions without string parsing glue.
- Keymap can trigger actions and diagnostics can explain availability/dispatch outcomes.
- At least one palette/menu trigger path uses the same action dispatch pipeline (no divergence).

Notes:

- v1 may keep `ActionId` == `CommandId` to avoid schema churn; the key is the authoring surface and routing semantics.

---

## M2 — View runtime v1 landed (minimal, ecosystem-level)

Exit criteria:

- A minimal `View` + `ViewCx` exists with:
  - action handler table registration,
  - `notify()` dirty marking,
  - `use_state`, `use_selector`, `use_query` integration surfaces.
- At least one demo renders via the view runtime without MVU.
- At least one gate explains view rebuild reasons (notify vs observed deps vs inspection mode).

---

## M3 — Multi-frontend convergence (imui + GenUI alignment)

Exit criteria:

- imui can dispatch `ActionId` directly (no string commands required).
- GenUI action bindings align with action conventions and can trigger the same action handler surfaces.
- Diagnostics selectors still work across all frontends (stable `test_id` surfaces).

---

## M4 — Adoption (cookbook + gallery)

Exit criteria:

- At least 2 cookbook examples migrated and used as the “before/after” teaching baseline.
- At least 1 ui-gallery page/snippet migrated.
- Docs show a single golden path for new users.
- `fretboard` templates do not teach a conflicting default paradigm (updated or explicitly deferred).

---

## M5 — Editor-grade proof (docking/workspace integration)

Exit criteria:

- At least one editor-grade surface (workspace shell / docking) uses the new action-first routing where appropriate.
- Regression gates exist (tests + diag script) for action routing + availability under overlays/focus changes.

---

## M6 — Cleanup (delete legacy, keep it boring)

Exit criteria:

- Redundant/legacy APIs are removed or clearly quarantined as “legacy”.
- Templates default to action-first + view runtime patterns.
- No in-tree demo requires stringly command routing glue.
- `cargo nextest run` gates remain green.
- “Risk matrix” items (R1–R6) have explicit mitigations/gates or are explicitly deferred.

---

## Post-v1 milestones (proposed)

These milestones are intentionally outside the v1 closure, but define the safe path to reduce
long-term surface area without breaking downstream users.

### M7 — Payload actions (v2) decision + prototype

Exit criteria:

- A concrete contract exists for parameterized/payloaded actions, including determinism and
  diagnostics expectations.
  - ADR: `docs/adr/0312-payload-actions-v2.md`
- At least one in-tree demo migrates from MVU payload routing to payload actions (or an explicit
  alternative is adopted).
  - Demo: `apps/fret-cookbook/examples/payload_actions_basics.rs`
  - Gate: `tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json`

### M8 — MVU deprecation window (warn + migrate)

Exit criteria:

- MVU’s long-term stance is decided (supported vs legacy-only) and reflected in docs/templates.
- If legacy-only: compile-time deprecations (or feature gating) are staged behind a deprecation
  window and do not break in-tree demos.
  - Feature gate: `ecosystem/fret/Cargo.toml` (`legacy-mvu`)
  - Module gating: `ecosystem/fret/src/lib.rs`
  - In-tree opt-in: `apps/fret-examples/Cargo.toml`, `apps/fret-ui-gallery/Cargo.toml`

### M9 — Hard delete legacy MVU (in-tree)

Exit criteria:

- `LEGACY_MVU_INVENTORY.md` has no remaining in-tree MVU usage.
- `ecosystem/fret` no longer exposes MVU surfaces:
  - remove the `legacy-mvu` feature,
  - delete `mvu` + `mvu_router` + `legacy` modules,
  - remove MVU re-exports from `prelude::*`.
- `apps/fret-examples` and `apps/fret-demo` no longer have a `legacy-mvu-demos` feature or legacy MVU demo routing.
- Docs/templates do not mention MVU as an available authoring path.
- A small gate prevents MVU APIs from being reintroduced (grep-based check is sufficient).

Current blockers (as of 2026-03-05):

- `ecosystem/fret` still exposes legacy MVU (`legacy-mvu` feature + modules).
- `apps/fretboard` scaffolding still contains internal legacy MVU template sources (not user-facing, but must be removed for M9).
- Docs still describe MVU as an available compat path (needs cleanup + history note).
