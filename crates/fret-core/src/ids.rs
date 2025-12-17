use slotmap::new_key_type;

new_key_type! {
    pub struct AppWindowId;
    pub struct NodeId;
    pub struct DockNodeId;
    pub struct PanelId;
    pub struct ImageId;
    pub struct FontId;
    pub struct TextBlobId;
    pub struct RenderTargetId;
}
