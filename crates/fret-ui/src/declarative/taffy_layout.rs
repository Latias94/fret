use super::prelude::*;

#[allow(dead_code)]
pub(crate) fn taffy_dimension(length: Length) -> Dimension {
    match length {
        Length::Auto => Dimension::auto(),
        Length::Fill => Dimension::percent(1.0),
        Length::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            Dimension::percent(f)
        }
        Length::Px(px) => Dimension::length(px.0),
    }
}

pub(crate) fn taffy_position(position: crate::element::PositionStyle) -> TaffyPosition {
    match position {
        crate::element::PositionStyle::Static | crate::element::PositionStyle::Relative => {
            TaffyPosition::Relative
        }
        crate::element::PositionStyle::Absolute => TaffyPosition::Absolute,
    }
}

fn taffy_lpa_from_inset_edge(edge: crate::element::InsetEdge) -> LengthPercentageAuto {
    match edge {
        crate::element::InsetEdge::Px(px) => LengthPercentageAuto::length(px.0),
        crate::element::InsetEdge::Fill => LengthPercentageAuto::percent(1.0),
        crate::element::InsetEdge::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            LengthPercentageAuto::percent(f)
        }
        crate::element::InsetEdge::Auto => LengthPercentageAuto::auto(),
    }
}

pub(crate) fn taffy_rect_lpa_from_inset(
    position: crate::element::PositionStyle,
    inset: crate::element::InsetStyle,
) -> TaffyRect<LengthPercentageAuto> {
    if position == crate::element::PositionStyle::Static {
        return TaffyRect {
            left: LengthPercentageAuto::auto(),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
    }
    TaffyRect {
        left: taffy_lpa_from_inset_edge(inset.left),
        right: taffy_lpa_from_inset_edge(inset.right),
        top: taffy_lpa_from_inset_edge(inset.top),
        bottom: taffy_lpa_from_inset_edge(inset.bottom),
    }
}

fn taffy_lpa_margin_edge(edge: crate::element::MarginEdge) -> LengthPercentageAuto {
    match edge {
        crate::element::MarginEdge::Px(px) => LengthPercentageAuto::length(px.0),
        crate::element::MarginEdge::Fill => LengthPercentageAuto::percent(1.0),
        crate::element::MarginEdge::Fraction(f) => {
            let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
            LengthPercentageAuto::percent(f)
        }
        crate::element::MarginEdge::Auto => LengthPercentageAuto::auto(),
    }
}

pub(crate) fn taffy_rect_lpa_from_margin_edges(
    margin: crate::element::MarginEdges,
) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        left: taffy_lpa_margin_edge(margin.left),
        right: taffy_lpa_margin_edge(margin.right),
        top: taffy_lpa_margin_edge(margin.top),
        bottom: taffy_lpa_margin_edge(margin.bottom),
    }
}

pub(crate) fn taffy_grid_line(line: crate::element::GridLine) -> TaffyLine<GridPlacement> {
    let start = line
        .start
        .map(taffy::style_helpers::line::<GridPlacement>)
        .unwrap_or(GridPlacement::Auto);
    let end = line
        .span
        .map(GridPlacement::Span)
        .unwrap_or(GridPlacement::Auto);
    TaffyLine { start, end }
}

pub(crate) fn taffy_grid_template(
    explicit: Option<&[crate::element::GridTrackSizing]>,
    repeat_count: Option<u16>,
) -> Vec<taffy::style::GridTemplateComponent<String>> {
    if let Some(explicit) = explicit.filter(|tracks| !tracks.is_empty()) {
        return explicit
            .iter()
            .copied()
            .map(|track| {
                taffy::style::GridTemplateComponent::Single(taffy_grid_track_sizing(track))
            })
            .collect();
    }

    repeat_count
        .filter(|count| *count > 0)
        .map(taffy::style_helpers::evenly_sized_tracks)
        .unwrap_or_default()
}

fn taffy_grid_track_sizing(
    track: crate::element::GridTrackSizing,
) -> taffy::style::TrackSizingFunction {
    match track {
        crate::element::GridTrackSizing::Auto => taffy::style_helpers::auto(),
        crate::element::GridTrackSizing::MinContent => taffy::style_helpers::min_content(),
        crate::element::GridTrackSizing::MaxContent => taffy::style_helpers::max_content(),
        crate::element::GridTrackSizing::Px(px) => taffy::style_helpers::length(px.0.max(0.0)),
        crate::element::GridTrackSizing::Fr(fr) => {
            let fr = if fr.is_finite() { fr.max(0.0) } else { 0.0 };
            taffy::style_helpers::fr(fr)
        }
        crate::element::GridTrackSizing::Flex(fr) => {
            let fr = if fr.is_finite() { fr.max(0.0) } else { 0.0 };
            taffy::style_helpers::flex(fr)
        }
    }
}

pub(crate) fn taffy_align_items(align: CrossAlign) -> TaffyAlignItems {
    match align {
        CrossAlign::Start => TaffyAlignItems::FlexStart,
        CrossAlign::Center => TaffyAlignItems::Center,
        CrossAlign::End => TaffyAlignItems::FlexEnd,
        CrossAlign::Stretch => TaffyAlignItems::Stretch,
    }
}

pub(crate) fn taffy_align_self(align: CrossAlign) -> TaffyAlignSelf {
    match align {
        CrossAlign::Start => TaffyAlignSelf::FlexStart,
        CrossAlign::Center => TaffyAlignSelf::Center,
        CrossAlign::End => TaffyAlignSelf::FlexEnd,
        CrossAlign::Stretch => TaffyAlignSelf::Stretch,
    }
}

pub(crate) fn taffy_justify(justify: MainAlign) -> JustifyContent {
    match justify {
        MainAlign::Start => JustifyContent::FlexStart,
        MainAlign::Center => JustifyContent::Center,
        MainAlign::End => JustifyContent::FlexEnd,
        MainAlign::SpaceBetween => JustifyContent::SpaceBetween,
        MainAlign::SpaceAround => JustifyContent::SpaceAround,
        MainAlign::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

pub(crate) fn apply_grid_item_fill_semantics(
    style: &mut TaffyStyle,
    layout_style: crate::element::LayoutStyle,
) {
    if layout_style.position == crate::element::PositionStyle::Absolute {
        return;
    }

    if matches!(layout_style.size.width, crate::element::Length::Fill) {
        style.size.width = Dimension::auto();
        style.justify_self = Some(TaffyAlignSelf::Stretch);
    }

    if matches!(layout_style.size.height, crate::element::Length::Fill) {
        style.size.height = Dimension::auto();
        style.align_self = Some(TaffyAlignSelf::Stretch);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct TaffyMeasureKey {
    pub(super) child: NodeId,
    pub(super) known_w: Option<u32>,
    pub(super) known_h: Option<u32>,
    pub(super) avail_w: (u8, u32),
    pub(super) avail_h: (u8, u32),
}

pub(crate) fn taffy_available_space_key(avail: TaffyAvailableSpace) -> (u8, u32) {
    match avail {
        TaffyAvailableSpace::Definite(v) => (0, v.to_bits()),
        TaffyAvailableSpace::MinContent => (1, 0),
        TaffyAvailableSpace::MaxContent => (2, 0),
    }
}
