use std::collections::HashMap;

use super::column_sizing_info::ColumnSizingInfoState;
use super::{ColumnDef, ColumnId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnSizingRegion {
    All,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnResizeMode {
    OnChange,
    OnEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnResizeDirection {
    Ltr,
    Rtl,
}

/// TanStack-compatible column sizing map: `column_id -> size`.
pub type ColumnSizingState = HashMap<ColumnId, f32>;

pub fn column_size(state: &ColumnSizingState, column: &ColumnId) -> Option<f32> {
    state.get(column).copied()
}

pub fn resolved_column_size<TData>(state: &ColumnSizingState, column: &ColumnDef<TData>) -> f32 {
    let raw = state.get(&column.id).copied().unwrap_or(column.size);
    raw.clamp(column.min_size, column.max_size)
}

pub fn column_can_resize<TData>(options: super::TableOptions, column: &ColumnDef<TData>) -> bool {
    options.enable_column_resizing && column.enable_resizing
}

fn round_column_size(size: f32) -> f32 {
    (size * 100.0).round() / 100.0
}

pub fn column_resize_preview_size(info: &ColumnSizingInfoState, column: &ColumnId) -> Option<f32> {
    let delta_percentage = info.delta_percentage?;
    let (_, start) = info
        .column_sizing_start
        .iter()
        .find(|(id, _)| id.as_ref() == column.as_ref())?;
    Some(round_column_size(
        (*start + *start * delta_percentage).max(0.0),
    ))
}

pub fn begin_column_resize(
    info: &mut ColumnSizingInfoState,
    resizing_column: ColumnId,
    start_offset: f32,
    start_size: f32,
    column_sizing_start: Vec<(ColumnId, f32)>,
) {
    *info = ColumnSizingInfoState {
        start_offset: Some(start_offset),
        start_size: Some(start_size),
        delta_offset: Some(0.0),
        delta_percentage: Some(0.0),
        is_resizing_column: Some(resizing_column),
        column_sizing_start,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizeEvent {
    Move,
    End,
}

fn update_column_resize(
    mode: ColumnResizeMode,
    direction: ColumnResizeDirection,
    sizing: &mut ColumnSizingState,
    info: &mut ColumnSizingInfoState,
    event: ResizeEvent,
    client_x: Option<f32>,
) {
    let Some(client_x) = client_x else { return };

    let Some(start_offset) = info.start_offset else {
        return;
    };
    let Some(start_size) = info.start_size else {
        return;
    };
    if start_size <= 0.0 {
        return;
    }

    let direction_mul = match direction {
        ColumnResizeDirection::Ltr => 1.0,
        ColumnResizeDirection::Rtl => -1.0,
    };

    let delta_offset = (client_x - start_offset) * direction_mul;
    let delta_percentage = (delta_offset / start_size).max(-0.999_999);

    info.delta_offset = Some(delta_offset);
    info.delta_percentage = Some(delta_percentage);

    if !(mode == ColumnResizeMode::OnChange || event == ResizeEvent::End) {
        return;
    }

    for (column_id, header_size) in &info.column_sizing_start {
        let next = (header_size + header_size * delta_percentage).max(0.0);
        sizing.insert(column_id.clone(), round_column_size(next));
    }
}

pub fn drag_column_resize(
    mode: ColumnResizeMode,
    direction: ColumnResizeDirection,
    sizing: &mut ColumnSizingState,
    info: &mut ColumnSizingInfoState,
    client_x: f32,
) {
    update_column_resize(
        mode,
        direction,
        sizing,
        info,
        ResizeEvent::Move,
        Some(client_x),
    );
}

pub fn end_column_resize(
    mode: ColumnResizeMode,
    direction: ColumnResizeDirection,
    sizing: &mut ColumnSizingState,
    info: &mut ColumnSizingInfoState,
    client_x: Option<f32>,
) {
    update_column_resize(mode, direction, sizing, info, ResizeEvent::End, client_x);
    *info = ColumnSizingInfoState::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::TableOptions;

    #[test]
    fn column_size_reads_from_map() {
        let mut state = ColumnSizingState::default();
        state.insert(ColumnId::from("a"), 123.0);

        assert_eq!(column_size(&state, &ColumnId::from("a")), Some(123.0));
        assert_eq!(column_size(&state, &ColumnId::from("b")), None);
    }

    #[test]
    fn resolved_column_size_falls_back_to_column_default_and_clamps() {
        #[derive(Debug)]
        struct Item;

        let col = ColumnDef::<Item>::new("a")
            .size(100.0)
            .min_size(60.0)
            .max_size(80.0);

        let state = ColumnSizingState::default();
        assert_eq!(resolved_column_size(&state, &col), 80.0);

        let mut state = ColumnSizingState::default();
        state.insert(ColumnId::from("a"), 10.0);
        assert_eq!(resolved_column_size(&state, &col), 60.0);
    }

    #[test]
    fn column_can_resize_respects_table_and_column_flags() {
        #[derive(Debug)]
        struct Item;

        let col = ColumnDef::<Item>::new("a").enable_resizing(false);
        assert!(!column_can_resize(TableOptions::default(), &col));

        let mut options = TableOptions::default();
        options.enable_column_resizing = false;
        let col = ColumnDef::<Item>::new("a").enable_resizing(true);
        assert!(!column_can_resize(options, &col));
    }

    #[test]
    fn column_resize_on_change_updates_sizing_on_move_and_resets_on_end() {
        let mut sizing = ColumnSizingState::default();
        let mut info = ColumnSizingInfoState::default();

        begin_column_resize(
            &mut info,
            ColumnId::from("a"),
            10.0,
            100.0,
            vec![(ColumnId::from("a"), 100.0)],
        );

        drag_column_resize(
            ColumnResizeMode::OnChange,
            ColumnResizeDirection::Ltr,
            &mut sizing,
            &mut info,
            60.0,
        );

        assert_eq!(sizing.get(&ColumnId::from("a")).copied(), Some(150.0));
        assert_eq!(info.delta_offset, Some(50.0));
        assert_eq!(info.delta_percentage, Some(0.5));
        assert_eq!(
            info.is_resizing_column.as_ref().map(|s| s.as_ref()),
            Some("a")
        );

        end_column_resize(
            ColumnResizeMode::OnChange,
            ColumnResizeDirection::Ltr,
            &mut sizing,
            &mut info,
            Some(60.0),
        );

        assert_eq!(sizing.get(&ColumnId::from("a")).copied(), Some(150.0));
        assert!(info.is_resizing_column.is_none());
        assert!(info.start_offset.is_none());
        assert!(info.start_size.is_none());
        assert!(info.delta_offset.is_none());
        assert!(info.delta_percentage.is_none());
        assert!(info.column_sizing_start.is_empty());
    }

    #[test]
    fn column_resize_on_end_only_writes_sizing_at_end() {
        let mut sizing = ColumnSizingState::default();
        let mut info = ColumnSizingInfoState::default();

        begin_column_resize(
            &mut info,
            ColumnId::from("a"),
            0.0,
            100.0,
            vec![(ColumnId::from("a"), 100.0)],
        );

        drag_column_resize(
            ColumnResizeMode::OnEnd,
            ColumnResizeDirection::Ltr,
            &mut sizing,
            &mut info,
            50.0,
        );

        assert!(sizing.is_empty());
        assert_eq!(info.delta_offset, Some(50.0));
        assert_eq!(info.delta_percentage, Some(0.5));

        end_column_resize(
            ColumnResizeMode::OnEnd,
            ColumnResizeDirection::Ltr,
            &mut sizing,
            &mut info,
            Some(50.0),
        );

        assert_eq!(sizing.get(&ColumnId::from("a")).copied(), Some(150.0));
        assert!(info.is_resizing_column.is_none());
    }

    #[test]
    fn begin_column_resize_uses_resizing_column_size_for_start_size_when_present() {
        let mut info = ColumnSizingInfoState::default();

        begin_column_resize(
            &mut info,
            ColumnId::from("ab"),
            0.0,
            150.0,
            vec![
                (ColumnId::from("a"), 100.0),
                (ColumnId::from("b"), 50.0),
                (ColumnId::from("ab"), 150.0),
            ],
        );

        assert_eq!(info.start_size, Some(150.0));
    }

    #[test]
    fn column_resize_rtl_flips_delta_direction() {
        let mut sizing = ColumnSizingState::default();
        let mut info = ColumnSizingInfoState::default();

        begin_column_resize(
            &mut info,
            ColumnId::from("a"),
            0.0,
            100.0,
            vec![(ColumnId::from("a"), 100.0)],
        );

        drag_column_resize(
            ColumnResizeMode::OnChange,
            ColumnResizeDirection::Rtl,
            &mut sizing,
            &mut info,
            50.0,
        );

        assert_eq!(sizing.get(&ColumnId::from("a")).copied(), Some(50.0));
        assert_eq!(info.delta_offset, Some(-50.0));
        assert_eq!(info.delta_percentage, Some(-0.5));
    }

    #[test]
    fn column_resize_preview_reads_from_info_without_touching_sizing() {
        let mut sizing = ColumnSizingState::default();
        let mut info = ColumnSizingInfoState::default();

        begin_column_resize(
            &mut info,
            ColumnId::from("a"),
            0.0,
            100.0,
            vec![(ColumnId::from("a"), 100.0)],
        );

        drag_column_resize(
            ColumnResizeMode::OnEnd,
            ColumnResizeDirection::Ltr,
            &mut sizing,
            &mut info,
            50.0,
        );

        assert!(sizing.is_empty());
        assert_eq!(
            column_resize_preview_size(&info, &ColumnId::from("a")),
            Some(150.0)
        );
    }
}
