use std::sync::Arc;

use fret_core::Px;

#[derive(Clone)]
pub struct VirtualListOptions {
    /// Bounded viewport height for the virtualized list surface.
    pub viewport_height: Px,
    /// Estimated row height used by the runtime virtualizer.
    pub estimate_row_height: Px,
    /// Overscan row count per side.
    pub overscan: usize,
    /// Caller-provided revision bump when item identities or row-height inputs change.
    pub items_revision: u64,
    /// Runtime measure mode.
    pub measure_mode: fret_ui::element::VirtualListMeasureMode,
    /// Runtime key-cache policy.
    pub key_cache: fret_ui::element::VirtualListKeyCacheMode,
    /// Number of off-window rows a retained host may keep alive.
    pub keep_alive: usize,
    /// Inter-row gap owned by the runtime virtualizer.
    pub gap: Px,
    /// Virtualizer scroll-margin offset.
    pub scroll_margin: Px,
    /// Optional known row-height callback used when `measure_mode == Known`.
    pub known_row_height_at: Option<Arc<dyn Fn(usize) -> Px + Send + Sync>>,
    /// Optional external scroll handle.
    pub handle: Option<fret_ui::scroll::VirtualListScrollHandle>,
    pub test_id: Option<Arc<str>>,
}
