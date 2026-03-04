# Open-source onboarding (fearless refactor v1) — Milestones

## M0 — First run is boring (minimum)

Definition of done:

- A first-time user can run at least 2 examples without reading deep docs:
  - `cargo run -p fret-cookbook --example hello`
  - `cargo run -p fret-cookbook --example simple_todo`
- `docs/examples/README.md` clearly points to templates vs cookbook vs gallery vs labs.

## M1 — Curated breadth (onboarding without overload)

Definition of done:

- Cookbook has an explicit “Official ladder” and hides Lab examples behind features.
- UI gallery defaults to a lite catalog and compiles without dev/material3 dependencies unless opted in.
- Diagnostics is presented as optional, with a copy/paste walkthrough (`hello`) and a boring follow-up (`simple_todo`).

## M2 — Dependency + feature audit (polish)

Definition of done:

- `ecosystem/fret` default features are audited and documented, with a clear “minimal vs app vs batteries”
  story.
- README code samples are confirmed current (or replaced with a minimal compile-tested snippet).
