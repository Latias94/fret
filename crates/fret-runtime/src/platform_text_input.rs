use fret_core::{Point, Rect};

/// UTF-16 code unit range used by platform text input and accessibility bridges.
///
/// Coordinate model: UTF-16 code units over a widget's **composed view** (base buffer text with
/// active IME preedit spliced at the caret).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Utf16Range {
    pub start: u32,
    pub end: u32,
}

impl Utf16Range {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn normalized(self) -> Self {
        Self {
            start: self.start.min(self.end),
            end: self.start.max(self.end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Minimal platform text input handler surface (Zed/GPUI-inspired).
///
/// v1: indices are UTF-16 code units over the composed view.
#[derive(Debug, Clone, PartialEq)]
pub enum PlatformTextInputQuery {
    SelectedTextRange,
    MarkedTextRange,
    TextForRange { range: Utf16Range },
    BoundsForRange { range: Utf16Range },
    CharacterIndexForPoint { point: Point },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformTextInputQueryResult {
    Range(Option<Utf16Range>),
    Text(Option<String>),
    Bounds(Option<Rect>),
    Index(Option<u32>),
}
