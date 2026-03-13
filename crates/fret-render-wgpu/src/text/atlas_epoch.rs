#[derive(Default)]
pub(crate) struct TextAtlasEpochState {
    next_epoch: u64,
}

impl TextAtlasEpochState {
    pub(crate) fn new(initial_epoch: u64) -> Self {
        Self {
            next_epoch: initial_epoch.max(1),
        }
    }

    pub(crate) fn next(&mut self) -> u64 {
        let epoch = self.next_epoch;
        self.next_epoch = self.next_epoch.saturating_add(1);
        epoch
    }
}
