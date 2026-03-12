---
title: UI Diagnostics Inspector (In-App) v1
status: draft
date: 2026-03-01
scope: diagnostics, inspector, in-app overlay
---

# UI Diagnostics Inspector (In-App) v1

This doc defines the **in-app** inspector UX and its runtime invariants.

Scope:

- In-app overlay + shortcuts: “what is this node?”, “why is input blocked?”, “copy a stable selector quickly”.
- Not in scope: a full DevTools GUI app (see `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`).

Key principle:

- The **portable source of truth remains the bundle** (`bundle.schema2.json` + sidecars). The in-app inspector is an
  iteration-speed tool, not a replacement for shareable artifacts.

## Contract boundaries

- `crates/fret-ui` remains a mechanism layer. The only stable hook needed by the in-app inspector is the ability to
  tell the UI tree “inspection is active” so hit-test metadata is available and caching is not a correctness footgun.
- Policy and UX live in `ecosystem/fret-bootstrap` (and later can migrate into `ecosystem/fret-ui-kit` as it matures).
- Selector quality rules are an **internal diagnostics policy** (not a `fret-ui` contract). The contract is the
  serialized selector types in `crates/fret-diag-protocol` and the ability to resolve them against a semantics snapshot.

## State machine (v1)

The in-app inspector has two user-facing modes:

1. **Inspect** (`inspect_enabled=true`)
2. **Pick armed** (`pick_armed=true`), which is a short-lived “next click selects” sub-mode.

Terminology:

- “Overlay present” means the UI renders the inspector HUD and/or selection outlines.
- “Inspection active” is the runtime flag that disables caching shortcuts and forces hit-test metadata availability.

### Invariants

These invariants should hold across all transports (filesystem triggers, DevTools WS, in-app shortcuts):

1. **Disabling inspect clears inspect state**
   - When `inspect_enabled` transitions to `false`, the runtime MUST clear any inspect-derived per-window state:
     - locked selection,
     - focus selection + nav stacks,
     - hover-derived selection,
     - inspect toasts (optional; toasts MAY remain as a short grace period but MUST NOT keep stale selection visible).
   - Reason: avoid “phantom inspector” overlays when tooling disables inspect out-of-band.

2. **Overlay visibility is driven by an explicit “wants_inspection_active” decision**
   - The overlay MUST NOT appear solely because stale selection data exists.
   - Presence is allowed when any of the following are true:
     - pick is armed,
     - inspect is enabled,
     - a short “pick overlay grace” is active (e.g. after a pick),
     - a toast/status message is active.

3. **Inspection active controls caching and hit-test metadata**
   - When overlay is present (per invariant 2), `UiTree::set_inspection_active(true)` MUST be set for that window.
   - When overlay is not present, it MUST be set to `false` to avoid perturbing perf baselines and cache behavior.

4. **Selection is window-scoped**
   - All inspect state (locked selection, hover selection, focus selection) is scoped to a specific window.

## UX behaviors (v1)

### Shortcuts

While inspection is active:

- `Ctrl/Cmd+Alt+I`: toggle inspect (in-app)
- `Ctrl/Cmd+Alt+H`: toggle help (in-app; enables inspect if needed)
- `Esc`: exit inspect (disarm pick first if armed)
- `Ctrl/Cmd+C`: copy best selector JSON (prefer `test_id`)
- `Ctrl/Cmd+Shift+C`: copy “selector + focus + path” details (including selector candidates + scores)
- `Ctrl/Cmd+T` (help): toggle the semantics tree browser panel (diagnostics-only)
- `F`: jump selection to the focused semantics node (locks selection)
- `L`: lock/unlock selection
- `Alt+Up/Down`: walk semantics parent chain, with a small “down stack” history
- (Help open) Type to filter the neighborhood view (`test_id` and `label` when unredacted); `Backspace` deletes.
- (Help open, search active) `Up/Down` selects a match; `Enter` locks selection to the selected match.
- (Help open, tree open, no search) `Up/Down` selects a tree row.
- (Help open, tree open, no search) `Left/Right` collapses/expands the selected tree node.
- (Help open) `Ctrl/Cmd+Enter` locks selection and copies the best selector JSON (works for both search matches and tree selection).
- (Help open) `PageUp/PageDown/Home/End` scroll the help output (paged text rendering).

### Overlay primitives

The overlay provides:

- A compact HUD panel (top-left) with status text and the best selector.
- An optional help view (togglable) that lists shortcuts and current mode flags.
- Outlines for:
  - `focus` (cyan),
  - `picked` (magenta),
  - `hovered` (green),
  each with a small label including `role`, `node_id`, optional `test_id`, optional `label`, and root `z_index`.

### Explainability panel (“why is input blocked?”)

The inspector must be able to answer *at least* the following questions in-app without requiring a bundle export:

1. “What would receive this click if I click now?”
2. “Why does the click not reach the underlay?”
3. “Which root/overlay layer is currently topmost and why?”

v1 UI requirements (minimal, top-left HUD extension):

- A collapsible panel (default hidden; shown when help is open) that displays:
  - pointer position and the current hit-test target (best-effort),
  - barrier summaries: `barrier_root` and `focus_barrier_root` (semantics ids, if present),
  - visible roots summary: `(root_id, z_index, blocks_underlay_input, hit_testable)`,
  - the “picked/hovered/focus” root z-index (so root disambiguation is obvious).
- The panel MUST be cheap to compute:
  - no full tree dumps per frame,
  - prefer O(depth) parent walks / bounded lists,
  - avoid allocating large strings unless the help panel is open.
- Redaction: when `FRET_DIAG_REDACT_TEXT=1`, the panel MUST NOT display unredacted labels.

Selection/hit-test policy note:

- When possible, “what is under the pointer” should prefer the runtime hit-test result and then walk up the semantics
  parent chain to find a selectable semantics node. This matches routing outcomes under overlays and is resilient when
  bounds overlap or when multiple roots exist.
- If hit-test metadata is missing/unavailable, fall back to bounds-based picking.

## Selector strategy

The “best selector” must be stable under refactors:

1. Prefer `test_id`.
2. Else prefer semantic `(role + name)` with an ancestor path only when needed for disambiguation.
3. Fall back to `global_element_id` when available (harness-quality, but not author-friendly).
4. Use `node_id` only as a last resort (in-run reference only).

When multiple visible roots exist (multiple overlays / viewports), the selector may include an optional
`root_z_index` gate to disambiguate otherwise-ambiguous matches.

## Implementation anchors (current)

This workstream intentionally keeps “evidence anchors” stable so future refactors can be audited quickly:

- In-app inspector shortcuts + hover/navigation/focus behaviors: `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs` (`InspectController`)
- Pick arming + pick-result-to-inspector-state updates: `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs` (`InspectController::{arm_pick,on_pick_success}`) and `ecosystem/fret-bootstrap/src/ui_diagnostics/pick_flow.rs`
- `UiDiagnosticsService` compatibility wrapper (delegates to `InspectController`): `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs`
- Overlay read path (snapshot): `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs` (`UiDiagnosticsService::inspect_overlay_model` / `InspectOverlayModel`)
- Inspector per-window state bucket: `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_state.rs` (`InspectState`)
- In-app overlay rendering: `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_overlay.rs`
- Explainability panel lines (why input is blocked): `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_explain.rs`
- Help neighborhood view lines (parent/siblings/children + filter): `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_neighborhood.rs`
- Help tree browser model (paged, keyboard-driven): `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_tree.rs`
- “Inspection active” wiring (view-cache correctness): `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Selector scoring/validation: `ecosystem/fret-bootstrap/src/ui_diagnostics/selector/validate.rs`
- Scripted inspector helper step (protocol + runner): `crates/fret-diag-protocol/src/lib.rs` (`InspectHelpLockBestMatchAndCopySelector`) and `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_inspect.rs`
- Regression gate (suite-level): suite `ui-gallery-overlay-steady` (scripts: `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-help-lock-match-copy-selector-steady.json`, `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-help-tree-lock-match-copy-selector-steady.json`).

## Non-goals (v1)

- A full-featured semantics tree browser inside the target app (virtualized, mouse-driven, global search, etc.). The
  in-app inspector only provides a minimal, keyboard-driven tree view in help mode.
- Script authoring/editing inside the target app.
- Remote debugging across machines as a supported feature.
