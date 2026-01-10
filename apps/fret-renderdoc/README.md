# fret-renderdoc

Developer tool for inspecting `fret-render` passes inside a RenderDoc capture (`.rdc`) via
`qrenderdoc --python`.

This is intended to make renderer debugging repeatable and scriptable:

- find draw/dispatch events by marker substring,
- export the selected event's render target output(s) as PNG,
- dump key uniform/storage buffers (best-effort) to JSON so the pass can be validated against our
  rendering contracts (viewport/scissor, clip stack, mask viewport mapping).

## Requirements

- RenderDoc installed (`qrenderdoc` + replay support for the capture backend).
- A capture file (`.rdc`) produced by the demo/runner.

If auto-detection does not find RenderDoc, set:

- `RENDERDOG_RENDERDOC_DIR=<RenderDoc install root>`

Or pass `--renderdoc-dir`.

## Usage

From the repo root:

```bash
cargo run -p fret-renderdoc -- dump --capture <path.rdc> --marker "<substring>"
```

Useful options:

- `--renderdoc-dir <dir>`: RenderDoc install root (contains `qrenderdoc`).
- `--selection <first|last|all>`: choose which matching event(s) to dump (default: `last`).
- `--out <dir>`: output dir (default: `.fret/renderdoc-inspect/<timestamp>`).
- `--basename <name>`: output file basename (default: `fret_dump`).

## Output

The command prints the path to `fret_dump_pass_state_json.response.json`.

The output directory contains:

- `fret_dump_pass_state_json.request.json`: the request passed to the script
- `fret_dump_pass_state_json.response.json`: the script response (matches + dumps)
- `<basename>.event<id>.rt<i>.png`: exported render target output(s)

Note: single-channel mask targets (e.g. `R8Unorm`) may appear as "solid red" in the exported PNG.
That's expected: RenderDoc maps the single channel to RGB for visualization.

