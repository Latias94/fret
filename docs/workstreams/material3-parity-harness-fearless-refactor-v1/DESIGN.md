# Material 3 Parity Harness Fearless Refactor v1

Status: Closed
Last updated: 2026-05-27

Status note (2026-05-27): this lane is closed. It shipped the Material suite manifest, Button /
Select / Switch packet baseline, reusable Material test support extraction, focused Select behavior
test target, and gated Select/Switch automation-surface contract. Future Material parity expansion
should start as a narrower follow-on for the next component packet or for the known controls golden
drift.

## Why This Lane Exists

`ecosystem/fret-ui-material3` now has a broad component surface, Material foundation modules, token
import/audit tools, gallery snippets, headless goldens, and diagnostics scripts. The missing piece is
not another isolated component port. The missing piece is a durable Material parity harness that can
turn upstream Material facts into Fret evidence, classify defects by layer, and hand bounded repair
tasks to agents without relying on screenshot interpretation.

The shadcn parity harness already proved this shape for a different design system:

```text
upstream source facts
-> upstream DOM / extracted evidence
-> Fret bundle, layout, semantics, and paint facts
-> mapped parts
-> mismatch or hardening report
-> repair, hardening, and gate queues
```

This lane adapts that workflow to Material 3, using Material spec, Compose Material3, MUI Material UI,
and Base UI as axis-specific references.

## Relevant Authority

- Layering:
  - `docs/architecture.md`
  - `docs/runtime-contract-matrix.md`
  - `docs/reference-stack-ui-behavior.md`
- Material work:
  - `docs/workstreams/material3/material3-refactor-plan.md`
  - `docs/workstreams/material3/material3-next-wave-v2.md`
  - `docs/workstreams/material3-expressive-alignment-v1/material3-expressive-alignment-v1-refactor-plan.md`
  - `docs/workstreams/material3-expressive-alignment-v1/material3-expressive-alignment-v1-todo.md`
- Existing parity harnesses:
  - `docs/workstreams/shadcn-parity-discovery-harness-v1/DESIGN.md`
  - `docs/workstreams/component-parity-fact-harness-v1/DESIGN.md`
  - `docs/workstreams/component-parity-fact-harness-v1/CLOSEOUT_AUDIT_2026-05-25.md`
  - `tools/parity-discovery/README.md`
- Existing Material adapter pilot:
  - `tools/parity-discovery/fixtures/material3_button_adapter_v1.json`
  - `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
- Upstream mirrors:
  - `F:/SourceCodes/Rust/fret/repo-ref/material-ui`
  - `F:/SourceCodes/Rust/fret/repo-ref/compose-multiplatform-core`
  - `F:/SourceCodes/Rust/fret/repo-ref/material-web`
  - `F:/SourceCodes/Rust/fret/repo-ref/base-ui`

## Problem

Material 3 is ahead in component count but behind in refactor workflow maturity:

- The first Material parity adapter exists only for Button; there is no Material suite manifest,
  coverage matrix, or reusable component packet order.
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs` is a 14.6k-line god test that mixes host
  scaffolding, snapshot serialization, component logic, interaction tests, and golden suites.
- Existing Material docs mark earlier MVP/foundation work as complete, while active risks now live
  across follow-up TODOs, diagnostics scripts, and generated artifacts.
- Some components expose stable `test_id`s, but there is no crate-wide contract that says which
  roots, chrome nodes, popup/listbox nodes, options, indicators, and secondary affordances must be
  automatable.
- Material source precedence must be axis-based. MUI is useful for web defaults and DOM behavior,
  Compose is better for toolkit interaction/state/motion, and the Material spec owns UX intent and
  token semantics. A single linear "copy this implementation" rule would create wrong ownership.
- The current workflow can prove individual component outcomes, but it does not yet generate a
  repair queue that says whether a finding belongs to Material recipe code, shared Material
  foundation, `fret-ui-kit` policy, `crates/fret-ui` mechanisms, gallery snippets, or diagnostics.

## Target State

This lane closes when Material 3 has the same operational loop that shadcn now has:

- A Material parity suite manifest exists and can regenerate current reports from existing evidence
  when historical diag sidecars are unavailable.
- At least three high-risk Material surfaces are represented as component fact packets:
  - Button as the existing low-risk pilot,
  - one field-family overlay surface (`Select`, `Autocomplete`, or `ExposedDropdown`),
  - one motion/interaction-heavy control (`Switch`, `Tabs`, or navigation item).
- Each packet carries:
  - upstream refs split by axis,
  - stable Fret `test_id` anchors,
  - Fret bundle/layout/semantics/paint evidence,
  - owner/layer classification,
  - repair, hardening, and gate queues.
- Material test infrastructure is split so reusable host/snapshot helpers live in support modules,
  while component suites remain focused and independently runnable.
- Stable Material automation surfaces are documented and enforced by at least one gate.
- The workstream records which issues belong to `ecosystem/fret-ui-material3`, which belong to
  `ecosystem/fret-ui-kit`, and which genuinely require `crates/*` mechanism work.

## In Scope

- Build on `tools/parity-discovery` instead of starting a new abstraction crate.
- Add Material-specific suite/fixture/report coverage around the existing shared packet shape.
- Audit and harden `test_id` conventions for Material components and gallery snippets.
- Split Material test harness support from the current monolithic `radio_alignment.rs`.
- Add or promote diagnostics scripts only when they prove interaction, focus, overlay, motion, or
  a11y outcomes that headless tests cannot cover.
- Refactor stale Material component/foundation duplication only when an evidence row or audit note
  shows the ownership boundary is wrong.

## Out Of Scope

- Full Material component library completeness.
- 1:1 API compatibility with Compose Material3, MUI, or Material Web.
- Moving Material policy into `crates/fret-ui`.
- A generic parity Rust crate before the JSON packet shape survives broader Material coverage.
- Screenshot-only parity decisions.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Material parity should reuse the existing `tools/parity-discovery` packet format first. | High | `component-parity-fact-harness-v1` closed with Button Group and Material Button packets. | If the format does not fit Material, create a narrow schema extension before creating a crate. |
| Material source precedence must be axis-based. | High | Material source-alignment guidance and existing Button adapter refs. | If one source becomes authoritative for a component, record that locally in the fixture. |
| The next highest ROI surface is field-family overlay parity. | High | Field-family checklist, Select/Autocomplete/ExposedDropdown diagnostics, and prior shadcn Select lessons. | If another component blocks harness generalization sooner, swap the first proof while keeping the same packet rules. |
| Test harness modularization is needed before broadening suites. | High | `radio_alignment.rs` is 14,594 lines and owns unrelated scaffolding plus many suites. | If extraction causes churn, split support modules first and defer component file splitting. |
| Core mechanism changes should be rare. | High | ADR 0066/0067 and existing Material foundation docs. | If a report identifies a real mechanism defect, split a focused mechanism follow-on with its own gates. |

## Architecture Direction

Layer ownership stays strict:

- `crates/fret-ui`: mechanism contracts such as layout, focus, semantics, hit testing, overlay roots,
  diagnostics data surfaces, and paint primitives.
- `ecosystem/fret-ui-kit`: shared headless policy such as roving focus, dismissal, focus restore,
  typeahead, and overlay policy when more than one design system should use it.
- `ecosystem/fret-ui-material3`: Material foundation and recipes: tokens, state layers, ripple,
  motion scheme, elevation, interactive size, component chrome, and Material-specific semantics
  composition.
- `apps/fret-ui-gallery`: teaching pages, stable automation surfaces, open/scroll choreography, and
  reproducible diagnostics entry points.
- `tools/parity-discovery`: fact joining, mismatch reporting, agent packet derivation, and suite
  summaries.

The refactor should proceed from evidence outward:

1. Inventory Material surfaces and stable selectors.
2. Promote the existing Button adapter into a Material suite manifest.
3. Add one field-family overlay packet and one interaction-heavy packet.
4. Use report queues to decide whether to change recipe, foundation, policy, mechanism, gallery, or
   diagnostics.
5. Refactor test/harness structure only in ways that preserve or improve gates.

## Closeout Condition

This lane can close when:

- the Material suite can regenerate current reports and an agent packet summary,
- at least three Material component packets exist with current evidence,
- the Material `test_id` contract is documented and gated,
- the reusable test harness support is no longer trapped in one god test file,
- fresh targeted gates pass,
- and any remaining component or mechanism work is split into narrower follow-ons.
