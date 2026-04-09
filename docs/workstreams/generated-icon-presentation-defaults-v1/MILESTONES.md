# Generated Icon Presentation Defaults v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Scope and evidence freeze

Exit criteria:

- The presentation-defaults problem is explicitly separated from the closed acquisition lane.
- The current runtime icon contract, generator registration path, and generator data model are
  audited.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened on 2026-04-09 as a narrow follow-on to the closed acquisition lane.
- M0 evidence freeze closed on 2026-04-09.
- The next active work is M1 presentation policy contract freeze.

## M1 — Presentation policy contract freeze

Exit criteria:

- The source of truth for default presentation is explicit.
- Ownership of generator/library/CLI/config responsibilities is explicit.
- The first proof target is explicit.

Primary evidence:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- M1 contract freeze closed on 2026-04-09.
- The next active work is M2 proof surface.

## M2 — Proof surface

Exit criteria:

- One smallest presentation-default proof lands.
- Authored-color imports reach the existing `OriginalColors` path.
- Monochrome imports keep the existing mask-mode posture.

Primary gates:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'`

Current status:

- M2 proof surface closed on 2026-04-09.
- See `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`.

## M3 — Docs and closeout

Exit criteria:

- Deterministic gates define the shipped policy.
- User-facing docs teach only the shipped contract.
- The lane closes explicitly or splits a narrower follow-on.

Primary gates:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo nextest run -p fret-ui-kit --features icons -E 'package(fret-ui-kit) & (test(declarative::icon::tests::icon_authored_uses_svg_image_for_original_color_icons) | test(declarative::icon::tests::icon_authored_uses_svg_icon_for_mask_icons))'`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

Current status:

- M3 docs and closeout closed on 2026-04-09.
- The lane is now closed on
  `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
