---
title: Component Parity Fact Harness v1
status: active
date: 2026-05-25
scope: shadcn, material3, parity-discovery, diagnostics, agent-repair
---

# Component Parity Fact Harness v1

This lane makes component parity repair deterministic enough for Codex-style agents. The target is
not another screenshot checklist. The target is a fact packet that states:

- what upstream source owns the behavior,
- which Fret recipe, policy, mechanism, app-demo, or diagnostics layer owns the implementation,
- which evidence proves the current state,
- which rows need repair, hardening, or promotion into a gate,
- and which residual risks are still not measurable.

The first pilot is Button Group because it already exposed the exact failure mode this lane should
solve: visually small layout drift in self-drawn controls can be caused by recipe chrome, docs-path
composition, layout mechanism, or diagnostics selector gaps. A future Material 3 port should use
the same loop, only with Material spec/MUI/Compose source adapters instead of shadcn registry and
Radix/Base UI references.

## Boundary

`crates/fret-ui` stays the mechanism/contract layer. This lane may identify mechanism defects, but
it must not move shadcn, Material, Radix, or Base UI policy into `fret-ui`.

Layer ownership remains:

- `crates/fret-ui`: layout vocabulary, focus, hit-test, semantics, overlay routing, diagnostics
  data surfaces.
- `ecosystem/fret-ui-kit`: headless policy and reusable interaction infrastructure.
- `ecosystem/fret-ui-shadcn`: shadcn recipe taxonomy, slot chrome, default sizing, and example
  composition.
- `ecosystem/fret-ui-material3`: Material recipe taxonomy, token/state layers, and example
  composition.
- `apps/fret-ui-gallery`: teaching examples, stable `test_id`s, scroll/open choreography, and
  diagnostics entry points.
- `tools/parity-discovery`: source-to-Fret fact joins, report generation, triage, and agent repair
  packets.

## Source Precedence

For shadcn:

- Visual recipe truth: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*` and the exact docs-path
  examples under `repo-ref/ui/apps/v4/registry/new-york-v4/examples/*`.
- Semantics and interaction truth: APG outcomes, Radix primitives, then Base UI when translating DOM
  assumptions into a self-drawn runtime.
- Diagnostics truth: Fret layout sidecars, `bundle.schema2`, screenshots, and AI packets from
  `fretboard-dev diag`.

For Material:

- Spec truth: Material 3 component spec and token/state descriptions.
- Runtime references: MUI Material/Base UI and Compose Material3 as parallel implementation
  references.
- Fret ownership: Material recipe/policy belongs outside `crates/fret-ui`; only reusable mechanism
  gaps promote inward.

## Harness Shape

The durable unit is a component fact report:

1. Source facts: curated or extracted upstream facts, source refs, DOM/golden targets, and source
   context metadata such as viewport, theme, density, and mode.
2. Fret facts: source refs, stable `test_id`s, layout sidecar nodes, bundle semantics nodes, and
   diagnostics bundle anchors.
3. Checks: structured predicates with owner, layer, confidence, and promotion target.
4. Agent packet: derived repair queue, hardening queue, gate queue, source refs, Fret wiring, and
   evidence contexts.
5. Promotion: mismatches become component fixtures, diag scripts, or mechanism harness cases based
   on owner/layer, not on where the symptom was seen.

## Button Group Pilot

The pilot reuses existing shadcn v1 Button Group fixtures and the latest UI Gallery diagnostics
seed. Its purpose is not to re-fix Button Group. Its purpose is to prove a future repair agent can
open one JSON report and know:

- Button Group is currently regression-locked on the known seed rows.
- Low/medium confidence rows still need hardening into more mechanical source-fact extraction.
- Source truth is shadcn `new-york-v4`, not a Fret screenshot.
- Fret evidence is tied to stable gallery `test_id`s and layout/bundle sidecars.
- Any future mismatch should already carry an owner/layer and a promotion target.

## No-Fear Refactor Queue

The long-term refactor is allowed to remove obsolete parity paths once the packet model proves it
can replace them. The intended migration order is:

1. Keep `tools/parity-discovery` as the first implementation because it already owns fixtures and
   reports.
2. Add `agent_packet` to every generated report.
3. Promote repeated packet schema into a reusable Python module or Rust crate only after two source
   families, shadcn and Material, share it.
4. Add live source adapters for upstream DOM/CSS and Material references.
5. Add paint/text/focus/semantics fact families beyond current layout/bounds metrics.
6. Delete stale ad hoc mismatch docs after their facts are represented as report rows and gates.

Do not start by creating a generic abstraction crate. The first abstraction boundary is the JSON
fact shape, because it is reviewable, easy for agents to consume, and already compatible with the
diagnostics archive.
