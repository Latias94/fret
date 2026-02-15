# GenUI Spec Rendering (json-render-inspired) v1 — Milestones

Status: In progress

This plan intentionally locks the spec shape early to avoid future refactors.

Design doc: `docs/workstreams/genui-json-render-v1.md`
TODO: `docs/workstreams/genui-json-render-v1-todo.md`

## M0 — Spec + validator baseline (contracts locked)

Exit criteria:

- ✅ Spec types exist (root + flat elements + optional state).
- ✅ Structural validator exists with stable issue codes and human-readable output.
- ✅ Renderer fails deterministically (invalid specs return issues; no silent drop).

## M1 — Minimal renderer + shadcn-backed catalog (static props)

Exit criteria:

- ✅ `fret-genui-core` renders a spec with a small component set.
- ✅ `fret-genui-shadcn` provides a curated, safe baseline catalog.
- ✅ Element identity is stable via `cx.keyed(element_key, ...)`.

## M2 — State + `visible` + basic expressions

Exit criteria:

- ✅ Renderer reads from a state model (`serde_json::Value`) via JSON Pointer.
- ✅ `visible` is supported (including `not`, `eq/neq`, comparisons, `$and/$or`).
- ✅ `$state` and `$cond/$then/$else` work for props and visibility.

## M3 — Repeat + item scope

Exit criteria:

- ✅ `repeat` renders children per item in a state array.
- ✅ `$item` and `$index` resolve correctly inside repeat scopes.
- ✅ Identity is stable across reorder when `repeat.key` is provided.

## M4 — Bindings + write-back for forms

Exit criteria:

- ✅ `$bindState` / `$bindItem` produce binding paths exposed to components.
- ✅ Shadcn Input/Switch can write back safely by emitting `setState` into the app-owned queue.

## M5 — Actions + schema export

Exit criteria:

- ✅ Event → action invocation emission works with resolved params.
- ✅ Catalog-derived JSON Schema + system prompt export exist for LLM structured outputs.
- ✅ Demo spec is validated + rendered + interactive (standard actions auto-applied in the demo app).

## M6 — Hardening + devtools closure (next)

Candidate exit criteria:

- Add an opt-in "spec auto-fixer" for common LLM mistakes (moved fields, missing defaults).
- Add at least one end-to-end harness test for spec rendering + interaction.
- Add lightweight in-app diagnostics panels (spec issues, state snapshot, action log) or integrate with existing devtools.

## M7 — Adaptive layout primitives (strategy layer)

Candidate exit criteria:

- `ResponsiveGrid` exists in `fret-genui-shadcn` and uses container queries by default.
- Demo spec includes a small section that visibly changes with window/panel resize.
- Catalog types express breakpoint-driven props cleanly (no ad-hoc stringly-typed breakpoints).

## M8 — Ingest utilities (mixed streams)

Candidate exit criteria:

- Mixed stream parser exists (text + JSONL patches) with deterministic behavior and small memory usage.
- Apps can opt into "patch-only" strict mode vs "mixed" mode.
