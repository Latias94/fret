# GenUI Spec Rendering (json-render-inspired) v1 — TODO

Status: MVP landed (polish in progress)

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
- [x] Split `fret-genui-shadcn` resolver into modules (keep the component surface stable while letting implementations grow).

## P0 — Layout/typography building blocks (guardrailed)

Goal: improve spec expressiveness for spacing/typography without leaking policy into `crates/fret-ui`.

- [x] Add minimal layout props to `VStack`/`HStack`: `p/items/justify/wrap` (typed + deterministic mapping).
- [x] Extend stack layout props: `px/py`, `wFull/hFull`, `minW0/minH0` (and decide whether `flex1` belongs here).
- [x] Decide whether to add a generic `Box`/`Container` component (padding + sizing + alignment) vs growing per-component layout props (`Box` shipped in `ecosystem/fret-genui-shadcn`).
- [x] Add a small typography surface: `Text.variant` (enum) mapped deterministically in the shadcn resolver.
- [x] Update demo specs to use layout props (visual sanity gates).

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

## P2.5 — Validation loop (forms, app-owned)

- [x] Decide v1 validation contract: issues keyed by JSON Pointer paths (e.g. `/form/email`), plus a stable issue shape for UI rendering.
- [x] Add a small validation helper (ecosystem-first): `ValidationStateV1` + `validate_all()` (validator registry).
- [x] Wire `validate_all()` into submit-like actions via the app-owned executor (gate submit; record issues; keep UI policy app-owned). Demo: `apps/fret-examples/src/genui_demo.rs`.

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
- [x] Add a demo “playground” inspector (tabs for state/queue/issues/spec/schema/prompt/editor/stream).
- [x] Add an opt-in spec auto-fixer for common LLM mistakes (move `visible/on/repeat` out of `props`) and expose it in the demo.
- [x] Decide if/where to enable JSON repair (input boundary only; never for patch-only mode). Decision: defer; keep patch-only strict; consider opt-in at the app boundary if needed.

## Testing + gates

- [x] Unit tests for standard actions, expression/binding resolution, schema export, and catalog validation.
- [x] Add an end-to-end interaction test (press → queue → state via auto-apply).
- [x] Add at least one end-to-end “spec renders a small dashboard” test using `fret-ui-kit`/shadcn resolver.
- [x] Add regression test for identity stability across repeat reorder (key field vs index fallback).
- [x] Add a smoke spec that exercises layout props + responsive components (render + strict catalog validation; no visual assertions yet).

## Next (proposed order)

- [x] Add a generic `Box` component (padding + sizing) to avoid growing per-component layout props indefinitely.
- [x] Improve catalog prompt hints for layout patterns (Box boundaries, HStack + Input `flex1 + minW0`).
- [x] Normalize card content ergonomics in the spec examples (see Card ergonomics section and demo specs).
- [x] Add one more smoke spec focused on forms layout (labels, input widths, wrap, and alignment).

## Next (v1.1 polish — keep contracts stable)

Goal: reduce “demo confusion” and make the shadcn catalog output more consistently good-looking without changing the SpecV1 grammar.

- [x] Demo UX: make queue-only vs auto-apply mode impossible to miss (copy + affordances + “why didn’t my counter change?” hints). (2026-02-16)
- [ ] Demo spec ergonomics: add a small “Card as body” example and a “Box boundaries” example that LLMs can copy.
- [ ] Validation presentation: add a second spec snippet that renders multiple issues per field (repeat + filter) with consistent spacing.
- [ ] Catalog prompting: add one or two shadcn-specific notes that steer output away from “glued-to-edge” layouts (prefer `Box.p` + `VStack.gap`).
- [x] Gates: add an e2e smoke test that asserts validation issues appear/disappear deterministically (no visual assertions). (2026-02-16)

## Next (v1.1.x — dashboard parity, minimal contract risk)

Goal: reduce the gap vs `repo-ref/json-render/examples/dashboard` without changing SpecV1 grammar.

- [x] Core plumbing: pass child node metadata into `ComponentResolver` (at least: child type name + resolved props + rendered element) so resolver-level “macro components” can assemble compound UIs (Tabs/Accordion) without fragile, data-only fallbacks. (2026-02-16)
- [ ] Resolver parity (shadcn-backed):
  - [x] `Dialog` (trigger label + content children). (2026-02-16)
  - [x] `Drawer` (trigger label + side + content children). (2026-02-16)
  - [x] `Popover` (trigger label + content children). (2026-02-16)
  - [x] `Tooltip` (trigger child + content text). (2026-02-16)
  - [x] `DropdownMenu` (trigger label + items[] mapped to shadcn entries). (2026-02-16)
  - [x] `Avatar` (src/alt/fallback) — fallback-only today (no URL → ImageId yet). (2026-02-16)
  - [x] `Table` (data-driven: columns[] + data[] + optional rowActions[]). (2026-02-16)
  - [x] One compound: `Tabs` + `TabContent` and `Accordion` + `AccordionItem` (macro-assembled from child meta). (2026-02-16)
  - [x] json-render compat: `Stack`, `Heading`, `Pagination`, chart placeholders (`BarChart`, `LineChart`). (2026-02-16)
  - [ ] Follow-up: implement real chart rendering (or document as unsupported for v1).
  - [ ] Follow-up: add URL → `ImageId` ingestion for `Avatar.src` (policy + caching app-owned).

## Next (v1.2+ optional parity — keep SpecV1 stable)

These are “nice-to-have” parity items from upstream json-render, but not required for a shippable in-tree demo.

- [x] Docs: add an upstream package mapping section (`@json-render/*` → `fret-genui-*`). (2026-02-16)
- [ ] Validation ergonomics: optionally allow spec-authored `Input.checks` (json-render-style) to be collected into an app-owned validator registry (still app-owned policy for when/how to validate).
- [ ] Spec transforms: optionally add a `nested_to_flat` helper for human-authored nested trees (do not change the canonical flat SpecV1 shape).
