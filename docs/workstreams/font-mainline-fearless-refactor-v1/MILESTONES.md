# Font Mainline Fearless Refactor v1 — Milestones

Status: Active

## M0: Audit and gate baseline frozen

Exit criteria:

- `fret-fonts`, `fret-render-text`, and `fret-launch` have current audit notes.
- The fallback diagnostics 3-pack is available:
  - mixed-script bundled fallback
  - locale-change fallback-policy key bump
  - settings-change fallback-policy key bump
- ADR 0257 implementation alignment points at the current font diagnostics evidence.

Primary evidence:

- `docs/workstreams/crate-audits/fret-fonts.l0.md`
- `docs/workstreams/crate-audits/fret-render-text.l0.md`
- `docs/workstreams/crate-audits/fret-launch.l1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## M1: `fret-fonts` becomes a small manifest crate

Exit criteria:

- Asset/profile declarations are no longer maintained as one large file.
- Feature-matrix coverage exists for the supported bundle combinations.
- The public surface is profile-first and does not encourage policy leakage.

Suggested gates:

- `cargo nextest run -p fret-fonts`
- representative feature-matrix `cargo check` / `cargo nextest` runs

## M2: `fret-render-text` gets explicit ownership seams

Exit criteria:

- Font DB/catalog/rescan/injected-font retention no longer live in the same large module as all
  shaping entrypoints.
- `wrapper.rs` is split by responsibility.
- The crate root exports an explicit facade instead of broad `pub mod` exposure.
- Fallback policy key transitions have crate-local regression coverage.

Suggested gates:

- `cargo nextest run -p fret-render-text`
- targeted renderer-backend tests that exercise fallback snapshots and generic resolution

## M3: `fret-launch` is wiring-only for fonts

Exit criteria:

- The runner font boundary is narrow and shared by desktop/web startup paths.
- Publication of catalog entries, locale, and text stack key happens through explicit helpers with
  no extra policy mixed into runner-specific modules.
- Any bundled-profile seeding policy has a justified home and no longer feels incidental.

Suggested gates:

- `cargo nextest run -p fret-launch`
- desktop/web compile checks for the touched paths

## M4: Cross-crate closure

Exit criteria:

- The diagnostics 3-pack stays green after the refactor.
- The crate audits and ADR alignment evidence describe the post-refactor owner map accurately.
- No new font policy logic leaked into `fret-fonts` or `fret-launch`.

Suggested gates:

- `target\\debug\\fretboard.exe diag run ...` for the promoted font scripts
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
