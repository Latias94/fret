# GenUI Spec Rendering (json-render-inspired) v1 — TODO

Status: Draft

Tracking doc for implementing the GenUI spec renderer and shadcn-backed catalog.

Design doc: `docs/workstreams/genui-json-render-v1.md`

## P0 — Contracts first (least-refactor path)

- [ ] Define spec grammar types (flat map + root) in `ecosystem/fret-genui-core`.
- [ ] Add a structural validator with stable issue codes (missing root, missing child, misplaced fields).
- [ ] Decide spec versioning strategy (`schema_version` field vs implicit).
- [ ] Define action binding shape (`on.<event> -> { action, params }`) and event naming conventions.

## P0 — Rendering MVP (small catalog)

- [ ] Implement `SpecRenderer` that renders `root` into `Vec<AnyElement>`.
- [ ] Identity contract: each element key is rendered under `cx.keyed(key, ...)`.
- [ ] Implement `emit(event)` dispatch plumbing (no-op actions first).
- [ ] Add `fret-genui-shadcn` catalog with 5–10 “safe baseline” components:
  - [ ] `Card`
  - [ ] `Text` / `Heading`
  - [ ] `Button`
  - [ ] `Badge`
  - [ ] `Input` / `Textarea` (optional for initial)

## P1 — State + expressions

- [ ] Introduce `Model<serde_json::Value>` as the state model (app-owned, passed into renderer).
- [ ] Implement JSON Pointer helpers (`get_by_pointer`, `set_by_pointer`, `add/remove` as needed).
- [ ] Implement expression resolution for props:
  - [ ] `$state`
  - [ ] `$cond/$then/$else`
- [ ] Implement `visible` evaluation (same expression model).

## P1 — Repeat + item scope

- [ ] Implement `repeat: { statePath, key? }` rendering.
- [ ] Add repeat scope propagation (repeat item/index/base path).
- [ ] Implement `$item` and `$index` resolution.

## P2 — Bindings + forms

- [ ] Implement `$bindState` / `$bindItem` binding extraction (prop name → state path).
- [ ] Add a safe write-back API exposed to components (no direct arbitrary JSON mutation from spec).
- [ ] Implement minimal form patterns in `fret-genui-shadcn` (Input/Checkbox/Switch write-back).

## P2 — Actions

- [ ] Implement action param resolution (same expression model as props).
- [ ] Add action handler interface + adapters for common app patterns (navigation, clipboard, downloads).
- [ ] Add “confirm” support if desired (optional; keep app-owned).

## P3 — Schema export for LLM structured outputs

- [ ] Export a catalog schema (components + actions) for LLM constraints.
- [ ] Decide schema library (`schemars` vs custom).
- [ ] Add prompt helpers / docs for generating valid specs.

## Testing + gates

- [ ] Unit test matrix for validator and expression resolver.
- [ ] Add at least one end-to-end “spec renders a dashboard” test using `fret-ui-kit` test harness patterns.
- [ ] Add regression test for identity stability across repeat reorder (key field vs index fallback).

