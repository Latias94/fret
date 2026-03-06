# Action-First Authoring + View Runtime (Fearless Refactor v1) — Milestones

Last updated: 2026-03-06

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`

---

## Current status snapshot (as of 2026-03-06)

This snapshot is intentionally evidence-based: only mark a milestone as “Met” when the in-tree code,
teaching surfaces, and gates line up.

- **M0**: Met (workstream docs + ADRs exist; indices are updated).
- **M1**: In progress (typed unit actions exist; continue converging keymap/palette/menu/pointer triggers on the same dispatch pipeline).
- **M2**: In progress (View runtime v1 exists; `ViewCx` action helpers landed; default onboarding has narrowed to three entrypoints; adoption in templates + cookbook/examples is ongoing).
- **M3**: Planned (multi-frontend convergence: declarative + imui + GenUI).
- **M4**: In progress (cookbook/examples + ui-gallery continue migrating to the same authoring surface).
- **M5**: Planned (editor-grade proof points: docking/workspace integration).
- **M6**: Planned (MVU deprecation window, then hard delete once adoption gates are met).
- **M7–M9**: Proposed (payload actions v2, MVU deprecation, MVU hard delete).

Evidence anchors (verified in-tree as of 2026-03-06):

- `ecosystem/fret/src/view.rs` (`ViewCx::on_action_notify_*` helpers)
- `ecosystem/fret-ui-kit/src/activate.rs` (`on_activate_*` helpers for low-noise `OnActivate` authoring)
- `apps/fretboard/src/scaffold/templates.rs` (scaffold templates prefer View + typed actions)
- `apps/fret-cookbook/examples/async_inbox_basics.rs` (`Cancel` uses the default path; `Start` remains advanced for host-side dispatcher/inbox scheduling)
- `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs` (prefers `on_action_notify*` helpers)
- `apps/fret-cookbook/examples/form_basics.rs` (prefers `on_action_notify_models`)
- `apps/fret-cookbook/examples/toast_basics.rs` (intentional advanced reference case for imperative Sonner host integration)
- `apps/fret-cookbook/examples/router_basics.rs` (`ClearIntents` uses the default path; back/forward remain advanced for router availability sync)
- `apps/fret-cookbook/examples/undo_basics.rs` (`Inc`/`Dec`/`Reset` use the default path; `Undo`/`Redo` keep the host-side RAF effect)
- `apps/fret-cookbook/examples/virtual_list_basics.rs` (prefers `on_action_notify_models` for scroll actions)
- `apps/fret-cookbook/examples/query_basics.rs` (prefers action helpers)
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs` (prefers action helpers)
- `apps/fret-examples/src/custom_effect_v1_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/liquid_glass_demo.rs` (reset/preset/toggle-inspector actions now use the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/async_playground_demo.rs` (`ToggleTheme` now uses the default `on_action_notify_models` path and keeps the theme side effect as render-time state synchronization)
- `apps/fret-examples/src/custom_effect_v2_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/imui_floating_windows_demo.rs` (pressable overlap target uses `on_activate_notify`)
- `tools/gate_no_models_mut_in_action_handlers.py` (teaching-surface regression gate)
- `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py` (locks the approved advanced `on_action_notify` teaching-surface exceptions and keeps `fret-examples` plus ui-gallery pages/snippets on the zero-exception path)

Hardening follow-up (open):

- Key-context aware `when` evaluation (`keyctx.*`) is aligned across keymap matching, menus/palette gating, shortcut display, and diagnostics (see TODO `AFA-actions-019`).
- Embedded viewport interop has a view-runtime demo proving `record_engine_frame` composition (see TODO `AFA-adopt-044`).
- Authoring ergonomics: semantics/test IDs/key contexts can be attached before `into_element(cx)`, and `fret-ui-kit::ui::*` constructors are cx-less; cookbook + templates demonstrate the patterns (see TODO “Reduce authoring noise”).
- Teaching-surface convergence: cookbook/examples are gated to avoid legacy `stack::*` layout helpers and teach one layout authoring surface (`fret-ui-kit::ui::*`); ui-gallery migration is in progress (see TODO “Reduce authoring noise” and gates `tools/gate_no_stack_in_cookbook.py`, `tools/gate_no_stack_in_examples.py`).
- Helper-surface convergence: README/docs/templates now frame `on_action_notify_models`, `on_action_notify_transient`, and local `on_activate*` as the default mental model; advanced aliases remain available but should stay out of first-contact material unless repeated demo evidence promotes them. The remaining advanced `on_action_notify` teaching cases are cookbook-only host-side categories locked by `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py`.

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
- If legacy-only: a deprecation window exists (warnings + migrations), and in-tree demos remain
  buildable while migrating.
- If removal is adopted: MVU is hard-deleted in-tree under M9.

### M9 — Hard delete legacy MVU (in-tree)

Exit criteria:

- `LEGACY_MVU_INVENTORY.md` has no remaining in-tree MVU usage.
- `ecosystem/fret` no longer exposes MVU surfaces:
  - delete `mvu` + `mvu_router` + `legacy` modules,
  - remove MVU re-exports from `prelude::*`.
- `apps/fret-examples` and `apps/fret-demo` no longer have legacy MVU demo routing.
- Docs/templates do not mention MVU as an available authoring path.
- A small gate prevents MVU APIs from being reintroduced (grep-based check is sufficient).

Current blockers (as of 2026-03-05):

- None.
