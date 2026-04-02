# Shadow Surface (Fearless Refactor v1) — TODO

Status: Complete (v1 closure landed; future upgrades need a new workstream or ADR)

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/shadow-surface-fearless-refactor-v1/MILESTONES.md`
- Existing blur shadow mechanism workstream:
  - `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `SSFR-{area}-{nnn}`

---

## A. Baseline Audit

- [x] SSFR-audit-001 Capture the upstream shadow values actually used by first-party
  `new-york-v4` shadcn recipes.
  - Start with `shadow-xs/sm/md/lg/xl`.
  - Source should come from upstream source files and/or checked-in web goldens, not recollection.
  - Captured through the current `new-york-v4` source/golden-backed preset audit and the linked
    shadow footprint gates for `shadow-xs/sm/md/lg/xl` surfaces (`Menubar`, `Card`, `Calendar`,
    overlay content, and chart tooltip).

- [x] SSFR-audit-002 Inventory current first-party shadow consumers and classify them:
  - `ShadowStyle` portable baseline consumers,
  - shadcn preset consumers,
  - manual custom `ShadowStyle` consumers,
  - `DropShadowV1` effect consumers.
  - Preset consumers stay on the shared `decl_style::shadow_*` mapping in
    `ecosystem/fret-ui-kit/src/declarative/style.rs`.
  - Remaining manual `ShadowStyle` sites currently split into:
    - product-owned editor surfaces (`ecosystem/fret-ui-editor/src/primitives/popup_surface.rs`,
      `apps/fret-editor/src/viewport_overlays.rs`)
    - interaction/animation helpers (`ecosystem/fret-ui-shadcn/src/input_otp.rs`,
      `ecosystem/fret-ui-shadcn/src/tabs.rs`)
    - design-system-specific mapping (`ecosystem/fret-ui-material3/src/foundation/elevation.rs`)
    - shared generic toast fallback retained intentionally
      (`ecosystem/fret-ui-kit/src/window_overlays/render.rs`), now source-aligned to the checked-in
      Sonner web baseline and covered by dedicated light/dark gates
  - Current explicit `DropShadowV1` consumers remain effect-owned lanes such as node canvas shadow /
    glow plus dedicated demo/conformance surfaces, not generic shadcn preset recipes.

- [x] SSFR-audit-003 Record which current docs/audits claim shadow parity without a shadow-specific
  gate and add status notes or follow-up evidence anchors.
  - Start with `docs/audits/shadcn-card.md`.
  - `docs/audits/shadcn-card.md` and `docs/audits/shadcn-new-york-v4-alignment.md` now point to
    explicit shadow gates/evidence.
  - `docs/audits/shadcn-sonner.md` now points at the dedicated Sonner shadow gate and documents the
    generic toast fallback as intentional retained behavior instead of an open parity gap.

---

## B. Preset and Theme Alignment

- [x] SSFR-preset-010 Align `shadow_xs()` to the chosen upstream source of truth.
  - Audited against the current `new-york-v4` `shadow-xs` usage and kept as-is.
  - Existing `Menubar` root light/dark shadow gate is the proof surface for this preset lane.

- [x] SSFR-preset-011 Align `shadow_sm()` to the chosen upstream source of truth.
  - Landed in `ecosystem/fret-ui-kit/src/declarative/style.rs`.
  - Card is now the first proof surface, backed by a dedicated light/dark gate.

- [x] SSFR-preset-012 Audit and align `shadow_md()`, `shadow_lg()`, and `shadow_xl()` so the
  preset family is internally coherent instead of fixing only one visible case.
  - `shadow_md()` / `shadow_lg()` were audited against the current `new-york-v4` overlay surfaces
    and kept as-is; existing `Select` / `DropdownMenu` / `ContextMenu` / `Menubar` gates already
    prove those footprints directly.
  - `shadow_xl()` is now evidenced by a dedicated chart tooltip light/dark footprint gate.

- [x] SSFR-theme-020 Seed `component.shadow.*` values in shadcn theme configuration where the
  preset baseline is supposed to be style-specific and reviewable.
  - `shadcn_new_york_config(...)` now seeds explicit geometry metrics for the audited
    `shadow-xs/sm/md/lg/xl` family, including the secondary `sm2/md2/lg2/xl2` lanes.
  - The values intentionally match the reviewed `fret-ui-kit` fallback literals so the theme owns
    the posture explicitly and reviewably.

- [x] SSFR-theme-021 Decide whether shadow color stays on one semantic `shadow` color token plus
  per-preset alpha, or whether some style baselines need additional explicit color/token treatment.
  - v1 decision: keep one semantic `shadow` color token and keep per-preset alpha in the preset
    helpers; do not split extra shadow color lanes yet.
  - `shadcn_new_york_config(...)` now seeds `shadow` explicitly to opaque black so the current
    Tailwind/shadcn posture is theme-owned instead of relying on the fallback path.

---

## C. Gates and Evidence

- [x] SSFR-gate-030 Add a card shadow parity gate.
  - Landed in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`.
  - Covers light and dark.
  - Gate checks shadow footprint/layer outcome, not just border/radius.

- [x] SSFR-gate-031 Extract or generalize existing shadow-inset helpers so non-overlay components
  can reuse the same comparison logic that overlay chrome already uses.
  - Landed as `ecosystem/fret-ui-shadcn/tests/support/shadow_insets.rs`.

- [x] SSFR-gate-032 Add at least one more non-overlay shadow gate beyond card.
  - `Calendar` now gates `shadow-sm` on a different geometry/radius profile in light and dark.

- [x] SSFR-gate-033 Update evidence anchors in shadcn audit docs after the new gates land.
  - Updated:
    - `docs/audits/shadcn-card.md`
    - `docs/audits/shadcn-calendar.md`
    - `docs/audits/shadcn-new-york-v4-alignment.md`

---

## D. Contract Closure

- [x] SSFR-contract-040 Decide the current v1 relationship between `ShadowStyle` and
  `DropShadowV1`.
  - v1 now explicitly chooses the coexistence posture.
  - `ShadowStyle` remains the portable component baseline; `DropShadowV1` remains the explicit
    effect-backed blur path.
  - Current `DropShadowV1` product consumers are effect-owned surfaces such as node static-node
    shadow and wire glow, not generic shadcn preset surfaces.

- [x] SSFR-contract-041 If `ShadowStyle` remains the portable baseline, update docs to say that
  clearly and stop implying CSS-like blur fidelity where we do not prove it.
  - `docs/adr/0060-shadows-and-elevation.md` now says this explicitly and scopes blur-backed work
    to ADR 0286 instead of implying silent fidelity upgrades.

- [x] SSFR-contract-042 If some component-shadow lanes should upgrade toward effect-backed blur on
  capable backends, document the trigger rules and deterministic degradation story before code
  deletion begins.
  - v1 rule: no implicit promotion of generic component shadows.
  - Effect-backed blur is reserved for surfaces that own explicit `EffectMode::FilterContent`
    bounds/intermediates and accept ADR 0286 degradation/perf rules.

- [x] SSFR-contract-043 If the chosen posture changes ADR 0060 materially, update or supersede the
  ADR and refresh any affected alignment docs.
  - ADR 0060 is now updated to document the coexistence posture with ADR 0286.
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md` is refreshed for both ADR 0060 and ADR 0286.

---

## E. Cleanup

- [x] SSFR-cleanup-050 Remove delete-ready wrong/stale shadow mappings after preset alignment and
  gate closure.
  - The previously open candidate, the generic toast fallback shadow in
    `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, is now covered by dedicated Sonner
    light/dark shadow gates and documented as source-aligned retained behavior.
  - Result: this pass does not leave any delete-ready stale preset mapping behind; surviving
    shared/manual sites are intentional product-owned surfaces, animation helpers, design-system
    mappings, or the retained generic toast baseline.

- [x] SSFR-cleanup-051 Remove or rewrite stale docs that still present old parity conclusions as
  shipped truth.
  - `docs/audits/shadcn-card.md` no longer says the `ShadowStyle` vs `DropShadowV1` contract/theme
    decision is still pending; it now points at the landed ADR/alignment closure and keeps only the
    cleanup follow-up open.
  - `docs/audits/shadcn-sonner.md` now points at the dedicated Sonner shadow gate and treats the
    shared toast fallback as the intentional retained baseline rather than an unresolved gap.

- [x] SSFR-cleanup-052 Re-audit any remaining custom/manual `ShadowStyle` construction sites that
  are clearly compensating for preset drift rather than expressing real product intent.
  - `ecosystem/fret-ui-editor/src/primitives/popup_surface.rs`: intentional editor-owned popup
    chrome backed by editor token keys and popup-specific tests.
  - `apps/fret-editor/src/viewport_overlays.rs`: intentional editor viewport marker shadow for
    crosshair readability, not a shadcn preset compensation.
  - `ecosystem/fret-ui-shadcn/src/input_otp.rs`: intentional ring/box-shadow transition emulation
    for the active slot, not recipe elevation drift.
  - `ecosystem/fret-ui-shadcn/src/tabs.rs`: `shadow_with_presence(...)` is an animation helper over
    the audited `shadow_sm()` preset, not a separate stale mapping.
  - `ecosystem/fret-ui-material3/src/foundation/elevation.rs`: intentional Material-specific
    elevation mapping sourced from Material Web, not shadcn preset drift.
  - Result: no additional delete-ready manual `ShadowStyle` sites were proven in this pass, and the
    previously open generic toast fallback is now closed as intentional retained shared chrome
    rather than a stale mapping candidate.
