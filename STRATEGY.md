---
name: Fret
last_updated: 2026-07-12
---

# Fret Strategy

## Target problem

Rust teams building desktop-first productivity and editor-grade applications need GPU-native UI foundations that can scale from ordinary app screens to docking, multi-window workflows, rich text, and embedded viewports. Fret has substantial mechanisms, but its public release path, authoring contract, and separately maintained, independently versioned consumer evidence are not yet coherent enough for teams to adopt it as a general-purpose framework.

## Our approach

Prove the framework through one separately maintained, independently versioned, published-crate-only workbench before expanding horizontally. Preserve Fret's GPU-first runtime and mechanism/policy boundaries, and let release closure and real consumer friction drive authoring, crate, and governance refactors.

## Who it's for

**Primary:** Rust application teams building desktop productivity tools and editor-grade workbenches - they are hiring Fret to ship GPU-native native/web interfaces without building text, focus, overlays, docking, rendering, and diagnostics foundations themselves.

## Key metrics

- **Published-consumer closure** - A separate consumer repository builds from crates.io without path, Git, workspace, or patch dependencies; measured in that repository's CI.
- **Public release health** - Release closure has zero internal dependency issues, taught entry crates build on docs.rs, and every public scaffold builds against the released versions; measured by release CI and registry records.
- **Real-workbench completion** - The external workbench turns a versioned project snapshot and checklist into a persisted release-readiness report through editing, asynchronous validation, table triage, settings, docking, commands, and accessibility semantics; measured by its diagnostics suite.
- **Upgrade and authoring cost** - One cold registry-version upgrade stays within the fixed time, call-site, build-regression, and unresolved-friction budgets in the active plan, with zero raw framework seams in ordinary application modules; measured in the consumer evidence ledger.
- **Runtime evidence** - Native and browser platform runs publish p50, p95, maximum frame time, worst-frame attribution, and platform-specific failures; measured from diagnostics bundles on named reference environments.

## Tracks

### Release and public consumption

Close the complete publish graph, align versions, make docs.rs and public scaffolds reliable, and keep registry-only consumption continuously tested.

_Why it serves the approach:_ A framework cannot be validated externally while its supported package graph or generated starter is unresolved.

### External workbench validation

Build and maintain one independently versioned workbench outside the monorepo using only released crates and public tooling.

_Why it serves the approach:_ An independent application exposes composition, upgrade, platform, and maintenance costs that in-repo harnesses cannot prove.

### Consumer-driven convergence

Resolve authoring language, capability boundaries, shallow facades, and crate ownership only from repeated workbench friction or release constraints.

_Why it serves the approach:_ Refactoring against observed call sites improves the framework without reopening proven runtime contracts or inventing another abstraction layer.

### Governance and evidence

Maintain an active-decision view of ADRs and a compact evidence ledger for release, API, upgrade, platform, and performance claims.

_Why it serves the approach:_ A small current decision surface keeps the validation result auditable while preserving historical contracts.

## Milestones

- **2026-10-10** - Decide whether the external validation passes, receives one bounded extension, or requires Fret to narrow the claims that failed, up to an internal GPU UI research platform when the public framework loop itself is not maintainable.

## Not working on

- New crates, broad features, or component families that are not required by the release closure, external workbench, regressions, security, or a hard contract.
- Rewriting `UiTree`, flattening Fret into a GPUI-style root, or replacing the proven frame/runtime architecture without falsifying evidence.
- Merging or splitting crates based only on line count or single-consumer status rather than version contracts, target isolation, or demonstrated ownership friction.
- Adding micro-ADRs for local implementation choices; new ADRs are reserved for hard-to-change cross-crate contracts.

## Marketing

**One-liner:** Fret is an experimental GPU-first Rust application UI framework being validated through a published-crate-only external workbench.

**Key message:** The mechanisms are real, but general-purpose product value is not treated as proven until release, external consumption, upgrade, platform, and performance gates close.
