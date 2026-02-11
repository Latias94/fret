# ADR 0212: Element Identity Debug Paths and Frame-Staged Element State

Status: Accepted

## Context

Fret uses a declarative, per-frame element tree (ADR 0028) with cross-frame state keyed by stable
identity. Today, element identity is represented as `GlobalElementId(u64)` for performance and
portability. This is a good runtime key but it is weak for:

- diagnosing focus/hover/capture issues (opaque IDs),
- inspector navigation to source callsites,
- debugging list keying mistakes (unkeyed reorder, duplicate keys),
- preparing for GPUI-style cached subtree execution (skip subtree execution without losing state/deps).

Zed/GPUI uses a path-based identity (`GlobalElementId(Arc<[ElementId]>)`) built through
`Window::with_global_id`, and stages element state across frames via `rendered_frame`/`next_frame`
with an explicit "accessed keys" set.

We want to close the debuggability gap while preserving Fret's lightweight runtime key and keeping
`crates/fret-ui` mechanism-only.

## Decision

### 1) Keep `GlobalElementId(u64)` as the runtime key

`GlobalElementId(u64)` remains the stable key for:

- element-local state (`(GlobalElementId, TypeId)`),
- routing targets (focus, actions, timers),
- declarative bridge mapping (`GlobalElementId -> NodeId`).

### 2) Add a diagnostics-only identity debug registry

Behind the `fret-ui/diagnostics` feature, record a debuggable "path" for element IDs:

- root name (per window root),
- callsite location (`file:line:col`),
- keyed salt hash (or slot for unkeyed children),
- parent chain.

This registry is best-effort and is used for:

- inspector display (hovered/focused element path),
- warnings that point to source (unkeyed reorder, duplicate key),
- future tooling (navigate-to-source).

It must not participate in correctness-critical decisions.

### 3) Add debug diagnostics for keying correctness

In debug builds (and/or diagnostics feature):

- unkeyed list reorder warnings must include callsite location,
- keyed list rendering must detect duplicate keys at a single callsite and emit a warning with
  source location.

### 4) Frame-stage element state (GPUI-style staging with lag)

Replace `last_seen_frame` mark/sweep with frame-staged state buffers:

- `next_state`: states touched during the current frame execution,
- `rendered_state`: states from the most recent completed frame,
- `lag_states`: a small ring (size `gc_lag_frames`) to preserve transient overlay/tree churn.

State access moves a state from `{next,rendered,lag}` into `next_state`.
This preserves the existing `gc_lag_frames` behavior while aligning with GPUI’s staged mental model.

This staging is required groundwork for cached subtree execution, where a subtree may be skipped
but still must preserve (or "inherit") state and dependency sets.

## Consequences

- Runtime identity stays fast and portable.
- Debuggability improves without inflating `GlobalElementId`.
- Keying mistakes become actionable (source locations; duplicate key detection).
- Element state storage becomes compatible with future cached subtree semantics.

## References

- ADR 0028: Declarative element tree + element state: `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0036: Observability/inspector hooks: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- GPUI identity + state staging (non-normative): `repo-ref/zed/crates/gpui/src/window.rs`
