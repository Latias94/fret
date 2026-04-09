# View-Locals Authoring (Fearless Refactor v1) — TODO

Status: closed
Last updated: 2026-03-20

## Docs and planning

- [x] Add this lane to `docs/README.md`.
- [x] Add this lane to `docs/roadmap.md`.
- [x] Add this lane to `docs/workstreams/README.md`.
- [x] Update `docs/authoring-golden-path-v2.md` with the `1-2 inline / 3+ bundle` rule.
- [x] Update `docs/examples/todo-app-golden-path.md` to teach `*Locals::new(cx)` and optional
      `bind_actions(&self, cx)`.
- [x] Update `ecosystem/fret/README.md` so the default action guidance matches the new rule.

## Proof surfaces

- [x] Migrate `apps/fret-examples/src/todo_demo.rs`.
- [x] Migrate `apps/fret-examples/src/simple_todo_demo.rs`.
- [x] Migrate `apps/fret-cookbook/examples/simple_todo.rs`.
- [x] Migrate `apps/fret-cookbook/examples/simple_todo_v2_target.rs`.
- [x] Migrate one non-Todo proof surface:
  - [x] `apps/fret-cookbook/examples/form_basics.rs`

## Template and source-policy tests

- [x] Update `apps/fretboard/src/scaffold/templates.rs` generated source strings.
- [x] Update `apps/fretboard/src/scaffold/templates.rs` template assertions.
- [x] Update `apps/fret-examples/src/lib.rs` authoring-surface assertions.
- [x] Update `apps/fret-cookbook/src/lib.rs` authoring-surface assertions.

## Validation

- [x] `cargo fmt`
- [x] `cargo nextest run -p fret-examples --lib todo_demo_prefers_default_app_surface simple_todo_demo_prefers_default_app_surface canonical_default_app_examples_stay_local_state_first`
- [x] `cargo nextest run -p fret-cookbook --lib onboarding_examples_use_the_new_app_surface migrated_basics_examples_use_the_new_app_surface`
- [x] `cargo nextest run -p fretboard-dev todo_template_uses_default_authoring_dialect simple_todo_template_has_low_adapter_noise_and_no_query_selector template_readmes_capture_authoring_guidance`
- [x] `git diff --check`
- [x] `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
