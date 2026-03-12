use fret_ui::{ElementContext, UiHost};

/// Lightweight provider surface for shadcn/ui-style "slot selectors".
///
/// Upstream shadcn/ui sometimes uses `data-slot` + CSS selectors to tweak component defaults when
/// nested under specific containers (e.g. Calendar becomes `bg-transparent` inside PopoverContent).
///
/// Fret does not have CSS selectors, so we model the same boundary via inherited element state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShadcnSurfaceSlot {
    PopoverContent,
    CardContent,
    TooltipContent,
}

pub(crate) fn surface_slot_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ShadcnSurfaceSlot> {
    cx.provided::<ShadcnSurfaceSlot>().copied()
}

#[track_caller]
pub(crate) fn with_surface_slot_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    slot: ShadcnSurfaceSlot,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(slot, f)
}
