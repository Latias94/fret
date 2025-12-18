use crate::{TextBlobId, geometry::Px, ids::FontId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextConstraints {
    pub max_width: Option<Px>,
    pub wrap: TextWrap,
}

impl Default for TextConstraints {
    fn default() -> Self {
        Self {
            max_width: None,
            wrap: TextWrap::Word,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font: FontId,
    pub size: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub size: crate::Size,
    pub baseline: Px,
}

pub trait TextService {
    fn prepare(
        &mut self,
        text: &str,
        style: TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics);

    fn release(&mut self, blob: TextBlobId);
}
