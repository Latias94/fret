# TODO (UI Gallery fearless refactor v1)

## Foundations

- [ ] Add a small helper to reduce boilerplate when wiring snippet-backed sections.
- [ ] Migrate the remaining gallery pages to snippet-backed sections (preview ≡ code).
- [ ] Ensure code blocks are scrollable and layout is consistent across pages.
- [ ] Add a minimal diag suite that asserts “preview ≡ code” pages are wired (smoke selectors + one screenshot each).

## Component tracker (shadcn docs taxonomy)

Status vocabulary:

- `No`: not snippet-backed
- `Partial`: some examples snippet-backed, but page still has inline code or mismatched preview/code
- `Yes`: preview ≡ code (snippet-backed + region-sliced)

| Component | UI Gallery page | Snippet-backed | Notes |
|---|---|---|---|
| accordion |  | No |  |
| alert |  | No |  |
| alert-dialog |  | No |  |
| aspect-ratio |  | No |  |
| avatar |  | No |  |
| badge |  | No |  |
| breadcrumb |  | No |  |
| button |  | No |  |
| button-group | `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/button_group.rs` | Yes | Snippets live in `apps/fret-ui-gallery/src/ui/snippets/button_group/`. |
| calendar |  | No |  |
| card |  | No |  |
| carousel |  | No |  |
| chart |  | No |  |
| checkbox |  | No |  |
| collapsible |  | No |  |
| combobox |  | No |  |
| command |  | No |  |
| context-menu |  | No |  |
| data-table |  | No |  |
| date-picker |  | No |  |
| dialog |  | No |  |
| direction |  | No |  |
| drawer |  | No |  |
| dropdown-menu |  | No |  |
| empty |  | No |  |
| field |  | No |  |
| hover-card |  | No |  |
| input |  | No |  |
| input-group |  | No |  |
| input-otp |  | No |  |
| item |  | No |  |
| kbd |  | No |  |
| label |  | No |  |
| menubar |  | No |  |
| native-select |  | No |  |
| navigation-menu |  | No |  |
| pagination |  | No |  |
| popover |  | No |  |
| progress |  | No |  |
| radio-group |  | No |  |
| resizable |  | No |  |
| scroll-area |  | No |  |
| select | `apps/fret-ui-gallery/src/ui/previews/gallery/forms/select.rs` | Yes | Snippets live in `apps/fret-ui-gallery/src/ui/snippets/select/`. |
| separator |  | No |  |
| sheet |  | No |  |
| sidebar |  | No |  |
| skeleton |  | No |  |
| slider |  | No |  |
| sonner |  | No |  |
| spinner |  | No |  |
| switch |  | No |  |
| table |  | No |  |
| tabs |  | No |  |
| textarea |  | No |  |
| toast |  | No |  |
| toggle |  | No |  |
| toggle-group |  | No |  |
| tooltip |  | No |  |
| typography |  | No |  |

