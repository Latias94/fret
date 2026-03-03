# Apps (Runnable harnesses)

`apps/` contains runnable binaries and end-to-end harness shells that exercise the full stack.
These are not stable library surfaces; they exist to prove contracts, capture diagnostics, and provide
“what can I run right now?” entry points.

## Recommended entry points (onboarding)

- [`apps/fretboard`](./fretboard) — dev CLI (templates + native/web demo runner).
  - Start here: `cargo run -p fret-cookbook --example hello`
  - Discover: `cargo run -p fretboard -- list native-demos`
- [`apps/fret-cookbook`](./fret-cookbook) — small, copy/paste-ready lessons (`--example ...`).
  - Discover: `cargo run -p fretboard -- list cookbook-examples`
  - Run: `cargo run -p fret-cookbook --example simple_todo`
- [`apps/fret-ui-gallery`](./fret-ui-gallery) — component catalog + conformance app (heavier; not the first step).
  - Run: `cargo run -p fret-ui-gallery`

## Maintainer harnesses (not the onboarding path)

- [`apps/fret-demo`](./fret-demo) — native demo harness shell (broad demo set; useful for maintainers).
- [`apps/fret-demo-web`](./fret-demo-web) — web/wasm demo harness shell (Trunk + WebGPU).

## How these fit together

- [`apps/fret-examples`](./fret-examples) is shared harness code used by demo shells; it is not intended to be run directly.
- Many other `apps/*` crates are maintainer tools (stress harnesses, debug utilities, diagnostics exporters).
  If you are onboarding, prefer the “boring ladder” in [docs/examples/README.md](../docs/examples/README.md).
