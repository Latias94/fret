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
- Not in scope: a full DevTools GUI app (see `docs/workstreams/diag-devtools-gui-v1.md`).

Key principle:

- The **portable source of truth remains the bundle** (`bundle.schema2.json` + sidecars). The in-app inspector is an
  iteration-speed tool, not a replacement for shareable artifacts.

## Contract boundaries

- `crates/fret-ui` remains a mechanism layer. The only stable hook needed by the in-app inspector is the ability to
  tell the UI tree “inspection is active” so hit-test metadata is available and caching is not a correctness footgun.
- Policy and UX live in `ecosystem/fret-bootstrap` (and later can migrate into `ecosystem/fret-ui-kit` as it matures).

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
- `F`: jump selection to the focused semantics node (locks selection)
- `L`: lock/unlock selection
- `Alt+Up/Down`: walk semantics parent chain, with a small “down stack” history

### Overlay primitives

The overlay provides:

- A compact HUD panel (top-left) with status text and the best selector.
- An optional help view (togglable) that lists shortcuts and current mode flags.
- Outlines for:
  - `focus` (cyan),
  - `picked` (magenta),
  - `hovered` (green),
  each with a small label including `role`, `node_id`, optional `test_id`, optional `label`, and root `z_index`.

## Selector strategy

The “best selector” must be stable under refactors:

1. Prefer `test_id`.
2. Else prefer semantic `(role + name)` with an ancestor path only when needed for disambiguation.
3. Fall back to `global_element_id` when available (harness-quality, but not author-friendly).
4. Use `node_id` only as a last resort (in-run reference only).

When multiple visible roots exist (multiple overlays / viewports), the selector may include an optional
`root_z_index` gate to disambiguate otherwise-ambiguous matches.

## Non-goals (v1)

- A full semantics tree browser inside the target app.
- Script authoring/editing inside the target app.
- Remote debugging across machines as a supported feature.
