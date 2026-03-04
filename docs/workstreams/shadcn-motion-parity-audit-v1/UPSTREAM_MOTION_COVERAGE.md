# Upstream Motion Coverage (repo-ref/ui)

Last updated: 2026-03-04.

This table is a checklist derived from `repo-ref/ui` component sources that contain motion-related
classes (`transition-*`, `animate-*`, `duration-*`, etc.).

The "Heuristic: motion keywords" column is a grep-based signal only; treat it as a prompt for
manual review (it can produce false positives/negatives).

| Upstream file | Priority | Motion cues | Fret impl | Heuristic: motion keywords | Notes |
| --- | --- | --- | --- | --- | --- |
| accordion.tsx | P0 | animate-accordion-up, animate-accordion-down, transition-all, transition-transform, duration-200 | ecosystem/fret-ui-shadcn/src/accordion.rs | Yes |  |
| alert-dialog.tsx | P0 | animate-in, animate-out, duration-200, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95 | ecosystem/fret-ui-shadcn/src/alert_dialog.rs | Yes |  |
| badge.tsx | P1 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/badge.rs | Yes |  |
| breadcrumb.tsx | P1 | transition-colors | ecosystem/fret-ui-shadcn/src/breadcrumb.rs | Yes |  |
| button.tsx | P1 | transition-all | ecosystem/fret-ui-shadcn/src/button.rs | Yes |  |
| checkbox.tsx | P0 | transition-shadow, transition-none | ecosystem/fret-ui-shadcn/src/checkbox.rs | Yes |  |
| combobox.tsx | P0 | animate-in, animate-out, transition-[color,box-shadow], duration-100, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95 | ecosystem/fret-ui-shadcn/src/combobox.rs | Yes |  |
| context-menu.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/context_menu.rs | Yes |  |
| dialog.tsx | P0 | animate-in, animate-out, transition-opacity, duration-200, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95 | ecosystem/fret-ui-shadcn/src/dialog.rs | Yes |  |
| drawer.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0 | ecosystem/fret-ui-shadcn/src/drawer.rs | Yes |  |
| dropdown-menu.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/dropdown_menu.rs | Yes |  |
| hover-card.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/hover_card.rs | Yes |  |
| input-group.tsx | P1 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/input_group.rs | Yes |  |
| input-otp.tsx | P0 | animate-caret-blink, transition-all, duration-1000 | ecosystem/fret-ui-shadcn/src/input_otp.rs | Yes |  |
| input.tsx | P0 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/input.rs | Yes |  |
| item.tsx | P1 | transition-colors, duration-100 | ecosystem/fret-ui-shadcn/src/item.rs | Yes |  |
| menubar.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/menubar.rs | Yes |  |
| native-select.tsx | P1 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/native_select.rs | Yes |  |
| navigation-menu.tsx | P0 | animate-in, animate-out, transition-[color,box-shadow], transition-all, duration-300, duration-200, fade-in-0, fade-out-0 | ecosystem/fret-ui-shadcn/src/navigation_menu.rs | Yes |  |
| popover.tsx | P0 | animate-in, animate-out, fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/popover.rs | Yes |  |
| progress.tsx | P1 | transition-all | ecosystem/fret-ui-shadcn/src/progress.rs | Yes |  |
| radio-group.tsx | P0 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/radio_group.rs | Yes |  |
| scroll-area.tsx | P0 | transition-[color,box-shadow], transition-colors | ecosystem/fret-ui-shadcn/src/scroll_area.rs | Yes |  |
| select.tsx | P0 | animate-in, animate-out, transition-[color,box-shadow], fade-out-0, fade-in-0, zoom-out-95, zoom-in-95, slide-in-from-top-2 | ecosystem/fret-ui-shadcn/src/select.rs | Yes |  |
| sheet.tsx | P0 | animate-in, animate-out, transition-opacity, duration-300, duration-500, fade-out-0, fade-in-0, slide-out-to-right | ecosystem/fret-ui-shadcn/src/sheet.rs | Yes |  |
| sidebar.tsx | P0 | transition-[width], transition-[left,right,width], transition-all, transition-[margin,opacity], transition-transform, transition-[width,height,padding], duration-200 | ecosystem/fret-ui-shadcn/src/sidebar.rs | Yes |  |
| skeleton.tsx | P1 | animate-pulse | ecosystem/fret-ui-shadcn/src/skeleton.rs | Yes |  |
| slider.tsx | P0 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/slider.rs | Yes |  |
| sonner.tsx | P1 | animate-spin | ecosystem/fret-ui-shadcn/src/sonner.rs | Yes (kit) | Loading spin is rendered by kit toast renderer. |
| spinner.tsx | P1 | animate-spin | ecosystem/fret-ui-shadcn/src/spinner.rs | Yes |  |
| switch.tsx | P0 | transition-all, transition-transform | ecosystem/fret-ui-shadcn/src/switch.rs | Yes |  |
| table.tsx | P1 | transition-colors | ecosystem/fret-ui-shadcn/src/table.rs | Yes |  |
| tabs.tsx | P0 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/tabs.rs | Yes |  |
| textarea.tsx | P0 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/textarea.rs | Yes |  |
| toggle.tsx | P1 | transition-[color,box-shadow] | ecosystem/fret-ui-shadcn/src/toggle.rs | Yes |  |
| tooltip.tsx | P0 | animate-in, animate-out, fade-in-0, fade-out-0, zoom-in-95, zoom-out-95, slide-in-from-top-2, slide-in-from-right-2 | ecosystem/fret-ui-shadcn/src/tooltip.rs | Yes |  |
