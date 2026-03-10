---
title: Non-Filesystem Capability Source v1
status: draft
date: 2026-03-09
scope: diagnostics, capability source, transport contracts
---

# Non-Filesystem Capability Source v1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`

## Purpose

This note answers one narrow contract question:

- what should "capability source" mean when diagnostics does not obtain capabilities from a
  filesystem path?

The goal is to prevent consumers from assuming that every capability decision must resolve to a
local path.

## Problem statement

The current diagnostics behavior is already aligned for filesystem-first execution:

- capability loading reuses the shared filesystem loader,
- campaign preflight and `diag doctor` report the same resolved source path,
- policy-skip evidence can point at `check.capabilities.json`.

That is correct for filesystem-backed runs, but it does not define what should happen when a future
transport or session provides capabilities directly.

Without an explicit answer, consumers will drift toward one of two bad outcomes:

- fabricating fake paths for non-filesystem sessions,
- or silently dropping provenance once a real path does not exist.

## Current filesystem-aligned baseline

Today the repo has one stable filesystem-oriented contract:

- a resolved capability source path may be reported when capabilities were loaded from
  `capabilities.json`,
- campaign outputs may also report `capabilities_check_path` for the local preflight artifact,
- maintainers may use those paths for drill-down, inspection, and debugging.

This note does not replace that behavior.

## Decision

The first non-filesystem extension should be **additive**.

Specifically:

- keep the existing filesystem path reporting semantics,
- do not reinterpret a path field to carry non-path identifiers,
- do not require non-filesystem transports to invent a synthetic file location,
- add a future-facing provenance object when a consumer needs non-filesystem capability identity.

## Proposed additive provenance shape

When diagnostics needs to report capability provenance beyond the filesystem case, the contract
should grow around a small additive object with semantics like:

- `kind`: `filesystem | transport_session | inline | unknown`
- `path`: optional filesystem path, only when `kind = filesystem`
- `label`: optional human-readable source label
- `transport`: optional transport identity such as `filesystem` or `devtools_ws`
- `session_id`: optional transport/session identifier

This note intentionally defines direction rather than a mandatory immediate schema change.

The important rule is semantic separation:

- `path` means a real filesystem path,
- `label` means a display-oriented source description,
- `transport` / `session_id` mean transport provenance,
- absence of `path` in non-filesystem cases is valid and should not be treated as missing data.

## Consumer rules

Consumers should interpret capability provenance as follows:

- filesystem-backed runs may continue to show and copy the resolved source path,
- non-filesystem runs should show `label` and/or transport identity when available,
- non-filesystem runs must not fabricate a pseudo-path just to match old UI expectations,
- `capabilities_check_path` remains a separate artifact-path concept and should not be overloaded
  to stand in for the original capability source.

## Non-goals

This note does not require:

- an immediate schema migration across all diagnostics payloads,
- a new probing subsystem,
- replacement of the current string-based filesystem source reporting,
- retrofitting every existing consumer before a real non-filesystem producer exists.

## Adoption guidance

Until a real non-filesystem producer lands:

- treat the current filesystem path reporting as the active shipped contract,
- use this note as the contract direction for future WebSocket/session-backed capability export,
- keep future changes additive so existing filesystem consumers remain valid.
