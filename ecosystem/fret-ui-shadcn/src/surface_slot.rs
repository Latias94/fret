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
}

#[derive(Debug, Default)]
struct ShadcnSurfaceSlotProviderState {
    current: Option<ShadcnSurfaceSlot>,
}

pub(crate) fn surface_slot_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ShadcnSurfaceSlot> {
    cx.inherited_state_where::<ShadcnSurfaceSlotProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
}

#[track_caller]
pub(crate) fn with_surface_slot_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    slot: ShadcnSurfaceSlot,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(ShadcnSurfaceSlotProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(slot);
        prev
    });
    let out = f(cx);
    cx.with_state(ShadcnSurfaceSlotProviderState::default, |st| {
        st.current = prev;
    });
    out
}
