use fret_core::{Point, Px, Rect, Size};
use fret_icons::IconId;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(transparent)]
pub(crate) struct Material3HeadlessBoundsV1([f32; 2]);

impl Material3HeadlessBoundsV1 {
    pub(crate) fn rect(self) -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(self.0[0]), Px(self.0[1])),
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct Material3HeadlessSettleWindowV1 {
    settle_from_frame: usize,
    total_frames: usize,
}

impl Material3HeadlessSettleWindowV1 {
    pub(crate) fn settle_from_frame(self) -> usize {
        self.settle_from_frame
    }

    pub(crate) fn total_frames(self) -> usize {
        self.total_frames
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Material3HeadlessIconV1 {
    ChevronRight,
    MoreHorizontal,
    Play,
    Search,
    Settings,
    Slash,
}

impl Material3HeadlessIconV1 {
    pub(crate) fn icon_id(self) -> IconId {
        match self {
            Self::ChevronRight => fret_icons::ids::ui::CHEVRON_RIGHT,
            Self::MoreHorizontal => fret_icons::ids::ui::MORE_HORIZONTAL,
            Self::Play => fret_icons::ids::ui::PLAY,
            Self::Search => fret_icons::ids::ui::SEARCH,
            Self::Settings => fret_icons::ids::ui::SETTINGS,
            Self::Slash => fret_icons::ids::ui::SLASH,
        }
    }
}

pub(crate) fn assert_material3_headless_schema_version(
    actual: u32,
    expected: u32,
    fixture_label: &str,
) {
    assert_eq!(
        actual, expected,
        "material3 {fixture_label} fixture schema version"
    );
}
