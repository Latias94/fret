#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedUiEventV1 {
    pub tick_id: u64,
    pub frame_id: u64,
    pub window: u64,
    pub kind: String,
    pub position: Option<PointV1>,
    pub debug: String,
}

impl RecordedUiEventV1 {
    fn from_event(app: &App, window: AppWindowId, event: &Event, redact_text: bool) -> Self {
        let kind = event_kind(event);
        let position = event.pointer_event().map(|p| PointV1::from(p.position()));
        let debug = event_debug_string(event, redact_text);

        Self {
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window: window.data().as_ffi(),
            kind,
            position,
            debug,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointV1 {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for PointV1 {
    fn from(value: Point) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RectV1 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<Rect> for RectV1 {
    fn from(value: Rect) -> Self {
        Self {
            x: value.origin.x.0,
            y: value.origin.y.0,
            w: value.size.width.0,
            h: value.size.height.0,
        }
    }
}
