# Component Ecosystem State Integration v1 - TODO Tracker

Status: Draft
Last updated: 2026-02-06

This tracker covers the work described in:

- `docs/workstreams/component-ecosystem-state-integration-v1.md`

Related:

- `docs/workstreams/state-management-v1.md`
- `docs/workstreams/state-management-v1-extension-contract.md`
- `docs/workstreams/imui-ecosystem-facade-v1.md`

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `CSTATE-{area}-{nnn}`
- Areas:
  - `contract` (layer boundaries and ownership)
  - `api` (public ecosystem API changes)
  - `recipe` (shadcn/ui-kit adapter adoption)
  - `imui` (immediate-mode compatibility)
  - `demo` (golden-path adoption)
  - `gate` (lint/test/diag gates)
  - `docs` (guides/migration)

---

## M0 - Lock contract and scope

Exit criteria:

- primitives vs recipe-layer ownership is documented and agreed,
- invalidation responsibility split is explicit,
- fearless-refactor scope is bounded.

- [x] CSTATE-contract-000 Publish the integration contract workstream doc.
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1.md`
- [x] CSTATE-contract-001 Publish this tracker and link it from the integration doc.
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1.md`
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1-todo.md`
- [x] CSTATE-contract-002 Confirm feature naming policy for optional state adapters (`state-selector`, `state-query`, plus umbrella `state`).
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1.md` ("Decision log and remaining open questions")
- [ ] CSTATE-contract-003 Decide whether to host third-party adapter traits in `fret-ui-kit` or a dedicated helper crate.

---

## M1 - Primitives remain state-stack agnostic

Exit criteria:

- no new primitive public APIs require selector/query types,
- guidance for value/callback-first APIs is documented for contributors.

- [x] CSTATE-api-010 Audit `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`, `ecosystem/fret-ui-material3`, and `ecosystem/fret-imui` for direct selector/query coupling in primitive-level APIs.
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1.md` ("Initial audit and guardrail")
- [ ] CSTATE-api-011 Refactor any leaked coupling to optional adapters or app-side orchestration.
- [ ] CSTATE-api-012 Add contributor guidance: primitive APIs should not fetch or derive async state implicitly.

---

## M2 - Recipe-level optional adapters

Exit criteria:

- recipe layer has clear optional selector/query integration points,
- base recipe APIs remain usable without selector/query dependencies.

- [x] CSTATE-recipe-020 Define adapter module layout (recommended: recipe-local `state` modules).
  - Evidence: `ecosystem/fret-ui-shadcn/src/state.rs`
- [x] CSTATE-recipe-021 Add one selector-based recipe helper (e.g. computed counters/filters).
  - Evidence: `ecosystem/fret-ui-shadcn/src/state.rs` (`use_selector_badge`)
- [x] CSTATE-recipe-022 Add one query-based recipe helper (loading/success/error/invalidate flow).
  - Evidence: `ecosystem/fret-ui-shadcn/src/state.rs` (`query_status_badge`, `query_error_alert`)
- [ ] CSTATE-recipe-023 Ensure typed routing is used for dynamic row/item commands in adapted recipes.

---

## M3 - `imui` compatibility path

Exit criteria:

- immediate-mode wrappers can consume query/selector outputs via service-first integration,
- no hook-only requirement leaks into `imui` core abstractions.

- [x] CSTATE-imui-030 Add a short compatibility note to the `imui` ecosystem workstream.
  - Evidence: `docs/workstreams/imui-state-integration-v1.md`
- [x] CSTATE-imui-031 Add one sample showing host-side query/selector orchestration feeding immediate draws.
  - Evidence: `docs/workstreams/imui-state-integration-v1.md` ("Host-side orchestration pattern")
- [ ] CSTATE-imui-032 Verify typed command routing guidance is mirrored in immediate-mode examples.

---

## M4 - Golden-path demos and templates

Exit criteria:

- at least one official demo and one scaffold template reflect the final guidance.

- [x] CSTATE-demo-040 Update `todo_demo` narrative/docs to explicitly call out the three-layer state split.
  - Evidence: `docs/examples/todo-app-golden-path.md` ("Three-layer state split")
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
- [ ] CSTATE-demo-041 Add/refresh one recipe-heavy example that uses optional selector/query adapters.
- [ ] CSTATE-demo-042 Ensure scaffold docs point to state integration guidance and adapter policy.

---

## M5 - Gates and regression protection

Exit criteria:

- automated checks reduce contract drift,
- docs and examples stay aligned with the chosen model.

- [x] CSTATE-gate-050 Add a lightweight check preventing direct selector/query coupling in primitive contracts (allowlist for adapter modules).
  - Evidence: `tools/check_component_state_coupling.ps1`
- [ ] CSTATE-gate-051 Add nextest coverage for one selector adapter and one query adapter path.
- [ ] CSTATE-gate-052 Add one `fretboard diag` script for async-state + command routing interaction regression.
- [ ] CSTATE-docs-053 Add a migration note for ecosystem maintainers adopting optional state adapters.
- [x] CSTATE-docs-054 Add ecosystem-by-ecosystem selector/query recommendation matrix.
  - Evidence: `docs/workstreams/component-ecosystem-state-integration-v1.md` ("Ecosystem-by-ecosystem state recommendation matrix")

---

## Suggested execution order

1. M0 contract freeze
2. M1 primitive API audit/refactor
3. M2 recipe adapters
4. M3 imui compatibility note + sample
5. M4 demos/templates
6. M5 gates and migration notes