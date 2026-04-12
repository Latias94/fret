# P3 Bounded Multi-Window Parity Package - 2026-04-12

Status: focused P3 gate decision / bounded runner-owned parity package freeze

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json`
- `tools/diag-scripts/suites/docking-arbitration/suite.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- `docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md`

## Purpose

After freezing the P3 runner-owned checklist, this lane still needed one bounded answer:

> what is the smallest repo-owned parity package that explicitly names hovered-window, peek-behind,
> transparent payload, and mixed-DPI follow-drag expectations without bloating the generic docking
> smoke entry or reopening `imui` as the wrong owner?

This note freezes that package.

## Audited evidence

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `tools/diag-scripts/suites/diag-hardening-smoke-docking/README.md`
- `tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json`
- `tools/diag-scripts/suites/docking-arbitration/suite.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
- `tools/diag-campaigns/README.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- `docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md`

## Assumptions-first resume set

1. Confident: `diag-hardening-smoke-docking` should stay small and general-purpose instead of
   absorbing every P3 runner stress script.
   Evidence:
   - the suite README already frames it as a small post-merge docking smoke,
   - `docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md` explicitly says to
     keep that smoke suite small while promoting broader multi-window gates elsewhere.
   Consequence if wrong:
   - this lane would silently widen a generic docking smoke entry and make future maintenance
     harder.
2. Confident: the four P3 expectations already have reusable script-level evidence and do not need
   fresh script authoring in this slice.
   Evidence:
   - overlapped-window hovered selection:
     `docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
   - peek-behind under the moving window:
     `docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
   - transparent payload overlap behavior:
     `docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
   - mixed-DPI / large-coordinate follow-drag stress:
     `docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
   Consequence if wrong:
   - this slice would turn into implementation-heavy script authoring instead of package closure.
3. Likely: a dedicated campaign manifest is the cleanest glue layer because P3 needs a lane-owned
   package that is narrower than `docking-arbitration` but broader than one single script.
   Evidence:
   - `tools/diag-campaigns/README.md` already treats campaign manifests as repo-owned bounded
     execution packages,
   - the four expectations naturally span several scripts rather than one all-purpose suite.
   Consequence if wrong:
   - this slice would either overload a generic docking suite or add another competing package shape.
4. Confident: `docking_arbitration_demo` remains the correct launched proof surface for this P3
   package.
   Evidence:
   - the selected scripts already target that demo,
   - the docking parity lane already treats it as the runner/platform multi-window proof surface.
   Consequence if wrong:
   - P3 would still lack one coherent launched entry even after freezing the package.

## Current gap

Before this decision, the repo already had the right script pieces, but no lane-owned bounded P3
entry.

The existing options were all slightly wrong for this lane:

- `diag-hardening-smoke-docking` is intentionally small and general-purpose,
- `docking-arbitration` is a broader docking suite rather than an IMUI-lane-specific parity
  package,
- and the generic `docking-smoke` campaign is a wider docking maintainer entry, not the compact P3
  answer this lane needs.

That left P3 with runner-owned checklist prose, but not yet one concise gate package that could be
reopened quickly.

## Frozen bounded parity package

From this point forward, the bounded P3 parity package is:

- campaign manifest:
  `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- launched target:
  `cargo run -p fret-demo --bin docking_arbitration_demo --release`
- canonical run command:
  `cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`
- canonical validation command:
  `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`

The package intentionally names one script per frozen expectation:

1. Hovered-window selection under overlap
   - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
2. Peek-behind under the moving tear-off window
   - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
3. Transparent payload overlap behavior
   - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
4. Mixed-DPI follow-drag expectation
   - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`

Important rule:

- the fourth script is the current bounded mixed-DPI / large-coordinate stress entry for this lane,
- it must still pass on single-monitor hosts, while surfacing mixed-DPI evidence when available,
- and it is good enough for the lane-owned package even though deeper platform acceptance work still
  belongs in the docking parity lane.

## Frozen package boundaries

For this lane:

- `tools/diag-campaigns/imui-p3-multiwindow-parity.json` owns the lane-specific bounded P3 package,
- `diag-hardening-smoke-docking` remains the small generic docking smoke entry,
- `docking-arbitration` remains the broader docking regression suite,
- and future runner-heavy expansion should replace items inside the campaign manifest rather than
  creating another parallel P3 package.

This also freezes the source-policy stance:

- do not widen `crates/fret-ui` because one of these four checks is painful,
- do not reopen generic `imui` helper growth to compensate for runner gaps,
- and route platform-specific follow work into the docking parity lane or a narrower runner
  follow-on.

## Decision

From this point forward:

1. `tools/diag-campaigns/imui-p3-multiwindow-parity.json` is the bounded P3 parity package for this
   lane.
2. The package explicitly names:
   - hovered-window,
   - peek-behind,
   - transparent payload,
   - and mixed-DPI follow-drag.
3. `docking_arbitration_demo` is the launched proof surface for this package.
4. `diag-hardening-smoke-docking` stays small; do not silently turn it into the P3 package.
5. If a future runner/platform change needs broader coverage, update this package or continue in the
   docking parity lane instead of reopening IMUI-layer ownership.

## Immediate execution consequence

For this lane:

- use the campaign manifest above as the default bounded P3 gate entry,
- treat `diag campaign validate` as the cheapest authoring-time gate,
- treat `diag campaign run imui-p3-multiwindow-parity --launch` as the canonical launched proof,
- and treat any attempt to solve these runner gaps via `crates/fret-ui` or generic `imui` growth as
  a source-policy regression.
