# Generated Icon Presentation Defaults v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this lane is now closed on an explicit versioned
`presentation-defaults.json` contract carried through `fret-icons-generator`, the thin
`fretboard icons import ...` CLI, generated registration/provenance output, runtime proof gates,
and public docs. See
`docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md` and
`docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

This workstream is a narrow follow-on to the closed `iconify-acquisition-prestep-v1` lane.
It does not reopen the shipped acquisition pre-step, the local-input-only generator contract, or
the runtime icon mechanism.

It owns one narrower question:

> how should generated icon packs choose correct default `IconPresentation` for imported icons,
> especially authored-color / multicolor assets, without destabilizing monochrome packs or pushing
> new heuristics into runtime code?

## Why this lane exists

The shipped icon system already closed the core runtime split:

- `Mask` is the themed/tinted path,
- `OriginalColors` is the authored-color path,
- and `icon_authored(...)` already routes `OriginalColors` icons to `SvgImage`.

The shipped generator/acquisition lanes also proved:

- imported Iconify data preserves authored SVG bodies,
- acquisition records upstream collection metadata such as `palette`,
- and generated packs remain deterministic and local-input-first.

What is still unresolved is the default presentation of generated packs:

- generated packs currently register every icon through the default `Mask` path,
- imported multicolor assets therefore keep their bytes but not their intended default render
  posture,
- and the repo does not yet define whether the right source of truth is explicit pack config,
  source analysis, collection-level hints, or a hybrid.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The runtime icon contract should stay closed; this lane should not reopen `SvgIcon` / `SvgImage` mechanism design. | Confident | `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `ecosystem/fret-ui-kit/src/declarative/icon.rs`, `ecosystem/fret-icons/src/lib.rs` | We would blur a generator/policy follow-on into a core runtime rewrite. |
| Generated packs currently default all imported icons to `Mask`. | Confident | `crates/fret-icons-generator/src/templates.rs`, `ecosystem/fret-icons/src/lib.rs` | The lane would be solving the wrong bug. |
| A correct fix likely requires generator-side metadata flow, not only a template-line swap. | Confident | `crates/fret-icons-generator/src/svg_dir.rs`, `crates/fret-icons-generator/src/iconify.rs` | We would under-scope the change and create a brittle patch. |
| Collection-level `palette` metadata may be useful input, but it is not obviously sufficient as the only defaulting rule. | Likely | `crates/fretboard/src/icons/acquire.rs`, Iconify `IconifyInfo` docs (`palette` is collection-level metadata for icon sets, not a per-icon presentation contract): `https://iconify.design/docs/types/iconify-info.html` | We could overfit to one upstream hint and misclassify mixed or local SVG sources. |
| SVG-directory imports need the same presentation story as Iconify collection imports. | Likely | `crates/fret-icons-generator/src/contracts.rs`, `crates/fret-icons-generator/src/svg_dir.rs` | A fix that only works for Iconify snapshots would leave the public generator contract inconsistent. |

## In scope

- Freeze how generated packs should decide default `IconPresentation`.
- Audit what generator/import surfaces need metadata or config changes.
- Decide whether the first shipped policy should be:
  - explicit config,
  - analyzed SVG defaults,
  - collection-level hints,
  - or a layered combination.
- Leave one repro/gate/evidence set for the first landable proof slice.

## Out of scope

- Reopening acquisition or hiding network inside `icons import`.
- Redesigning the runtime icon rendering split.
- Changing semantic alias policy.
- Broad visual-design policy for first-party icon packs unrelated to imported/generated packs.

## Target shipped state

### Non-negotiable target

- Generated packs can register imported icons with the correct default `IconPresentation`.
- The decision remains deterministic and reviewable.
- Existing monochrome packs do not silently regress into authored-color rendering.
- The runtime contract stays unchanged.

### Shipped direction

The lane closed on this v1 direction:

- extend generator-side intermediate metadata so imported icons carry an explicit render mode,
- keep the final registry registration explicit and generated in code,
- and prefer explicit versioned config over hidden runtime inference or silent heuristics.

This direction is now proven and should remain the baseline until a narrower follow-on justifies a
new helper surface.
