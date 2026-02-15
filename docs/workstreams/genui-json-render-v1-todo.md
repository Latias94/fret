# GenUI Spec Rendering (json-render-inspired) v1 — TODO

Status: In progress

Tracking doc for implementing the GenUI spec renderer and shadcn-backed catalog.

Design doc: `docs/workstreams/genui-json-render-v1.md`

## P0 — Contracts first (least-refactor path)

- [x] Define spec grammar types (flat map + root) in `ecosystem/fret-genui-core`.
- [x] Add a structural validator with stable issue codes (missing root, missing child, misplaced fields).
- [x] Decide spec versioning strategy (`schema_version` field, currently `1`).
- [x] Define action binding shape (`on.<event> -> { action, params }`) and event naming conventions.

## P0 — Rendering MVP (small catalog)

- [x] Implement `SpecRenderer` that renders `root` into `Vec<AnyElement>`.
- [x] Identity contract: each element key is rendered under `cx.keyed(key, ...)`.
- [x] Implement event→action invocation emission into an app-owned queue model.
- [x] Add `fret-genui-shadcn` catalog with a conservative baseline component set (Card/Text/Button/Badge/Input/Switch/Stacks/Separator/ScrollArea).

## P1 — State + expressions

- [x] Introduce `Model<serde_json::Value>` as the state model (app-owned, passed into renderer).
- [x] Implement JSON Pointer helpers (`get`, `set`, basic array ops via standard actions).
- [x] Implement expression resolution for props (`$state`, `$cond/$then/$else`).
- [x] Implement `visible` evaluation (visibility condition grammar + evaluator).

## P1 — Repeat + item scope

- [x] Implement `repeat: { statePath, key? }` rendering.
- [x] Add repeat scope propagation (item/index/base path).
- [x] Implement `$item` and `$index` resolution.

## P2 — Bindings + forms

- [x] Implement `$bindState` / `$bindItem` binding extraction (prop name → state path).
- [x] Keep writes app-owned: components emit `setState` into the queue (or fall back to direct apply when no queue exists).
- [x] Implement minimal form patterns in `fret-genui-shadcn` (Input + Switch write-back).

## P2 — Actions

- [x] Implement action param resolution (including repeat-scoped path semantics for `$item/$bindItem/$index`).
- [x] Add opt-in `GenUiRuntime.auto_apply_standard_actions` for demos/simple apps (still emits into the queue).
- [x] Add a first-class action handler interface + executor (`GenUiActionExecutorV1`) with standard actions and portable effect actions (`openUrl`, `clipboardSetText`).
- [x] Add “confirm/onSuccess/onError” executor helpers (opt-in confirm policy; best-effort chaining; bounded recursion).

## P3 — Schema export for LLM structured outputs

- [x] Export a catalog-derived JSON Schema for LLM constraints.
- [x] Decide schema strategy: custom JSON Schema export (keep portable; no `schemars` dependency in v1).
- [x] Add system prompt export from catalog.
- [x] Add typed catalog guardrails for prop/param values (primitive types + enums + nullable + dynamic expressions).
- [x] Expand catalog typing: object/array/oneOf + required/default metadata.
- [x] Add SpecStream compiler (JSONL RFC6902 patch stream → in-progress spec JSON).

## P4 — Adaptive layouts (strategy layer)

- [x] Add `ResponsiveGrid` to `fret-genui-shadcn` (container-query driven).
- [x] Add `ResponsiveStack` (switches between HStack/VStack via queries).
- [x] Add a resize-driven demo spec section to validate behavior by eye.

## P4 — LLM ingest utilities (strategy/boundary layer)

- [x] Add mixed-stream parser utilities (text + JSONL patches) similar to `pipeJsonRender`.
- [ ] Decide if/where to enable JSON repair (input boundary only; never for patch-only mode).

## Testing + gates

- [x] Unit tests for standard actions, expression/binding resolution, schema export, and catalog validation.
- [x] Add an end-to-end interaction test (press → queue → state via auto-apply).
- [x] Add at least one end-to-end “spec renders a small dashboard” test using `fret-ui-kit`/shadcn resolver.
- [x] Add regression test for identity stability across repeat reorder (key field vs index fallback).
