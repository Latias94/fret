# Fret Examples Build Latency v1 - M2 Demo Build Split Decision - 2026-04-29

Status: complete

## Decision

Use separate example-family crates for heavy demo families. The first proof is
`apps/fret-examples-imui`, which owns standalone IMUI demo sources and exposes direct bins.

`apps/fret-examples` may temporarily re-export the moved module while older demo selectors migrate,
but the fast iteration target must not depend on `fret-examples`.

## Rationale

- `apps/fret-demo/src/bin/*` targets live in one Cargo package, so package dependencies are not a
  clean per-bin isolation boundary.
- Feature-family gating would require users and tools to remember the right feature/no-default
  feature combination for each demo target.
- A separate family crate gives each heavy demo family its own dependency closure, runnable bins, and
  source-policy gates without changing framework contracts.

## Proof Slice

- Moved `imui_hello_demo`, `imui_floating_windows_demo`, and `imui_shadcn_adapter_demo` source
  ownership from `apps/fret-examples/src/` to `apps/fret-examples-imui/src/`.
- Added direct bins under `apps/fret-examples-imui/src/bin/`.
- Kept the legacy `fret_examples::imui_shadcn_adapter_demo` path as a re-export from
  `apps/fret-examples/src/lib.rs`.
- Updated source-policy gates to read the new owner path.
- Checked `cargo tree -p fret-examples-imui -e normal`; it has no `fret-examples v...`
  dependency entry.

## Gates

```text
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples-imui --bins --jobs 1
cargo check -p fret-demo --bin imui_hello_demo --bin imui_floating_windows_demo --bin imui_shadcn_adapter_demo --jobs 1
cargo check -p fret-examples --lib --jobs 1
```

## Next Step

Migrate `imui_response_signals_demo` and `imui_interaction_showcase_demo` next; they carry more
diagnostic/documentation references, so keep them in a separate slice from the low-coupling demos.
