# M1 Contract Freeze — 2026-04-10

Status: closed decision record

Related:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

## Frozen decision

This lane freezes the following owner contract for diagnostics skill evidence:

1. public `fretboard diag ...` help evidence belongs to `crates/fretboard/src/cli/help.rs`;
2. workspace-dev `fretboard-dev diag ...` help evidence belongs to
   `apps/fretboard/src/cli/help.rs`;
3. `fret-diag-workflow` symbol validation must check both owners separately;
4. `fret-diag-workflow` documentation must name both owners explicitly and warn against mixing
   them.

## Consequences

- The validator becomes a real drift guard again instead of a false-positive trap.
- Maintainers do not need to mutate CLI help output just to satisfy a stale owner mapping.
- The public/workspace-dev split remains aligned with the shipped package boundary.

## Explicit non-goals

- Do not add duplicate `fretboard diag run ...` examples to `apps/fretboard/src/cli/help.rs`.
- Do not move workspace-dev `fretboard-dev diag run ...` examples into the public help file.
- Do not expand this fix into broader `fret_skills.py` schema changes.
