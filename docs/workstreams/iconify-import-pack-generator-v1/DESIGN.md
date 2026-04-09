# Iconify Import-Pack Generator v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this document remains the design/freeze context for the lane, but the
final shipped verdict now lives in
`docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

This workstream is a narrow follow-on to `icon-system-extension-v1`. It does not reopen the icon
runtime split, the icon-definition contract, or the pack metadata/install seam that already closed
there.

It owns one narrower question:

> how a third-party developer should generate an icon-pack crate against the shipped Fret icon
> contract, using build-time import/vendoring rather than a new runtime icon client.

## Why this lane exists

The previous lane closed the v1 contract correctly:

- semantic `IconId` stays the reusable component-facing identity,
- `SvgIcon` vs `SvgImage` is an explicit runtime split,
- and pack metadata / install seams are now explicit through `PACK_METADATA`, `PACK`,
  `IconPackRegistration`, and app/bootstrap installation paths.

What remains open is not another runtime question.
It is a producer-tooling question:

- how should a developer take Iconify-style source data or local SVGs,
- generate a crate that fits the shipped pack contract,
- and keep that generated output deterministic, reviewable, and publishable?

That is exactly the kind of problem that should become a narrow follow-on instead of silently
reopening the broader icon-system lane.

The accepted M1 boundary freeze for that producer contract now lives in:

- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

## First-principles constraints

1. Semantic `IconId` remains the only reusable component-facing icon identity.
2. Build-time import/vendoring is the approved workflow; a runtime Iconify network client is out
   of scope.
3. Generated output must target the shipped pack contract directly:
   `PACK_METADATA`, `PACK`, `VENDOR_PACK`, optional semantic-alias registration values, and
   explicit `app::install(...)` seams.
4. Generated code and assets must be repo-committable and git-reviewable rather than hidden behind
   opaque runtime caches.
5. This lane must not leak pack-specific policy or design-system semantics into `crates/fret-ui`
   or `ecosystem/fret-icons`.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The predecessor lane already shipped the v1 icon contract that generated packs must target. | Confident | `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `docs/adr/0065-icon-system-and-asset-packaging.md` | This lane would accidentally become another contract-reset lane instead of a generator follow-on. |
| The repo already has a partial generator substrate, but it is still vendor-specific today. | Confident | `tools/gen_icons.py`, `tools/icon_codegen.py`, `tools/sync_icons.py`, `tools/verify_icons.py`, `ecosystem/fret-icons-lucide`, `ecosystem/fret-icons-radix` | The first slice would need to start from zero rather than consolidating an existing pattern. |
| The real missing surface is the producer contract for generated crates, not another app/runtime API. | Likely | `docs/crate-usage-guide.md`, `docs/examples/todo-app-golden-path.md`, `ecosystem/fret-bootstrap/src/lib.rs`, `ecosystem/fret-icons/src/lib.rs` | This lane might otherwise overfocus on authoring helpers or runtime rendering instead of generated-pack shape. |
| A dioxus-iconify-style workflow is useful as a non-normative reference for build-time vendoring, but not as a dependency or direct API model. | Likely | optional local reference `repo-ref/dioxus-iconify/README.md`, `docs/repo-ref.md` | We could either ignore a useful precedent or overfit to another framework's runtime API. |

## In scope

- Freeze the v1 source-input boundary for generated packs.
- Freeze the generated crate/output boundary for v1.
- Decide how provenance/import model should be recorded for generated packs.
- Decide what part of the generator is repo-local tooling vs reusable public surface.
- Leave one proof surface, one gate set, and one evidence set for future implementation slices.

## Out of scope

- A runtime HTTP Iconify client.
- Another `fret-ui` runtime SVG surface.
- Reworking `IconRenderMode`, `SvgIcon`, or `SvgImage`.
- Design-system-specific icon choices for shadcn, Material, or app recipes.
- A generic remote asset loader beyond the icon-pack boundary.

## Target shipped state

### Non-negotiable target

- A developer can start from Iconify-style collection data or local SVG inventories and generate a
  pack crate that fits Fret's current icon contract without inventing another registration shape.
- The generated crate is explicit and grep-friendly:
  - vendored SVG assets,
  - generated ids/constants,
  - `PACK_METADATA`,
  - `PACK` / `VENDOR_PACK`,
  - optional semantic alias registration surface,
  - explicit `app::install(...)`.
- Provenance is explicit and durable enough to survive later diagnostics/tooling work.
- The output is deterministic and safe to commit into source control.

### Accepted generator boundary

The accepted v1 direction is:

- inputs:
  - pinned local Iconify-style collection snapshots,
  - local SVG files/directories,
  - explicit local alias/provenance configuration,
  - and no stable in-generator network fetch contract;
- outputs:
  - a normal Rust icon-pack crate in the shape already taught by first-party packs,
  - deterministic generated source + vendored SVG assets + provenance/readme output,
  - optional semantic alias registration only when explicitly configured,
  - and no runtime dependency on a generator crate;
- reusable surface:
  - a future public CLI backed by a reusable library crate,
  - not a repo-local `tools/`-only answer.

The unresolved question is no longer the contract boundary itself.
It is the smallest proof surface that demonstrates this frozen contract end-to-end.

## Optional reference posture

The local `repo-ref/dioxus-iconify/README.md` checkout is useful because it demonstrates a
generator-first, git-friendly, build-time icon import workflow.

However:

- `repo-ref/` is local state, not a dependency,
- its runtime component API is not normative for Fret,
- and this lane should borrow workflow lessons rather than cargo-cult another framework's output.

## Baseline audit

The current starting fact pattern is recorded in:

- `docs/workstreams/iconify-import-pack-generator-v1/BASELINE_AUDIT_2026-04-09.md`

That audit is the current status read for M0:

- the shipped predecessor contract is explicit,
- the current generator substrate is only partially generic,
- and the current first-party pack crates are only partially generated.

The next accepted decision after that baseline is:

- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

The first landed implementation/proof read after that freeze is:

- `docs/workstreams/iconify-import-pack-generator-v1/M2_PROOF_SURFACE_2026-04-09.md`

The current M4 source-expansion follow-on read after that is:

- `docs/workstreams/iconify-import-pack-generator-v1/M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md`
