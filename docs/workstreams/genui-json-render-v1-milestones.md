# GenUI Spec Rendering (json-render-inspired) v1 — Milestones

Status: Draft

This plan intentionally locks the spec shape early to avoid future refactors.

Design doc: `docs/workstreams/genui-json-render-v1.md`
TODO: `docs/workstreams/genui-json-render-v1-todo.md`

## M0 — Spec + validator baseline (contracts locked)

Exit criteria:

- Spec types exist (root + flat elements + optional state).
- Structural validator exists with stable issue codes and human-readable output.
- Renderer can fail deterministically with “invalid spec” errors (no silent drop).

## M1 — Minimal renderer + shadcn-backed catalog (static props)

Exit criteria:

- `fret-genui-core` renders a spec with a small component set.
- `fret-genui-shadcn` provides a curated, safe baseline catalog.
- Element identity is stable via `cx.keyed(element_key, ...)`.

## M2 — State + `visible` + basic expressions

Exit criteria:

- Renderer reads from a state model (`serde_json::Value`) via JSON Pointer.
- `visible` is supported.
- `$state` and `$cond/$then/$else` work for props and visibility.

## M3 — Repeat + item scope

Exit criteria:

- `repeat` renders children per item in a state array.
- `$item` and `$index` resolve correctly inside repeat scopes.
- Identity is stable across reorder when `repeat.key` is provided.

## M4 — Bindings + write-back for forms

Exit criteria:

- `$bindState` / `$bindItem` produce binding paths exposed to components.
- Shadcn form components can write back safely (controlled API).

## M5 — Actions + schema export

Exit criteria:

- Event → action binding dispatch works with resolved params.
- Catalog schema export exists for LLM structured outputs.
- At least one end-to-end demo spec is validated + rendered + interactive.

