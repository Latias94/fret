---
title: Capability Provenance Minimal Implementation v1
status: draft
date: 2026-03-09
scope: diagnostics, capability provenance, implementation plan
---

# Capability Provenance Minimal Implementation v1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`

## Purpose

This note turns the capability-provenance direction into one bounded implementation plan.

It does not require immediate code changes. It defines the smallest additive rollout that keeps
filesystem consumers stable while making room for non-filesystem producers later.

## Current output map

Today capability provenance is exposed through multiple filesystem-oriented fields:

- `diag doctor` emits top-level `capabilities_path` plus the `capabilities` payload,
- campaign capability-preflight summary items write
  `evidence.extra.capabilities_source_path`,
- the same summary items also write
  `source.metadata.capabilities_source_path`,
- `campaign.result.json` aggregate currently exposes `capabilities_check_path`,
- DevTools `Regression` drill-down currently consumes `capabilities_check_path`,
  not a richer provenance object.

This means the repo already has useful path visibility, but it does not yet have one normalized
shape for provenance.

## Minimal additive contract

The first implementation should add one optional normalized object:

- `capability_source`

Recommended shape:

- `kind`: `filesystem | transport_session | inline | unknown`
- `path`: optional filesystem path
- `label`: optional human-readable source label
- `transport`: optional transport identifier
- `session_id`: optional session identifier

Rules:

- keep existing path fields during the compatibility window,
- only set `path` when it is a real filesystem path,
- do not backfill fake path strings for non-filesystem sessions.

## Minimal rollout order

### Phase 1 — Internal normalization only

Add one internal helper or struct in `fret-diag` that can represent capability provenance before
JSON emission.

Target outcomes:

- campaign preflight and `diag doctor` can build the same provenance value,
- filesystem cases still populate a real `path`,
- non-filesystem cases can later populate `kind`, `label`, `transport`, and `session_id`
  without reshaping every consumer again.

This phase is internal-only and should not remove any existing JSON field.

### Phase 2 — Additive JSON emission on existing surfaces

Once the internal shape exists, extend current JSON outputs additively.

Recommended first surfaces:

- `diag doctor`
  - keep `capabilities_path`,
  - add top-level `capability_source`,
- campaign capability-preflight summary item
  - keep `evidence.extra.capabilities_source_path`,
  - add `evidence.extra.capability_source`,
  - keep `source.metadata.capabilities_source_path`,
  - add `source.metadata.capability_source`,
- `campaign.result.json` aggregate
  - keep `capabilities_check_path`,
  - add `capability_source` when preflight used one resolved source.

This gives one canonical machine-readable shape without breaking path-based consumers.

### Phase 3 — Consumer adoption

After additive emission exists, consumers should migrate in this order:

- CLI and human-readable diagnostics output:
  - prefer `path` for filesystem cases,
  - otherwise show `label` or `transport/session_id`,
- DevTools GUI:
  - keep reading `capabilities_check_path`,
  - prefer `capability_source` when present for richer drill-down text,
- MCP / machine-readable consumers:
  - prefer `capability_source`,
  - keep falling back to legacy path fields during the compatibility window.

## Field-level recommendations

### `diag doctor`

Recommended additive output:

- keep:
  - `capabilities_path`
- add:
  - `capability_source`

Reason:

- `diag doctor` already has a top-level path field and is the easiest place to establish the new
  normalized object.

### Campaign summary item evidence

Recommended additive output:

- keep:
  - `evidence.extra.capabilities_source_path`
  - `source.metadata.capabilities_source_path`
- add:
  - `evidence.extra.capability_source`
  - `source.metadata.capability_source`

Reason:

- this is the actual per-decision provenance surface,
- existing tests already assert the path string here,
- additive object placement lets DevTools/MCP adopt richer semantics without waiting for a larger
  summary schema redesign.

### Campaign aggregate/result payloads

Recommended additive output:

- keep:
  - `aggregate.capabilities_check_path`
- add:
  - `aggregate.capability_source`

Reason:

- maintainers often start from `campaign.result.json`,
- aggregate visibility prevents a consumer from opening item-level payloads just to determine where
  the capability decision came from.

## Compatibility policy

The intended compatibility sequence is:

1. add `capability_source`,
2. keep all existing path fields,
3. migrate consumers to prefer the object,
4. only consider field retirement after all active producers and consumers no longer depend on the
   legacy path-only view.

No retirement should be attempted in the same change that introduces the object.

## Suggested first code touch points

If implementation starts, the smallest touch points are:

- `crates/fret-diag/src/lib.rs`
  - shared capability loader / normalization helpers,
- `crates/fret-diag/src/commands/doctor.rs`
  - doctor JSON emission,
- `crates/fret-diag/src/diag_campaign.rs`
  - capability-preflight evidence emission and aggregate result JSON,
- `apps/fret-devtools/src/native.rs`
  - regression drill-down display fallback order,
- `apps/fret-devtools-mcp/src/native.rs`
  - dashboard/resource wording if richer provenance becomes visible there.

## Regression gates to require

When code work begins, the minimum regression set should include:

- one `diag doctor` JSON test that asserts legacy `capabilities_path` plus additive
  `capability_source`,
- one campaign preflight test that asserts:
  - `capabilities_check_path`,
  - legacy `capabilities_source_path`,
  - additive `capability_source`,
- one consumer test for DevTools or MCP proving that missing `path` does not collapse the
  provenance display.

## Non-goals

This plan does not require:

- immediate non-filesystem producer support,
- a summary schema version bump just for provenance,
- replacing `capabilities_check_path`,
- retirement of `capabilities_source_path` in this phase.
