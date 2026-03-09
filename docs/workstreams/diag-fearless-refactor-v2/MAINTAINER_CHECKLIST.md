---
title: Diagnostics Maintainer Checklist
status: draft
date: 2026-03-09
scope: diagnostics, maintainer, checklist, policy-skip, evidence
---

# Diagnostics Maintainer Checklist

Status: Draft

Purpose:

- give maintainers one short "what to check before landing" note,
- keep new diagnostics work aligned with the shared contract,
- make policy-skip outcomes readable without re-opening multiple notes.

## 1) First-open reading order

When investigating one diagnostics run, open files in this order:

1. aggregate human summary or dashboard view,
2. `regression.index.json`,
3. the selected `regression.summary.json`,
4. the best available evidence artifact for the selected non-passing item.

Practical rule:

- do not start from raw `bundle.json` unless the bounded artifacts are insufficient.

## 2) How to interpret policy-skip results

Treat these fields as one contract slice:

- `status = skipped_policy`
- `reason_code = capability.missing`
- `campaigns_skipped_policy`
- `capabilities_check_path`

Meaning:

- `skipped_policy` means the item did not execute because policy/capability checks said it should
  not run.
- `capability.missing` means the skip reason is missing runtime or environment capability, not an
  assertion failure inside the item itself.
- `campaigns_skipped_policy` is the batch-level counter for that bucket; do not merge it back into
  generic failure counts when presenting status.
- `capabilities_check_path` points at the campaign-local capability check artifact that explains
  why the item was skipped.

Do not interpret this state as:

- deterministic item failure,
- flaky failure,
- timeout,
- or "no evidence available".

## 3) Which artifact to open first

For a selected non-passing summary item, prefer evidence in this order:

1. `bundle_dir` when the item actually executed and produced bundle artifacts,
2. `capabilities_check_path` when the item is `skipped_policy`,
3. summary/index JSON when only aggregate context exists.

Consumer rule:

- GUI, MCP, CLI, and maintainer docs should all preserve this distinction.

## 4) Landing checklist for a new diagnostics feature

Before landing a new diagnostics-facing change, answer these questions:

### Ownership

- Which layer owns the change?
  - runtime/protocol/aggregate artifact contract in `crates/*`,
  - consumer/presentation behavior in CLI, GUI, or MCP,
  - app- or recipe-local workflow polish outside the shared contract.

### Gate

- What regression protection exists?
  - Rust unit/integration test,
  - diag script/suite,
  - perf gate/baseline,
  - or a bounded consumer test for shared projections.

### Evidence

- What evidence is left behind for another maintainer?
  - repro command,
  - artifact path(s),
  - exact test name or diag suite name,
  - and, when relevant, packed bundle or capability-check artifact.

### Docs

- Which note must be updated?
  - contract/workstream note,
  - maintainer-facing checklist or status note,
  - CLI/GUI/MCP wording if user-visible terminology changes.

## 5) Minimum deliverables

Prefer leaving this 3-pack:

- one repro,
- one gate,
- one evidence anchor set.

For contract-affecting changes, also leave:

- one updated workstream/ADR note,
- one consumer test or wording check if presentation changed.

## 6) Red flags

Stop and re-evaluate if a change does any of the following:

- collapses `skipped_policy` back into generic failure wording,
- treats raw `*json` text-holder names as proof of persisted-contract drift,
- requires a GUI-only or MCP-only parser for data already shaped by shared diagnostics code,
- adds a new status or reason-code family when an existing normalized bucket already fits,
- widens the contract before there is a concrete consumer.

## 7) Short version

If only one reminder is needed:

- preserve the shared vocabulary, add one gate, and leave the best available evidence artifact
  behind.
