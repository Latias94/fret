# Action-First Authoring + View Runtime (Fearless Refactor v1) — Crate Plan

Last updated: 2026-03-03

This document describes the **crate boundary plan** for the refactor:

- which crates are impacted,
- which new crates (if any) we should introduce,
- what the “do not cross” boundaries are,
- and which parts are explicitly out of scope.

This is a workstream note. Hard boundaries must be backed by ADRs.

---

## Status (as of 2026-03-03)

- v1 landed without introducing new crates (Option A).
- `ActionId == CommandId` is implemented in `crates/fret-runtime/src/action.rs`.
- View runtime v1 lives under `ecosystem/fret/src/view.rs` and is used by cookbook/templates.

---

## 1) Boundary stance (do not violate)

Non-negotiable:

- `crates/fret-ui` remains **mechanism/contract-only** (ADR 0066).
- Interaction policy (dismiss rules, focus restore, hover intent, default sizing/spacing) stays in ecosystem
  (`fret-ui-kit`, `fret-ui-shadcn`, design-system crates).
- No backend types (`winit`, `wgpu`) leak into portable authoring surfaces.

---

## 2) Crates impacted (expected)

### 2.1 Portable core contracts

- `crates/fret-runtime`
  - Owns portable identity types and keymap-facing values.
  - Candidate landing site for `ActionId` (v1 may alias `CommandId`).

- `crates/fret-app`
  - Owns registration surfaces for metadata used by menus/palette/keymap.
  - Must converge action metadata with existing command metadata (avoid duplication).

### 2.2 UI kernel substrate

- `crates/fret-ui`
  - Must expose the **mechanism** to:
    - attach an `ActionId` to interactive elements,
    - route triggered actions through dispatch,
    - query availability at a point in the tree,
    - emit diagnostics traces (without encoding policy).

### 2.3 Ecosystem authoring/runtime

- `ecosystem/fret`
  - Golden path facade. Likely re-exports the new view runtime and action authoring sugar.

- `ecosystem/fret-ui-kit`
  - Supplies authoring ergonomics (builders, patch chain).
  - Needs an action-first adapter surface so components can bind actions without string glue.

- `ecosystem/fret-ui-shadcn` (and `ecosystem/fret-ui-material3`)
  - Component/policy layers. Must adopt action-first triggers for buttons/menu items where applicable.
  - Should not pull kernel-only hooks into policy code; keep a clean dependency story.

### 2.4 Other first-party frontends

- `ecosystem/fret-imui` + `ecosystem/fret-authoring`
  - Immediate-mode authoring frontend. Needs a stable “emit action trigger” seam.
  - Must remain policy-light and reuse the same state helpers via adapters.

- `ecosystem/fret-selector`, `ecosystem/fret-query`
  - State helpers. View runtime should compose them; avoid reimplementing selector/query semantics.

- `ecosystem/fret-genui-core` / `ecosystem/fret-genui-shadcn`
  - Data-driven frontend. Should align on action IDs (and optionally metadata surfaces) without
    allowing arbitrary dispatch.

---

## 3) New crates (optional; decide in Milestone M0)

This refactor can be landed without adding new crates by placing the view runtime inside `ecosystem/fret`.

However, two new crates may reduce coupling and clarify “what is stable”:

### Option A (minimal churn; recommended for v1)

- No new crates.
- Add modules under `ecosystem/fret`:
  - `fret::actions` (typed action sugar)
  - `fret::view` (view runtime)

Pros:

- fastest adoption (single dependency for apps),
- simplest to teach (golden path).

Cons:

- `ecosystem/fret` grows; may become a “misc bucket” unless carefully curated.

### Option B (cleaner boundaries; more churn)

- New: `ecosystem/fret-actions`
  - typed action sugar + macro exports,
  - re-exported by `fret`.

- New: `ecosystem/fret-view`
  - `View` + `ViewCx` runtime + hooks integration glue,
  - re-exported by `fret`.

Pros:

- smaller dependency surfaces for ecosystem crates,
- easier to test/iterate independently.

Cons:

- more crate plumbing + re-exports,
- more naming/bikeshedding overhead.

Decision gate:

- ADR 0308 (where view runtime lives) should pick A or B before implementation scales.

Decision (v1):

- Choose Option A for v1: no new crates; land `actions` + `view` modules under `ecosystem/fret`.

---

## 4) Fearless refactor boundaries (what is allowed to change)

Allowed within this workstream:

- Authoring surfaces in ecosystem crates (new APIs, deprecations, migrations).
- Action identity + dispatch contracts (as ADR-backed changes).
- View runtime façade introduced as additive surface.
- Adoption migrations in cookbook/gallery.

Not allowed (separate workstreams/ADRs required):

- Replacing the kernel element taxonomy (`AnyElement`/`ElementKind`) wholesale.
- Moving policy into kernel crates.
- Introducing a general-purpose scripting/plugin runtime.

---

## 5) Cleanup boundary (explicit deletion window)

The cleanup milestone (M6) may delete/quarantine legacy surfaces only if:

- all in-tree demos/templates have migrated, and
- ecosystem crates either migrated or explicitly document “legacy/compat” usage.

Candidate cleanup targets:

- legacy routing glue that is no longer recommended in templates,
- duplicated authoring entry points that teach conflicting patterns.

Note:

- MVU deprecation (if desired) must be its own explicit decision after adoption evidence.
