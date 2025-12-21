use fret_core::{Event, Px, Size};
use fret_ui::{EventCx, LayoutCx, PaintCx, TextArea, Widget};

#[derive(Debug, Default)]
pub struct TextProbeService {
    text: String,
    title: Option<String>,
    revision: u64,
}

impl TextProbeService {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn set(&mut self, title: Option<String>, text: String) {
        if self.title == title && self.text == text {
            return;
        }
        self.title = title;
        self.text = text;
        self.revision = self.revision.saturating_add(1);
    }
}

pub struct TextProbePanel {
    area: TextArea,
    last_revision: Option<u64>,
}

impl TextProbePanel {
    pub fn new(initial_text: impl Into<String>) -> Self {
        Self {
            area: TextArea::new(initial_text).with_min_height(Px(240.0)),
            last_revision: None,
        }
    }

    fn maybe_sync(&mut self, cx: &mut LayoutCx<'_>) -> bool {
        let Some(service) = cx.app.global::<TextProbeService>() else {
            return false;
        };
        let rev = service.revision();
        if self.last_revision == Some(rev) {
            return false;
        }
        self.last_revision = Some(rev);
        let area = std::mem::take(&mut self.area);
        self.area = area.with_text(service.text());
        true
    }
}

impl Widget for TextProbePanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        self.area.event(cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let _ = self.maybe_sync(cx);
        self.area.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.area.paint(cx);
    }
}
