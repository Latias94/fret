# Menu And Dropdown Diagnostics Packet v1

## Truth

- Material3 `Menu` exposes stable root, root chrome, item, and item chrome selectors.
- Material3 `DropdownMenu` opens a Menu overlay from the dedicated gallery page, gives initial focus
  to the first enabled item, closes on Escape, and restores focus to the trigger.
- Item chrome fills the item bounds for both default and override menu styles.
- The shared dismiss/focus policy remains in `fret-ui-kit`; the Material recipe owns visual chrome
  and test-id stamping.

## Artifacts

- `ecosystem/fret-ui-material3/src/menu.rs`
- `ecosystem/fret-ui-material3/src/dropdown_menu.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The focus/dismiss script opens `ui-gallery-page-material3-menu`, activates
`ui-gallery-material3-menu-trigger`, waits for the Material menu root and item selectors, asserts
focus on the first enabled item, captures an open bundle, presses Escape, and waits for trigger
focus restore. The chrome-fill script keeps the visual item/root chrome evidence for default and
override styles.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json --dir target/fret-diag/material3-menu-focus-dismiss-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json --dir target/fret-diag/material3-menu-item-chrome-fill-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Results:

- focus/dismiss: `PASS`, run id `1779940975756`
- chrome-fill: `PASS`, run id `1779941390986`

Bounded evidence:

- `target/fret-diag/material3-menu-focus-dismiss-20260528/sessions/1779940661967-71588/1779940975756/ai.packet`
- `target/fret-diag/material3-menu-focus-dismiss-20260528/sessions/1779940661967-71588/share/1779940975756.zip`
- `target/fret-diag/material3-menu-item-chrome-fill-20260528/sessions/1779941051623-57416/1779941390986/ai.packet`
- `target/fret-diag/material3-menu-item-chrome-fill-20260528/sessions/1779941051623-57416/share/1779941390986.zip`

## Residual Risk

No current Menu or DropdownMenu recipe, foundation, kit-policy, or mechanism residual remains from
this packet. A shared kit roving/typeahead abstraction should only be split if a second design
system needs the same behavior.
