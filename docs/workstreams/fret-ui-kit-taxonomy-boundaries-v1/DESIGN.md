# Fret UI Kit Taxonomy Boundaries v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`fret-ui-kit` is doing several jobs: style helpers, headless engines, Radix-like primitives,
declarative adapters, IMUI helpers, and recipes. The previous IMUI owner-split lanes reduced
private module size, but the broader taxonomy still needs a source-backed owner map so future work
does not move policy into `fret-ui` or put component recipes into low-level primitives.

## Target Taxonomy

- `style`: tokens, variants, density, and visual helper resolution.
- `headless`: behavior engines and data-only policy kernels.
- `primitives`: Radix/Base UI-style behavior wrappers and provider/portal helpers.
- `declarative`: element adapters and mechanism composition helpers.
- `imui`: immediate-mode authoring helpers and Dear ImGui parity policy.
- `recipes`: opinionated app/component compositions.

## First Slice

Create a source audit that maps current modules to these owners, then move or rename one confused
owner without changing public behavior.

## Non-goals

- Moving policy into `crates/fret-ui`.
- Rebuilding the entire kit layout in one patch.
- Widening the public `fret-imui` facade without a proof surface.
