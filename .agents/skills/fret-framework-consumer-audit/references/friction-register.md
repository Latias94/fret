# Friction register categories

Use these categories to classify findings from a framework-consumer audit.

## 1) Discoverability / onboarding

Typical symptom:

- the user cannot tell which doc, example, or command to start from

Capture:

- the misleading starting point
- the correct starting point
- the exact moment the consumer had to switch tracks

Likely owner:

- docs, example index, template docs

Typical next move:

- docs/examples patch

## 2) Scaffold / template gap

Typical symptom:

- generated code runs, but the first real customization immediately falls off the golden path

Capture:

- generated file or command
- the first non-trivial edit that became awkward
- what extra concepts the user had to learn unexpectedly

Likely owner:

- `fretboard new ...`, templates, first-party examples

Typical next move:

- template update, example patch, or builder recipe

## 3) Teaching-surface drift

Typical symptom:

- examples teach a surface that is no longer the recommended public path

Capture:

- snippet/page/doc that teaches the wrong thing
- the intended public surface
- why a new user would copy the wrong pattern

Likely owner:

- first-party docs/examples/gallery snippets

Typical next move:

- docs/example cleanup

## 4) Public API / ergonomics gap

Typical symptom:

- a small consumer task requires too many concepts, imports, or escape hatches

Capture:

- the minimal task
- the unexpected concepts or helper gaps
- whether the burden comes from docs, helpers, or the API shape itself

Likely owner:

- ecosystem authoring surface, public helper APIs, sometimes hard contracts

Typical next move:

- helper addition, API redesign, or maintainer workstream

## 5) Policy / component parity gap

Typical symptom:

- the consumer experience is rough because a component or recipe behaves unlike the expected upstream target

Capture:

- consumer-visible mismatch
- smallest repro
- whether the issue belongs to mechanism, policy, or recipe

Likely owner:

- `ecosystem/*`, parity surfaces

Typical next move:

- `fret-shadcn-source-alignment` or `fret-material-source-alignment`

## 6) Runtime / contract bug

Typical symptom:

- the task fails because focus, layout, routing, events, or semantics are actually wrong

Capture:

- smallest runnable repro
- broken invariant
- owner layer proof

Likely owner:

- `crates/*` contract/mechanism layers, or a cross-layer seam

Typical next move:

- `fret-diag-workflow`, then maintainer lane if needed

## 7) Diagnostics / evidence gap

Typical symptom:

- the issue is real, but the consumer or maintainer cannot prove it quickly

Capture:

- the missing selector, script, sidecar, or artifact
- what evidence would have made the issue easy to review

Likely owner:

- diag tooling, test surfaces, `test_id` coverage

Typical next move:

- `fret-diag-workflow`

## 8) Performance footgun

Typical symptom:

- the task “works” but the path teaches janky, expensive, or unbounded patterns

Capture:

- user-visible hitch or suspicious pattern
- whether the issue is default-lane guidance or an implementation regression

Likely owner:

- guidance, recipes, perf-sensitive component/runtime paths

Typical next move:

- `fret-perf-optimization` or `fret-diag-workflow`

## 9) Ecosystem / integration gap

Typical symptom:

- the consumer can build the toy demo, but real app workflows become awkward when integrating expected Rust/app ecosystem pieces

Capture:

- the concrete integration target or workflow
- what the consumer expected to compose naturally
- what framework-shaped workaround was required
- whether the issue is docs, helper gaps, or contract shape

Likely owner:

- public authoring surface, framework contracts, external-app guidance, sometimes tooling

Typical next move:

- `fret-framework-consumer-audit` for proof and routing first,
- then `fret-framework-maintainer-guide`, `fret-external-app-mode`, or `fret-app-ui-builder` depending on ownership

## Minimum fields per finding

Every finding should record:

- severity (`P0` / `P1` / `P2`)
- lane
- user symptom
- broken truth
- smallest proof
- likely owner
- next move
- evidence type
