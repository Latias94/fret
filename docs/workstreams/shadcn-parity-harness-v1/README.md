---
title: Shadcn Parity Harness v1
status: active
date: 2026-05-09
scope: ui-gallery, fret-ui-shadcn, diagnostics, shadcn-parity
---

# Shadcn Parity Harness v1

This workstream turns local "looks wrong" reports into reusable parity cases for Fret's
GPU-first shadcn implementation. The first seed set is deliberately small:

- Button Group / Input: the search icon button must keep shadcn's compact default button padding
  instead of collapsing to the raw icon width.
- Button Group / Dropdown Menu: the chevron trigger must keep the same compact default button
  sizing while still respecting the upstream example's `!pl-2` override.
- Button Group / ButtonGroupText: the `https://` and `.com` addons must participate in the
  stretched control row and remain vertically centered with the adjacent input.

Mechanism Harness v2 (`docs/mechanism-harness-v2.md`) generalizes the reusable part of this seed:
scenario fixtures, observed runtime trees, shared geometry predicates, and case-id-addressable
runner output. This v1 workstream remains the shadcn/UI Gallery seed evidence; new mechanism-level
layout, hit-test, overlay, focus, or semantics regressions should start from the v2 harness shape.

## Source Precedence

Use the source that owns the axis being tested:

- Chrome and layout recipe truth: `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`,
  `button.tsx`, `input.tsx`, and `dropdown-menu.tsx`.
- Docs-path example truth: `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/examples/button-group-input.tsx`,
  `button-group-dropdown.tsx`, and `input-group-button-group.tsx`, plus
  `content/docs/components/button-group.mdx`.
- Interaction semantics: Radix DropdownMenu trigger/content semantics remain the behavior baseline;
  Base UI is a secondary headless reference when DOM assumptions need translation. The current seed
  defects are layout/chrome recipe defects, not focus, dismissal, or overlay mechanism defects.

## Reusable Workflow

1. Pick the exact upstream source axis before editing: `semantics`, `chrome`, `docs surface`, or
   `teaching surface`.
2. Audit the shadcn component source and the exact docs-path example. Example-local props and classes
   are parity truth for gallery snippets.
3. Classify the owning layer:
   - `crates/fret-ui`: only runtime mechanisms/contracts such as layout vocabulary, focus, hit-test,
     overlay routing, and semantics.
   - `ecosystem/fret-ui-kit`: reusable headless policy or token/chrome infrastructure.
   - `ecosystem/fret-ui-shadcn`: shadcn taxonomy, default recipe chrome, slot sizing, and component
     composition.
   - `apps/fret-ui-gallery`: first-party teaching snippets, page constraints, stable `test_id`, and
     diagnostics wiring.
4. Add stable selectors before automation. Prefer component-local `test_id` surfaces that map to the
   semantic or visual owner being tested.
5. Gate deterministic geometry in Rust when the invariant is numeric. Use diag scripts for portable
   repro, layout sidecars, screenshots, and bundles.
6. Leave a proof note for every seed: `Truth / Artifacts / Wiring / Proof / Residual risk`.

## Seed Proofs

### Button Group / Input

- Truth: upstream `Button` default size with a direct icon child uses compact horizontal padding
  (`has-[>svg]:px-3`), while `size="icon"` is the separate fixed-square lane.
- Artifacts: `Button` recipe padding logic in `ecosystem/fret-ui-shadcn/src/button.rs`; stable gallery
  selector `ui-gallery-button-group-input-search-button`.
- Wiring: `apps/fret-ui-gallery/src/ui/snippets/button_group/input.rs` renders the docs-path
  Button Group / Input preview.
- Proof: unit test `default_size_icon_only_button_keeps_compact_inline_padding`; render-flow test
  `gallery_button_group_shadcn_parity_seed_layout_invariants`; diag script
  `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json`.
- Residual risk: this seed checks default icon-only buttons in the Button Group context; future work
  should add web-vs-Fret numeric goldens for more button sizes and DPI/font variants.

### Button Group / Dropdown Menu

- Truth: upstream `button-group-dropdown.tsx` uses `DropdownMenuTrigger asChild` over a default-size
  outline `Button` with `!pl-2`; the trigger must not collapse around the chevron.
- Artifacts: same default button padding logic plus stable selector
  `ui-gallery-button-group-dropdown-trigger`.
- Wiring: `apps/fret-ui-gallery/src/ui/snippets/button_group/dropdown_menu.rs` owns the Fret
  translation of the upstream docs-path example.
- Proof: render-flow test `gallery_button_group_shadcn_parity_seed_layout_invariants`; diag script
  captures layout sidecar and screenshot for the dropdown seed.
- Residual risk: existing dropdown interaction scripts cover menu open, typeahead, focus, and
  item chrome; this seed only locks the trigger sizing regression.

### Button Group / ButtonGroupText

- Truth: upstream `ButtonGroupText` is `flex items-center`, so text/addon content centers within the
  stretched Button Group row.
- Artifacts: `ButtonGroupText` inner content row fills the stretched chrome height; stable selectors
  `ui-gallery-button-group-text-prefix`, `ui-gallery-button-group-text-control`, and
  `ui-gallery-button-group-text-suffix`.
- Wiring: `apps/fret-ui-gallery/src/ui/snippets/button_group/text.rs` is the focused Fret follow-up
  for the upstream `asChild` label lane.
- Proof: unit test `button_group_text_new_children_preserves_inline_custom_content`; render-flow test
  `gallery_button_group_shadcn_parity_seed_layout_invariants`; diag script captures a layout sidecar
  and screenshot for review.
- Residual risk: the current seed still focuses on ButtonGroupText in one docs-path viewport; the
  script now gates center-y alignment through `UiPredicateV1::BoundsMetricDelta`, so future visual
  centering cases can be added without app-specific Rust assertions.

## Gate Set

- Rust recipe tests:
  - `cargo nextest run -p fret-ui-shadcn button_padding_x_compacts_when_icon_present`
  - `cargo nextest run -p fret-ui-shadcn default_size_icon_only_button_keeps_compact_inline_padding`
  - `cargo nextest run -p fret-ui-shadcn button_group_text_new_children_preserves_inline_custom_content`
- UI Gallery render-flow test:
  - `cargo nextest run -p fret-ui-gallery gallery_button_group_shadcn_parity_seed_layout_invariants`
- Diagnostics script:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`

Promoted suite:

- `tools/diag-scripts/suites/ui-gallery-shadcn-parity/suite.json`

## Verification Evidence

2026-05-09 local proof run:

- Mechanism gates:
  - `cargo test -p fret-ui --lib chrome_container -j 1`
- Recipe gates:
  - `cargo test -p fret-ui-shadcn --lib default_size_icon_only_button_keeps_compact_inline_padding -j 1`
  - `cargo test -p fret-ui-shadcn --lib button_group_horizontal_text_items_fill_stretched_control_row -j 1`
  - `cargo test -p fret-ui-shadcn --lib button_group_text_new_children_preserves_inline_custom_content -j 1`
- Gallery gate:
  - `cargo test -p fret-ui-gallery --lib gallery_button_group_shadcn_parity_seed_layout_invariants -j 1 -- --nocapture`
- Diagnostics gate:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json --dir target/fret-diag/shadcn-parity-harness-v1 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- cargo run -p fret-ui-gallery`
  - Result: `PASS (run_id=1778305718625)`.
  - AI packet: `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/1778305718625/ai.packet`.
  - Share pack: `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/share/1778305718625.zip`.
  - Layout sidecars:
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/1778305719717-ui-gallery-shadcn-parity-seed.button-group-input.layout/layout.taffy.v1.json`
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/1778305720064-ui-gallery-shadcn-parity-seed.button-group-dropdown.layout/layout.taffy.v1.json`
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/1778305723134-ui-gallery-shadcn-parity-seed.button-group-text.layout/layout.taffy.v1.json`
  - Screenshots:
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/screenshots/1778305719780-ui-gallery-shadcn-parity-seed.button-group-input/window-4294967297-tick-35-frame-35.png`
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/screenshots/1778305720134-ui-gallery-shadcn-parity-seed.button-group-dropdown/window-4294967297-tick-40-frame-40.png`
    - `target/fret-diag/shadcn-parity-harness-v1/sessions/1778305469784-31936/screenshots/1778305723205-ui-gallery-shadcn-parity-seed.button-group-text/window-4294967297-tick-50-frame-50.png`

## Completion Audit

Objective-to-artifact checklist:

- Seed coverage: Input, Dropdown Menu, and ButtonGroupText are covered by stable gallery selectors,
  one render-flow invariant test, and one diagnostics script.
- Upstream source alignment: shadcn v4 `new-york-v4` component and docs-path example files are the
  recorded truth for chrome, slot sizing, and example-local props. Radix DropdownMenu remains the
  interaction baseline; Base UI remains the secondary headless reference for DOM-assumption
  translation. This seed did not require changing overlay/focus/dismissal semantics.
- Layer classification:
  - `crates/fret-ui`: flow-engine mechanism fix for wrapper transparency and cross-axis `Fill`
    promotion.
  - `ecosystem/fret-ui-shadcn`: recipe fixes for Button default icon-only padding and
    ButtonGroupText stretched chrome/content centering.
  - `apps/fret-ui-gallery`: teaching-surface selectors and render-flow parity gate.
  - `tools/diag-scripts`: portable diagnostics script and promoted suite.
- Stable `test_id` coverage: `ui-gallery-button-group-input-search-button`,
  `ui-gallery-button-group-dropdown-trigger`, `ui-gallery-button-group-text-prefix`,
  `ui-gallery-button-group-text-control`, and `ui-gallery-button-group-text-suffix` all appear in
  the PASS diagnostics bundle with unique matches.
- Automated gates: mechanism unit tests, recipe unit tests, UI Gallery render-flow test, registry
  verifier, and diagnostics PASS run are listed under Verification Evidence.
- Evidence artifacts: the PASS run records layout sidecars, screenshots, an AI packet, and a share
  zip under `target/fret-diag/shadcn-parity-harness-v1`.
- ADR/documentation closure: `docs/adr/IMPLEMENTATION_ALIGNMENT.md` now records the ADR 0057
  evidence for the flow-engine mechanism change, and `docs/shadcn-declarative-progress.md` links
  this workstream as the reusable shadcn parity harness seed.
