#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DragValueMode {
    Scrub,
    Typing,
}

#[derive(Debug)]
pub(super) struct DragValueState {
    pub(super) mode: DragValueMode,
    pub(super) scrub_id: Option<fret_ui::GlobalElementId>,
    pub(super) scrub_revision: u64,
}

impl Default for DragValueState {
    fn default() -> Self {
        Self {
            mode: DragValueMode::Scrub,
            scrub_id: None,
            scrub_revision: 0,
        }
    }
}
