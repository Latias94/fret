# Icon System Extension v1

Status: Closed closeout lane (v1 icon contract shipped; follow-on only)
Last updated: 2026-04-09

Related:

- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

Closeout reading rule on 2026-04-09:

- treat this document as the historical design record for a closed icon-contract lane, not as an
  active roadmap
- read the shipped result from `CLOSEOUT_AUDIT_2026-04-09.md`
- reopen only through a narrower follow-on if fresh pack-specific or tooling-specific evidence
  appears

Closeout note on 2026-04-09:

- the v1 goal is complete:
  first-class icon-definition contract, explicit `SvgIcon` vs `SvgImage` runtime split, and
  explicit third-party pack metadata/install seams are now shipped
- the remaining questions are follow-on opportunities rather than active v1 contract debt

This lane is now explicitly **correctness-first**.

If the smallest possible patch and the durable public boundary disagree, choose the durable
boundary and accept the refactor while the repo is still pre-release.

## Why this lane exists

Fret already has the right high-level icon architecture:

- semantic `IconId` values for component authoring,
- optional icon-pack crates,
- renderer-owned SVG caching and rasterization,
- and explicit app/bootstrap install seams.

That baseline is good enough for monochrome UI icons and in-repo packs such as Lucide and Radix.
It is not yet a complete extension platform for:

- multicolor SVG icons as a first-class authoring path,
- third-party import crates with richer pack metadata,
- or the broader “Iconify-like” workflow where the framework contract must stay stable while icon
  sources vary widely.

This lane exists to close that gap without breaking ADR 0065's core rule:

- component code stays on semantic `IconId`,
- pack/vendor wiring stays explicit,
- rendering remains owned by the renderer/runtime stack,
- and policy stays out of `crates/fret-ui`.

## Scope

This lane owns three tightly related questions:

1. What durable icon-definition contract should `fret-icons` expose beyond raw SVG bytes and
   aliases?
2. What declaration/runtime surface should Fret expose for multicolor SVG icons without weakening
   the current monochrome `currentColor` icon path?
3. What should a first-party-approved third-party import-pack crate shape look like so we do not
   need a second pack-protocol rewrite later?

## Non-goals

This lane does not own:

- a generic remote Iconify client in the core runtime,
- design-system-specific icon choices for shadcn or Material recipes,
- generic asset locator/resolver evolution beyond the icon boundary already defined by ADR 0065,
- or pack-by-pack parity work for every upstream icon vendor.

If the work expands into one of those areas, split a narrower follow-on.

## Owning layers

- `ecosystem/fret-icons`
  - semantic contract, registry surface, pack metadata shape
- `crates/fret-ui`
  - declaration/runtime surface for multicolor icon rendering
- `ecosystem/fret-ui-kit`
  - default authoring helpers that resolve icon ids into UI-ready sources
- `ecosystem/fret-bootstrap`
  - pack registration and preload wiring on the golden path

This lane does not justify pushing icon policy or vendor-specific defaults into `crates/fret-ui`.

## First-principles constraints

1. Semantic `IconId` remains the only reusable component-facing icon identity.
2. Alias resolution must preserve the whole icon definition, not only raw bytes.
3. Renderer choice must be explicit in the resolved contract rather than guessed in helper code.
4. `crates/fret-ui` owns runtime SVG surfaces, not icon-pack or design-system policy.
5. Build-time import/vendoring is the approved Iconify-style workflow; runtime network fetching is
   not part of the framework icon contract.

## Assumptions-first baseline

### 1) Lane ownership

- Area: workstream ownership
- Assumption: no current active workstream owns icon-contract expansion as a first-class target.
- Evidence:
  - `docs/roadmap.md`
  - `docs/workstreams/README.md`
  - `docs/todo-tracker.md`
  - `docs/workstreams/resource-loading-fearless-refactor-v1/*`
- Confidence: Likely
- Consequence if wrong: we could duplicate a live lane instead of continuing it.

### 2) Semantics-first must stay intact

- Area: public component contract
- Assumption: component crates must continue to depend on semantic `IconId` values rather than
  vendor-prefixed ids or raw asset paths.
- Evidence:
  - `docs/adr/0065-icon-system-and-asset-packaging.md`
  - `docs/crate-usage-guide.md`
- Confidence: Confident
- Consequence if wrong: icon-pack expansion would leak vendor identity into reusable component APIs.

### 3) Multicolor support exists below the current default authoring surface

- Area: runtime/render pipeline
- Assumption: the renderer/runtime stack already supports multicolor SVG rasterization, but the
  default declarative icon path still lowers to monochrome mask rendering.
- Evidence:
  - `crates/fret-core/src/scene/mod.rs`
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`
  - `crates/fret-ui/src/canvas.rs`
- Confidence: Confident
- Consequence if wrong: the lane would need renderer work before authoring-surface work.

### 4) Third-party pack integration is intentionally supported today

- Area: extension surface
- Assumption: Fret already intends external pack crates to exist and to expose explicit install
  seams (`my_icons::app::install` or `register_icon_pack(...)`).
- Evidence:
  - `docs/crate-usage-guide.md`
  - `docs/examples/todo-app-golden-path.md`
  - `ecosystem/fret-bootstrap/src/lib.rs`
- Confidence: Confident
- Consequence if wrong: the work should stay in-tree-only and would not need a generalized pack protocol.

### 5) ADR 0065 is only partially implemented today

- Area: contract closure
- Assumption: the accepted ADR already names capabilities that are still not fully expressed in the
  current implementation, notably glyph fallback and optional multicolor component handling.
- Evidence:
  - `docs/adr/0065-icon-system-and-asset-packaging.md`
  - `ecosystem/fret-icons/src/lib.rs`
  - `crates/fret-ui/src/elements/cx.rs`
- Confidence: Confident
- Consequence if wrong: the lane risks over-designing beyond the accepted contract.

## Must-be-true outcomes

When this lane is genuinely done, the following should be true:

1. The icon contract clearly distinguishes semantic identity from vendor ids and from render mode.
2. Alias resolution preserves source + fallback + presentation semantics together.
3. Fret has a first-class declarative path for multicolor SVG icons that does not weaken the
   existing monochrome `currentColor` icon golden path.
4. Third-party pack crates can publish one explicit, grep-friendly installation shape that works on
   both the `fret` and `fret-bootstrap` golden paths.
5. Pack metadata/provenance has an explicit home rather than being left to README convention alone.
6. The implementation remains layered:
   - mechanism in `crates/*`,
   - pack/authoring helpers in `ecosystem/*`,
   - no design-system policy leaks into `crates/fret-ui`.
7. The lane leaves at least one smallest repro, one gate set, and one evidence set for each
   contract slice.

## Shipped closure

The lane now closes the original cross-layer contract gaps:

- the third-party pack contract is explicit in code and docs rather than implied by examples
  alone,
- pack metadata/provenance now has a first-class contract home,
- and the authored-vs-themed icon posture is now taught through a closed guidance path instead of
  remaining scattered.

## Remaining follow-on space

What remains after closeout is narrower than the original lane:

- pack-specific parity or curation work,
- richer diagnostics/gallery proof surfaces for authored-color icons,
- and future tooling that wants to consume `InstalledIconPacks`.

Those are follow-on opportunities, not unfinished v1 contract closure.

## Target shipped state

The intended interface state is recorded in:

- `docs/workstreams/icon-system-extension-v1/TARGET_INTERFACE_STATE.md`

That file is the contract-first target for this lane.
Implementation slices should move toward that state rather than optimize for the smallest patch set.

## Delivered phases

### Phase 0: baseline + contract closure

- Audited ADR 0065 against the current implementation.
- Froze the correctness-first target state for v1:
  - icon-definition contract,
  - declaration/runtime surface split,
  - pack-crate protocol,
  - metadata/provenance home.

### Phase 1: contract reset in `fret-icons`

- Introduced a first-class icon-definition contract so the registry resolves more than raw bytes.
- Landed render-intent/presentation and fallback semantics in the contract itself.
- Kept `fret-icons` renderer-agnostic.

### Phase 2: runtime SVG surface in `crates/fret-ui`

- Added a distinct declarative multicolor SVG surface instead of overloading `SvgIconProps`.
- Preserved the existing monochrome `SvgIcon` semantics.

### Phase 3: ecosystem authoring and pack protocol

- Updated `fret-ui-kit` so themed UI icons and registry-authored icons are two explicit postures.
- Wrote down the durable third-party pack/import protocol.
- Added pack metadata/provenance where the contract needs it.

### Phase 4: teaching surface and evidence

- Updated `docs/crate-usage-guide.md` and the relevant first-party example/docs surfaces.
- Added/refreshed tests and the smallest proof surfaces for multicolor and custom-pack flows.

## Historical first executable slice

The first slice stays documentation-first, but no longer optimizes for the smallest contract delta:

- record the contract/implementation drift,
- freeze the durable target interface state,
- decide the durable icon-definition contract,
- and name the exact declaration/runtime split before changing code.
