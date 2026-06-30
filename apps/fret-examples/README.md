# fret-examples

Shared harness code used by runnable shells (for example `apps/fret-demo`, `apps/fret-demo-web`, and other
app-level test harnesses).

This crate is intentionally **not** the primary onboarding surface. If you are looking for something to run:

- Templates ladder (recommended): [docs/examples/README.md](../../docs/examples/README.md)
- Second-hour public app scaffold: `fretboard new workbench-lite --name my-workbench`
- Cookbook lessons: `apps/fret-cookbook` (`cargo run -p fret-cookbook --example ...`)
- UI gallery app: `apps/fret-ui-gallery` (`cargo run -p fret-ui-gallery`)
- Demo shells: `apps/fret-demo` / `apps/fret-demo-web`

`api_workbench_lite_demo` remains an advanced proof surface because it intentionally exercises
lower-level runtime seams. Use the `workbench-lite` scaffold when the goal is copyable public app
authoring through `fret::app::prelude::*`.
