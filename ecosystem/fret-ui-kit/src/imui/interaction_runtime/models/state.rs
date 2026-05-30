use fret_ui::GlobalElementId;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::imui) struct LongPressSignalState {
    pub(in crate::imui) timer: Option<fret_runtime::TimerToken>,
    pub(in crate::imui) holding: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::imui) struct ImUiLifecycleSessionState {
    pub(in crate::imui) active: bool,
    pub(in crate::imui) edited_during_active: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::imui) struct ImUiActiveItemState {
    pub(in crate::imui) active: Option<GlobalElementId>,
}
