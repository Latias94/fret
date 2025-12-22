use slotmap::new_key_type;

new_key_type! {
    pub struct AppWindowId;
    pub struct NodeId;
    pub struct DockNodeId;
    pub struct ImageId;
    pub struct FontId;
    pub struct TextBlobId;
    pub struct RenderTargetId;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TickId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalDropToken(pub u64);
