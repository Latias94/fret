# Input Dispatch v2 Workstream (Phases, Default Actions, and Action Availability)

This document is a **workstream note** that complements ADR 1157 (`docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`).
It captures the concrete “editor-grade” ergonomics goals, cross-crate integration seams, and the remaining decision points
needed to avoid a large late-stage rewrite.

## Why this matters (editor-grade UX)

Editor UI requires a few “hand feel” primitives that must remain composable:

- explicit dispatch phases (Preview/Observer, Capture, Bubble),
- suppressing runtime default behavior without blocking propagation (`prevent_default`),
- queryable per-command availability along the current dispatch path (“is this action available?”),
- consistent gating across surfaces (shortcuts, command palette, OS menus, in-app menus).

These are **hard-to-change** because they cut across `fret-ui` dispatch, `fret-runtime` command metadata, runners/menus, and
ecosystem-level overlay policies.

## Contract summary (what we want to lock in)

### 1) Dispatch phases

Stable phase contract:

```text
Preview (Observer)  ->  Capture  ->  Bubble  ->  DefaultActions
```

- Preview exists for “click-through outside-press” and similar observer policies.
- Capture exists for deterministic pre-processing/cleanup (root → target).
- Bubble remains the primary component interaction phase (target → root).
- DefaultActions are mechanism-owned behaviors that can be suppressed.

### 2) `prevent_default` is orthogonal to propagation

- `stop_propagation`: control handler traversal (phase/path).
- `prevent_default(DefaultAction)`: suppress a specific runtime default behavior while still allowing propagation.

This keeps policy-heavy crates out of runtime internals while still enabling editor-grade overrides.

### 3) Action availability is a pure query with tri-state semantics

Widget-scoped commands need a pure query surface:

- `Available`: the focused dispatch path considers this command available.
- `Blocked`: the focused dispatch path explicitly blocks the command (treat as unavailable).
- `NotHandled`: the dispatch path does not provide an answer (treat as **unknown**, not as false).

Rationale: “unknown” is crucial for composability. If a widget does not participate in a command family, it should not
silently disable the command across surfaces.

#### Snapshot semantics (runner-friendly)

The per-window published snapshot uses `Option<bool>` semantics:

- `Some(true)` for `Available`
- `Some(false)` for `Blocked`
- `None` for `NotHandled` (unknown)

This aligns `fret-runtime::WindowCommandActionAvailabilityService` with the `CommandAvailability` tri-state contract.

### 4) Cross-surface gating is aggregated in a data-only snapshot

For menus / command palette / shortcut help, we want a single data seam:

- `InputContext` (`WindowInputContextService`)
  - Includes a window-scoped input arbitration snapshot (`InputContext.window_arbitration`) so
    ecosystem policies (overlays, docking, viewport tools) can make consistent decisions without
    reaching into global services.
- explicit overrides (`WindowCommandEnabledService`)
- dispatch-path availability (`WindowCommandActionAvailabilityService`)

Aggregated as `WindowCommandGatingSnapshot` (`crates/fret-runtime/src/window_command_gating.rs`).

## Ecosystem integration (how components plug in)

### Command palette (shadcn surface)

The command palette should be a *global discovery surface* and should not accidentally inherit the “text input scope”
of its query field. Its entry list should be gated using the aggregated snapshot:

- per-command `when` gating (input context),
- explicit enable overrides,
- widget-scope action availability (dispatch-path).

### Overlay policy and “frozen command target”

When a command palette opens, there are two competing desirable behaviors:

1) **Freeze** gating/target to the pre-open focus (editor-style discoverability and stable enabled/disabled state).
2) **Follow focus** into the palette input (DOM-style; can unexpectedly disable most widget actions).

Recommendation for editor-grade UX: **freeze gating** for the palette while it is open, and keep the palette’s own
shortcuts scoped to itself via its own keymap/handlers.

This requires a stable, explicit integration seam (either a dedicated service snapshot, or a “command target” concept).

## Compatibility with “per-frame rebuilt” UI

Fret can migrate from a retained `UiTree` to a per-frame rebuilt element tree over time, but this contract remains valid
if we keep the following invariant:

> Action availability queries operate on the **latest rendered dispatch tree snapshot** (not on transient builder state).

That keeps runner/menus/palette gating consistent even as the internal authoring model evolves.

## Next steps (recommended order)

1) Lock the tri-state semantics in docs and tests (availability snapshot uses `None` for unknown).
2) Ensure command palette gating uses `WindowCommandGatingSnapshot` consistently across all entry builders.
3) Promote a single “frozen gating snapshot while overlay is open” pattern that other overlays can reuse.
4) Extend availability coverage for core text commands (`text.copy/cut/paste/select_all/clear/delete*`) across
   text widgets and read-only selection surfaces.
5) Expand menu/OS runner gating to use the same aggregated snapshot (no divergent heuristics).
