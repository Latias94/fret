# SVG Presentation Analysis Scaffolding v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this lane is now closed on an explicit helper-owned follow-on for
`fretboard icons suggest svg-dir-presentation-overrides ...`. The shipped surface can analyze a
local SVG directory, emit a conservative `presentation-defaults.json` with per-icon
`original-colors` overrides only when evidence is strong, and optionally emit a versioned review
report, all without changing generator/import defaults. See
`docs/workstreams/svg-presentation-analysis-scaffolding-v1/M2_PROOF_SURFACE_2026-04-09.md` and
`docs/workstreams/svg-presentation-analysis-scaffolding-v1/CLOSEOUT_AUDIT_2026-04-09.md`.

This workstream is a narrow follow-on to the closed
`iconify-presentation-defaults-report-v1` lane. It does not reopen generated-pack
presentation policy, runtime icon rendering ownership, or provenance-driven pack-level defaulting.

It owns one narrower question:

> once Fret can consume an explicit versioned `presentation-defaults.json`, how should it help
> users starting from a local SVG directory scaffold conservative per-icon authored-color
> overrides without teaching hidden policy or widening the stable generator contract?

## Why this lane exists

The shipped icon contract already supports explicit presentation policy:

- generated/imported packs can consume `presentation-defaults.json`,
- provenance-driven Iconify helpers can scaffold advisory defaults from explicit acquisition data,
- and optional review reports can keep those decisions auditable.

What remained open was the local-SVG path:

- many third-party/custom packs start from a repository-owned SVG directory rather than Iconify
  acquisition provenance,
- authored-color exceptions are usually sparse and per-icon rather than pack-wide,
- and generic SVG heuristics are too noisy to become an implicit defaulting rule.

This lane therefore targeted a thin helper layer:

- explicit CLI stage in `fretboard`,
- reuse of the existing config contract,
- conservative strong-evidence analysis only,
- optional report output for review,
- and no change to import or runtime behavior.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The shipped `presentation-defaults.json` contract is already correct and should stay unchanged here. | Confident | `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fret-icons-generator/src/presentation_defaults.rs` | This lane would accidentally turn into another generator-contract rewrite. |
| Local SVG analysis belongs in `fretboard`, not `fret-icons-generator`, because the analysis is source-specific convenience logic rather than stable generation policy. | Confident | `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`, `crates/fretboard/src/icons/mod.rs`, `crates/fret-icons-generator/src/contracts.rs` | We would widen the stable generator surface with heuristic/helper logic. |
| The generator may still need one tiny shared helper so the SVG-dir analysis uses the exact same icon-name normalization as import/generation. | Likely | `crates/fret-icons-generator/src/svg_dir.rs`, `crates/fret-icons-generator/src/naming.rs` | Helper output could drift from the actual generated icon ids, making suggested overrides invalid. |
| Generic SVG analysis is only trustworthy as a conservative per-icon override suggester, not as a pack-level `default_render_mode` classifier. | Confident | `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`, local SVG variability, absence of explicit provenance like Iconify `palette` | The helper would silently overfit and lock authored-color policy too broadly. |
| Parse failures should stay best-effort report entries instead of failing the whole helper run once other icons remain analyzable. | Likely | Existing helper/report posture in `crates/fretboard/src/icons/suggest.rs`, review-first CLI stance | One broken SVG would make the helper unusable on large directories. |

## In scope

- One explicit helper subcommand:
  `fretboard icons suggest svg-dir-presentation-overrides --source <dir> --out <file> [--report-out <file>]`
- Conservative analysis of local SVG directories using shared generator naming rules.
- Emitting the existing versioned `presentation-defaults.json` contract with per-icon overrides
  only.
- Optional versioned report output for review and auditing.
- Public docs/source-policy coverage that teaches the helper as advisory scaffolding.

## Out of scope

- Changing `presentation-defaults.json`.
- Import-time default inference or runtime icon-policy changes.
- Inferring pack-level `default_render_mode`.
- Auto-classifying single-color non-black icons as authored-color.
- Acquisition/network work or new provenance formats.

## Target shipped state

### Non-negotiable target

- The helper remains explicit and file-based.
- Only strong evidence produces per-icon `original-colors` overrides.
- The helper does not infer `default_render_mode`.
- Generated icon names in the suggestion output exactly match generator/import naming.
- Parse failures remain visible in the report without invalidating the whole run.

### Shipped direction

This lane closed on:

- an explicit local-SVG analysis helper in `fretboard`,
- one small shared naming export in `fret-icons-generator`,
- conservative per-icon override scaffolding only,
- optional versioned review reports,
- and docs/tests that keep the helper outside import defaults.
