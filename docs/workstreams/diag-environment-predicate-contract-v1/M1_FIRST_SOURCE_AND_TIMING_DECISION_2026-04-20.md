# M1 First Source And Timing Decision - 2026-04-20

Status: active decision note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `BASELINE_AUDIT_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-diag/src/transport/fs.rs`
- `crates/fret-diag/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`

## Purpose

This note freezes the first concrete source choice for future diagnostics environment predicates and
the timing rule that blocks a premature manifest syntax.

The goal is to avoid two common failure modes:

1. inventing `requires_environment` syntax before the repo knows when the source is actually
   available,
2. or overloading `capabilities.json` with host-environment facts that are not capabilities.

## Findings

### 1) The first real candidate source is `host.monitor_topology`

Among the currently visible environment lanes, the first source that actually supports a real
campaign selection question is the host monitor inventory exported as
`bundle.json.env.monitor_topology`.

Why this is the first candidate:

- it already exists as a runner-owned diagnostics environment fingerprint,
- it answers a real scheduling question that capabilities cannot express honestly,
- and it is the exact missing source behind the mixed-DPI automation follow-on.

### 2) Current campaign preflight happens before launch

`crates/fret-diag/src/diag_campaign.rs` currently runs capability preflight immediately after the
campaign start-plan writes directories/metadata and before any campaign item execution begins.

That means a fresh tool-launched run does not yet have a runtime-populated host environment source
available at this point.

### 3) Current preflight provenance is capability-only and filesystem-first

The existing preflight path reads `capabilities.json` through
`read_filesystem_capabilities_with_provenance(...)` and records `capability_source`.

That contract is useful, but it is specifically about capabilities and their provenance.

It is not the right place to smuggle host monitor topology or future environment fingerprints.

## Decision

From this point forward:

1. The first candidate source for environment predicates is `host.monitor_topology`.
2. Do not freeze `requires_environment` syntax yet.
3. Do not overload `capabilities.json` with environment fingerprints.
4. Before any manifest syntax lands, diagnostics must first name how an environment source is
   published and when it is available:
   - preflight from a persisted filesystem sidecar,
   - preflight from a transport/session handshake,
   - launch-time after the child starts,
   - or post-run evidence only.
5. A source that is only available as post-run evidence must not be treated as a truthful
   campaign-selection predicate.

## Immediate consequence

This lane stays in contract-design mode.

The next implementation-worthy slice is not `requires_environment`.
The next implementation-worthy slice is a separate environment-source provenance lane that can sit
parallel to `capabilities.json` and `capability_source`.

That lane should answer:

- what the first sidecar or transport payload is called,
- which source ids it can carry,
- how provenance is reported,
- and which availability class each source belongs to.

## Explicit non-decision

This note does not freeze:

- the final manifest JSON key,
- a general boolean/expression language,
- or whether the first consumer should run at campaign preflight or at launch-time.

Those should only be chosen after the environment-source provenance path exists.
