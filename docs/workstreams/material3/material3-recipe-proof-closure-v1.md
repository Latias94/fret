# Material 3 Recipe Proof Closure v1

Date: 2026-06-02
Status: Closed for the current public recipe surface

This note records the proof-gate closure pass for `ecosystem/fret-ui-material3`. The intent is to
make the shadcn-level completeness bar concrete for Material recipes without copying shadcn visuals:
every public recipe needs a reviewable proof layer, and every exception needs an explicit contract.

## Truth

- Public root recipe sources must not rely on `behavior_only` or `token_only` proof status.
- Rendered recipes must have a headless golden suite unless they are explicitly classified as a
  non-rendered supporting API.
- Headless goldens must cover the stable Material matrix: 3 scale factors by 4 scheme variants.
- Goldens prove rendered scene/chrome drift; focused behavior tests still prove semantics, keyboard,
  focus, dismiss, layout, and motion invariants.
- Material policy remains in `ecosystem/fret-ui-material3` foundation or recipe code. The manifest is
  a proof contract, not a reason to push Material behavior into `crates/*`.

## Artifacts

- Proof manifest:
  `ecosystem/fret-ui-material3/tests/fixtures/material3_recipe_proof_manifest_v1.json`
- Manifest hard gate:
  `ecosystem/fret-ui-material3/tests/material3_recipe_proof_manifest.rs`
- Headless golden test host:
  `ecosystem/fret-ui-material3/tests/material3_headless_goldens.rs`
- Headless runners:
  `ecosystem/fret-ui-material3/tests/support/headless_golden_runners/`
- Fixture suites:
  `ecosystem/fret-ui-material3/tests/fixtures/material3_headless_*_cases_v1.json`
- Golden outputs:
  `goldens/material3-headless/v1/material3-*.json`

Tabs and ChipSet now have dedicated fixture-driven headless suites, clearing the last rendered
recipe proof gaps in the current manifest. ChipSet intentionally has no container-level disabled
API; disabled state belongs to child chips because the set is a grouping and roving-focus policy
surface.

## Wiring

The manifest test now enforces these contracts:

- Every public root recipe source under `src/*.rs`, except `lib.rs`, appears in the manifest.
- Manifest entry ids match source file stems.
- Manifest entries are sorted and unique.
- `token_visual_component` values match `material3_token_visual_cases_v1.json`.
- Referenced behavior test files exist.
- Referenced headless suites have a runner module, a `material3_headless_<suite>_suite_goldens_v1`
  test marker, and all required golden files.
- `behavior_only` and `token_only` hard-fail.
- `supporting_api` is allowlisted; currently only `motion` is allowed.

Each headless suite must produce 12 golden files:

- Scale segments: `scale1_0`, `scale1_25`, `scale2_0`
- Scheme labels: `dark.tonal_spot`, `light.tonal_spot`, `dark.expressive`,
  `light.expressive`

The shared matrix lives in
`ecosystem/fret-ui-material3/tests/support/headless_golden_runners/mod.rs` as
`MATERIAL3_HEADLESS_SCALE_FACTORS_V1` and `MATERIAL3_HEADLESS_SCHEMES_V1`.

## Refresh Flow

Refresh one suite at a time so diffs remain reviewable. PowerShell example:

```powershell
$env:FRET_UPDATE_GOLDENS = "1"
cargo test -p fret-ui-material3 --test material3_headless_goldens material3_headless_tabs_suite_goldens_v1 -- --nocapture
Remove-Item Env:FRET_UPDATE_GOLDENS
cargo test -p fret-ui-material3 --test material3_headless_goldens material3_headless_tabs_suite_goldens_v1 -- --nocapture
```

Use the same shape for any suite:

```powershell
cargo test -p fret-ui-material3 --test material3_headless_goldens material3_headless_<suite>_suite_goldens_v1 -- --nocapture
```

The suite name uses underscores in runner and test names. Golden file stems use hyphens, for example
`chip_set` maps to `material3-chip-set.scale1_0.dark.tonal_spot.json`.

## New Component Checklist

1. Add or refactor the public recipe source under `ecosystem/fret-ui-material3/src/`.
2. Add a manifest entry whose `id` equals the source file stem.
3. Add `token_visual_component` only when the component participates in the token visual fixture
   suite.
4. Add focused behavior tests for semantics, keyboard/focus, dismiss, selection, layout, or motion.
   Do not treat a headless golden as behavior proof for state machines.
5. Add a fixture/parser/runner when the recipe renders a stable scene.
6. Register the runner in `material3_headless_goldens.rs`.
7. Generate and replay the 12-file golden matrix without `FRET_UPDATE_GOLDENS`.
8. Run the proof manifest test before committing the component.
9. If the source is a public non-rendered helper, classify it as `supporting_api`, explain
   `known_gap`, and extend the allowlist only when the exception is intentional.

## Proof

Recommended gates for this closure layer:

```powershell
cargo test -p fret-ui-material3 --test material3_recipe_proof_manifest material3_recipe_proof_manifest_tracks_public_recipe_coverage_v1 -- --nocapture
cargo test -p fret-ui-material3 --test material3_headless_goldens material3_headless_tabs_suite_goldens_v1 -- --nocapture
cargo test -p fret-ui-material3 --test material3_headless_goldens material3_headless_chip_set_suite_goldens_v1 -- --nocapture
```

Full component changes should still run their focused behavior tests plus `cargo check` and `cargo
clippy` for `fret-ui-material3`.

## Residual Risk

- Headless JSON goldens are deterministic scene proofs, not a replacement for GPU raster screenshots,
  assistive-technology validation, or mobile IME/inset coverage.
- `motion.rs` remains a public supporting API. Its proof is through component motion tests, not a
  standalone rendered recipe suite.
- ChipSet would benefit from a future `Toolbar` semantics follow-up if the core semantics vocabulary
  is expanded; that should be a separate mechanism/a11y lane.
- Gallery snippets and rustdoc examples are not closed by this proof layer. They should be audited
  next against the manifest so the teaching surface matches the tested recipe surface.
