# imui compatibility retained surface reduction v1 - milestones

Tracking doc: `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`

Baseline audit:
`docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`

Closeout audit:
`docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

This file is intentionally narrower than the old retained-bridge exit plan.
It tracks only the public/proof `imui` compatibility-retained follow-on.

## Phase A - Source-of-truth handoff

Status: Completed

Goal:

- make this directory the active follow-on for retained-backed `imui` public/proof surfaces
- and keep v2 as a closed closeout record

Deliverables:

- this directory with `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and a baseline audit
- updated docs entrypoints that point here instead of the closed v2 lane

Exit gates:

- contributors have one current answer to "what is the active `imui` follow-on now?"
- and v2 is no longer presented as if it were still active

## Phase B - Teaching-surface containment

Status: Completed

Goal:

- keep retained-bridge authoring out of the normal first-party `imui` teaching surface
- while preserving one explicit compatibility proof where it still adds value

Deliverables:

- a source-policy gate over first-party `imui` examples
- an explicit retained-bridge compatibility warning on the surviving proof demo

Exit gates:

- only explicit compatibility proofs touch retained-bridge authoring on first-party `imui` demos
- other `imui` examples stay on the normal facade/editor surface story

## Phase C - Ecosystem compatibility labeling

Status: Completed

Goal:

- make retained-backed public ecosystem `imui` modules read as compatibility-only instead of as
  silent normal surfaces

Deliverables:

- updated module docs in `fret-node` and `fret-plot`
- source-policy tests that lock the compatibility labeling

Exit gates:

- retained-backed public `imui` modules are visibly compatibility-only and delete-planned
- and the repo does not rely on reviewer memory to preserve that posture

## Phase D - Survival / delete decisions

Status: Completed

Goal:

- decide which retained-backed public `imui` facades survive, under what names, and for how long

Deliverables:

- explicit delete decisions for node and plot retained-backed `imui` facades
- the surviving first-party proof moved onto the declarative compatibility surface
- one final closeout audit

Exit gates:

- there is no ambiguous retained-backed public `imui` surface left in limbo
- and future cleanup can continue from the declarative node-graph compatibility seam without
  reopening the whole `imui` stack
