use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::{Color, KeyCode, Modifiers, Point, PointerId, Px, Rect, Size};
use fret_runtime::{Model, TimerToken};
use fret_ui::action::{UiActionHostExt as _, UiFocusActionHost};
use fret_ui::element::Length;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, TextField, TextFieldBlurBehavior, TextFieldOptions,
    TextFieldOutcome,
};
use fret_ui_editor::primitives::EditSessionOutcome;
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
};

use super::{
    KernelApp, named_demo_state, proof_compact_readout_element, proof_drag_preview_card,
    proof_section_chrome_label,
};

mod geometry;
mod readouts;

use geometry::{
    PROOF_COLLECTION_GRID_FALLBACK_COLUMNS, PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX,
    proof_collection_drag_rect, proof_collection_drag_threshold_met,
    proof_collection_layout_metrics, proof_collection_localize_rect,
    proof_collection_rects_intersect, proof_collection_zoom_line, proof_collection_zoom_request,
};
use readouts::{
    proof_collection_active_line, proof_collection_assets_line,
    proof_collection_command_package_line, proof_collection_command_status_line,
    proof_collection_context_menu_line, proof_collection_delete_status,
    proof_collection_duplicate_status, proof_collection_rename_cancel_status,
    proof_collection_rename_commit_status, proof_collection_rename_invalid_status,
    proof_collection_rename_line, proof_collection_rename_ready_status,
    proof_collection_rename_status_line, proof_collection_select_all_line,
    proof_collection_select_all_status, proof_collection_selection_line,
    proof_collection_visible_order_line,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionAsset {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) path: Arc<str>,
    pub(super) kind: Arc<str>,
    pub(super) size_kib: u32,
}

#[derive(Clone)]
struct ProofCollectionDragPayload {
    lead_label: Arc<str>,
    lead_path: Arc<str>,
    asset_ids: Arc<[Arc<str>]>,
    asset_paths: Arc<[Arc<str>]>,
}

#[derive(Debug, Clone, PartialEq)]
struct ProofCollectionRenderedItem {
    id: Arc<str>,
    local_bounds: Rect,
}

#[derive(Debug, Clone, PartialEq)]
struct ProofCollectionBoxSelectSession {
    pointer_id: PointerId,
    origin_local: Point,
    current_local: Point,
    baseline_selected: Vec<Arc<str>>,
    append_mode: bool,
    threshold_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ProofCollectionBoxSelectState {
    session: Option<ProofCollectionBoxSelectSession>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProofCollectionKeyboardState {
    active_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofCollectionRenameSession {
    target_id: Arc<str>,
    original_label: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofCollectionRenameCommit {
    target_id: Arc<str>,
    previous_label: Arc<str>,
    next_label: Arc<str>,
    renamed_assets: Vec<ProofCollectionAsset>,
}

#[derive(Debug, Default)]
struct ProofCollectionInlineRenameFocusState {
    timer: Option<TimerToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofCollectionDeleteResult {
    remaining_assets: Vec<ProofCollectionAsset>,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
    deleted_assets: Vec<ProofCollectionAsset>,
    deleted_ids: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofCollectionDuplicateResult {
    next_assets: Vec<ProofCollectionAsset>,
    duplicated_assets: Vec<ProofCollectionAsset>,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
}

fn proof_collection_assets_in_visible_order(
    assets: Arc<[ProofCollectionAsset]>,
    reverse_order: bool,
) -> Vec<ProofCollectionAsset> {
    let mut visible = assets.iter().cloned().collect::<Vec<_>>();
    if reverse_order {
        visible.reverse();
    }
    visible
}

fn proof_collection_selected_assets<'a>(
    assets: &'a [ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> Vec<&'a ProofCollectionAsset> {
    let by_id = assets
        .iter()
        .map(|asset| (asset.id.as_ref(), asset))
        .collect::<HashMap<_, _>>();

    selection
        .selected()
        .iter()
        .filter_map(|id| by_id.get(id.as_ref()).copied())
        .collect()
}

fn proof_collection_active_id(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<Arc<str>> {
    let contains = |id: &Arc<str>| collection_keys.iter().any(|key| key == id);

    keyboard
        .active_id
        .clone()
        .filter(contains)
        .or_else(|| selection.anchor().cloned().filter(contains))
        .or_else(|| selection.first_selected().cloned().filter(contains))
        .or_else(|| collection_keys.first().cloned())
}

fn proof_collection_select_all_shortcut_matches(key: KeyCode, modifiers: Modifiers) -> bool {
    matches!(key, KeyCode::KeyA)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

fn proof_collection_duplicate_shortcut_matches(key: KeyCode, modifiers: Modifiers) -> bool {
    matches!(key, KeyCode::KeyD)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

fn proof_collection_select_all_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<(ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState)> {
    let contains = |id: &Arc<str>| collection_keys.iter().any(|key| key == id);
    let next_active = proof_collection_active_id(collection_keys, selection, keyboard)?;
    let next_anchor = selection
        .anchor()
        .cloned()
        .filter(contains)
        .or_else(|| collection_keys.first().cloned());

    Some((
        ImUiMultiSelectState::from_ordered_selection(
            collection_keys,
            collection_keys.to_vec(),
            next_anchor,
        ),
        ProofCollectionKeyboardState {
            active_id: Some(next_active),
        },
    ))
}

fn proof_collection_rename_shortcut_matches(key: KeyCode, modifiers: Modifiers) -> bool {
    key == KeyCode::F2 && modifiers == Modifiers::default()
}

fn proof_collection_begin_rename_session(
    visible_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<ProofCollectionRenameSession> {
    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&visible_keys, selection, keyboard)?;
    let asset = visible_assets.iter().find(|asset| asset.id == active_id)?;

    Some(ProofCollectionRenameSession {
        target_id: asset.id.clone(),
        original_label: asset.label.clone(),
    })
}

fn proof_collection_begin_inline_rename_in_app(
    app: &mut KernelApp,
    rename_session_model: &Model<Option<ProofCollectionRenameSession>>,
    rename_draft_model: &Model<String>,
    rename_focus_pending_model: &Model<bool>,
    rename_status_model: &Model<String>,
    session: &ProofCollectionRenameSession,
) {
    let _ = app.models_mut().update(rename_session_model, |state| {
        *state = Some(session.clone());
    });
    let _ = app.models_mut().update(rename_draft_model, |draft| {
        draft.clear();
        draft.push_str(session.original_label.as_ref());
    });
    let _ = app
        .models_mut()
        .update(rename_focus_pending_model, |state| {
            *state = true;
        });
    let _ = app.models_mut().update(rename_status_model, |status| {
        status.clear();
        status.push_str(&proof_collection_rename_ready_status(
            session.original_label.as_ref(),
        ));
    });
}

fn proof_collection_commit_rename(
    stored_assets: &[ProofCollectionAsset],
    session: &ProofCollectionRenameSession,
    draft: &str,
) -> Option<ProofCollectionRenameCommit> {
    let next_label = Arc::<str>::from(draft.trim());
    if next_label.is_empty() {
        return None;
    }

    let _target = stored_assets
        .iter()
        .find(|asset| asset.id == session.target_id)?;
    let renamed_assets = stored_assets
        .iter()
        .cloned()
        .map(|mut asset| {
            if asset.id == session.target_id {
                asset.label = next_label.clone();
            }
            asset
        })
        .collect::<Vec<_>>();

    Some(ProofCollectionRenameCommit {
        target_id: session.target_id.clone(),
        previous_label: session.original_label.clone(),
        next_label,
        renamed_assets,
    })
}

#[track_caller]
fn proof_collection_inline_rename_focus_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<ProofCollectionInlineRenameFocusState>> {
    cx.slot_state(
        || Arc::new(Mutex::new(ProofCollectionInlineRenameFocusState::default())),
        |state| state.clone(),
    )
}

fn proof_collection_sync_inline_rename_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    pending_focus: bool,
    pending_focus_model: &Model<bool>,
    focus_state: &Arc<Mutex<ProofCollectionInlineRenameFocusState>>,
) {
    let (cancel_token, arm_token) = {
        let mut state = focus_state.lock().unwrap_or_else(|err| err.into_inner());
        match (pending_focus, state.timer) {
            (true, None) => {
                let token = cx.app.next_timer_token();
                state.timer = Some(token);
                (None, Some(token))
            }
            (false, Some(token)) => {
                state.timer = None;
                (Some(token), None)
            }
            _ => (None, None),
        }
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_token {
        cx.set_timer_for(input_id, token, Duration::ZERO);
    }

    let focus_state_for_timer = focus_state.clone();
    let pending_focus_model_for_timer = pending_focus_model.clone();
    cx.timer_add_on_timer_for(
        input_id,
        Arc::new(move |host, action_cx, token| {
            {
                let mut state = focus_state_for_timer
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                if state.timer != Some(token) {
                    return false;
                }
                state.timer = None;
            }

            let pending = host
                .update_model(&pending_focus_model_for_timer, |value| {
                    std::mem::take(value)
                })
                .unwrap_or(false);
            if !pending {
                return false;
            }

            host.request_focus(input_id);
            host.request_redraw(action_cx.window);
            false
        }),
    );
}

fn proof_collection_restore_focus_after_inline_rename(
    host: &mut dyn UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    focus_target_model: &Model<Option<GlobalElementId>>,
) {
    let target = host
        .models_mut()
        .read(focus_target_model, |state| *state)
        .ok()
        .flatten();
    if let Some(target) = target {
        host.request_focus(target);
        host.request_redraw(action_cx.window);
    }
}

fn proof_collection_context_menu_selection(
    selection: &ImUiMultiSelectState<Arc<str>>,
    asset_id: Arc<str>,
) -> (ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState) {
    let next_selection = if selection.is_selected(&asset_id) {
        selection.clone()
    } else {
        ImUiMultiSelectState::single(asset_id.clone())
    };

    (
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(asset_id),
        },
    )
}

fn proof_collection_keyboard_next_index(
    current: usize,
    len: usize,
    columns: usize,
    key: KeyCode,
) -> Option<usize> {
    let last = len.checked_sub(1)?;
    match key {
        KeyCode::ArrowRight => Some((current + 1).min(last)),
        KeyCode::ArrowLeft => Some(current.saturating_sub(1)),
        KeyCode::ArrowDown => Some((current + columns).min(last)),
        KeyCode::ArrowUp => Some(current.saturating_sub(columns)),
        KeyCode::Home => Some(0),
        KeyCode::End => Some(last),
        _ => None,
    }
}

fn proof_collection_keyboard_move_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    next_id: Arc<str>,
    extend_range: bool,
) -> ImUiMultiSelectState<Arc<str>> {
    if !extend_range {
        return ImUiMultiSelectState::single(next_id);
    }

    let anchor = selection
        .anchor()
        .cloned()
        .unwrap_or_else(|| next_id.clone());
    let Some(anchor_index) = collection_keys.iter().position(|key| key == &anchor) else {
        return ImUiMultiSelectState::single(next_id);
    };
    let Some(next_index) = collection_keys.iter().position(|key| key == &next_id) else {
        return ImUiMultiSelectState::single(next_id);
    };
    let (start, end) = if anchor_index <= next_index {
        (anchor_index, next_index)
    } else {
        (next_index, anchor_index)
    };

    ImUiMultiSelectState::from_ordered_selection(
        collection_keys,
        collection_keys[start..=end].to_vec(),
        Some(anchor),
    )
}

fn proof_collection_keyboard_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    columns: usize,
    key: KeyCode,
    modifiers: Modifiers,
) -> Option<(ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState)> {
    if collection_keys.is_empty() || modifiers.alt || modifiers.ctrl || modifiers.meta {
        return None;
    }

    if key == KeyCode::Escape {
        return Some((
            ImUiMultiSelectState::default(),
            ProofCollectionKeyboardState {
                active_id: proof_collection_active_id(collection_keys, selection, keyboard),
            },
        ));
    }

    let current_id = proof_collection_active_id(collection_keys, selection, keyboard)?;
    let current_index = collection_keys
        .iter()
        .position(|item| item == &current_id)?;
    let next_index =
        proof_collection_keyboard_next_index(current_index, collection_keys.len(), columns, key)?;
    let next_id = collection_keys[next_index].clone();
    let next_selection = proof_collection_keyboard_move_selection(
        collection_keys,
        selection,
        next_id.clone(),
        modifiers.shift,
    );

    Some((
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(next_id),
        },
    ))
}

fn proof_collection_delete_key_matches(key: KeyCode) -> bool {
    matches!(key, KeyCode::Delete | KeyCode::Backspace)
}

fn proof_collection_delete_selection(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<ProofCollectionDeleteResult> {
    let deleted_assets = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let deleted_ids = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    if deleted_ids.is_empty() {
        return None;
    }

    let deleted_contains = |id: &Arc<str>| deleted_ids.iter().any(|item| item == id);
    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let focus_source_index = proof_collection_active_id(&visible_keys, selection, keyboard)
        .and_then(|id| visible_keys.iter().position(|key| key == &id))
        .or_else(|| {
            deleted_ids
                .last()
                .and_then(|id| visible_keys.iter().position(|key| key == id))
        })
        .unwrap_or(0);

    let remaining_visible = visible_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let remaining_assets = stored_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let next_active = if remaining_visible.is_empty() {
        None
    } else {
        Some(
            remaining_visible[focus_source_index.min(remaining_visible.len() - 1)]
                .id
                .clone(),
        )
    };
    let next_selection = next_active
        .clone()
        .map(ImUiMultiSelectState::single)
        .unwrap_or_default();

    Some(ProofCollectionDeleteResult {
        remaining_assets,
        next_selection,
        next_keyboard: ProofCollectionKeyboardState {
            active_id: next_active,
        },
        deleted_assets,
        deleted_ids,
    })
}

fn proof_collection_duplicate_label_candidate(label: &str, index: usize) -> String {
    if index == 1 {
        format!("{label} Copy")
    } else {
        format!("{label} Copy {index}")
    }
}

fn proof_collection_duplicate_id_candidate(id: &str, index: usize) -> String {
    if index == 1 {
        format!("{id}-copy")
    } else {
        format!("{id}-copy-{index}")
    }
}

fn proof_collection_duplicate_path_candidate(path: &str, index: usize) -> String {
    let suffix = if index == 1 {
        "-copy".to_string()
    } else {
        format!("-copy-{index}")
    };

    match path.rsplit_once('.') {
        Some((stem, ext)) if !ext.contains('/') => format!("{stem}{suffix}.{ext}"),
        _ => format!("{path}{suffix}"),
    }
}

fn proof_collection_unique_copy_text(
    used: &mut HashSet<String>,
    candidate: impl Fn(usize) -> String,
) -> Arc<str> {
    let mut index = 1;
    loop {
        let value = candidate(index);
        if used.insert(value.clone()) {
            return Arc::from(value);
        }
        index += 1;
    }
}

fn proof_collection_duplicate_selection(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    reverse_order: bool,
) -> Option<ProofCollectionDuplicateResult> {
    let selected_visible_assets = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    if selected_visible_assets.is_empty() {
        return None;
    }

    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&visible_keys, selection, keyboard);
    let mut used_ids = stored_assets
        .iter()
        .map(|asset| asset.id.to_string())
        .collect::<HashSet<_>>();
    let mut used_labels = stored_assets
        .iter()
        .map(|asset| asset.label.to_string())
        .collect::<HashSet<_>>();
    let mut used_paths = stored_assets
        .iter()
        .map(|asset| asset.path.to_string())
        .collect::<HashSet<_>>();
    let mut duplicates_by_source = HashMap::<Arc<str>, ProofCollectionAsset>::new();

    for asset in &selected_visible_assets {
        let duplicate = ProofCollectionAsset {
            id: proof_collection_unique_copy_text(&mut used_ids, |index| {
                proof_collection_duplicate_id_candidate(asset.id.as_ref(), index)
            }),
            label: proof_collection_unique_copy_text(&mut used_labels, |index| {
                proof_collection_duplicate_label_candidate(asset.label.as_ref(), index)
            }),
            path: proof_collection_unique_copy_text(&mut used_paths, |index| {
                proof_collection_duplicate_path_candidate(asset.path.as_ref(), index)
            }),
            kind: asset.kind.clone(),
            size_kib: asset.size_kib,
        };
        duplicates_by_source.insert(asset.id.clone(), duplicate);
    }

    let mut remaining_and_duplicates =
        Vec::with_capacity(stored_assets.len() + duplicates_by_source.len());
    for asset in stored_assets {
        remaining_and_duplicates.push(asset.clone());
        if let Some(duplicate) = duplicates_by_source.get(&asset.id) {
            remaining_and_duplicates.push(duplicate.clone());
        }
    }

    let next_visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(remaining_and_duplicates.clone()),
        reverse_order,
    );
    let duplicated_ids_set = duplicates_by_source
        .values()
        .map(|asset| asset.id.as_ref())
        .collect::<HashSet<_>>();
    let duplicated_assets = next_visible_assets
        .iter()
        .filter(|asset| duplicated_ids_set.contains(asset.id.as_ref()))
        .cloned()
        .collect::<Vec<_>>();
    let duplicated_ids = duplicated_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let next_active = active_id
        .and_then(|id| duplicates_by_source.get(&id).map(|asset| asset.id.clone()))
        .or_else(|| duplicated_ids.first().cloned());
    let next_selection = duplicated_ids
        .first()
        .cloned()
        .map(|anchor| ImUiMultiSelectState::new(duplicated_ids.clone(), Some(anchor)))
        .unwrap_or_default();

    Some(ProofCollectionDuplicateResult {
        next_assets: remaining_and_duplicates,
        duplicated_assets,
        next_selection,
        next_keyboard: ProofCollectionKeyboardState {
            active_id: next_active,
        },
    })
}

fn proof_collection_set_command_status(
    app: &mut KernelApp,
    command_status_model: &Model<String>,
    next_status: String,
) {
    let _ = app.models_mut().update(command_status_model, |status| {
        status.clear();
        status.push_str(&next_status);
    });
}

fn proof_collection_box_select_hits(
    collection_keys: &[Arc<str>],
    rendered_items: &[ProofCollectionRenderedItem],
    drag_rect: Rect,
) -> Vec<Arc<str>> {
    let bounds_by_id = rendered_items
        .iter()
        .map(|item| (item.id.as_ref(), item.local_bounds))
        .collect::<HashMap<_, _>>();

    collection_keys
        .iter()
        .filter(|key| {
            bounds_by_id
                .get(key.as_ref())
                .is_some_and(|bounds| proof_collection_rects_intersect(*bounds, drag_rect))
        })
        .cloned()
        .collect()
}

fn proof_collection_box_select_state_for_hits(
    collection_keys: &[Arc<str>],
    baseline_selected: &[Arc<str>],
    hits: &[Arc<str>],
    append_mode: bool,
) -> ImUiMultiSelectState<Arc<str>> {
    let selected = if append_mode {
        let mut merged = baseline_selected.to_vec();
        for hit in hits {
            if !merged.iter().any(|item| item == hit) {
                merged.push(hit.clone());
            }
        }
        merged
    } else {
        hits.to_vec()
    };

    ImUiMultiSelectState::from_ordered_selection(collection_keys, selected, None)
}

fn proof_collection_box_select_selection(
    collection_keys: &[Arc<str>],
    rendered_items: &[ProofCollectionRenderedItem],
    session: &ProofCollectionBoxSelectSession,
) -> ImUiMultiSelectState<Arc<str>> {
    let drag_rect = proof_collection_drag_rect(session.origin_local, session.current_local);
    let hits = proof_collection_box_select_hits(collection_keys, rendered_items, drag_rect);
    proof_collection_box_select_state_for_hits(
        collection_keys,
        &session.baseline_selected,
        &hits,
        session.append_mode,
    )
}

fn proof_collection_box_select_active_rect(state: &ProofCollectionBoxSelectState) -> Option<Rect> {
    let session = state.session.as_ref()?;
    session
        .threshold_met
        .then(|| proof_collection_drag_rect(session.origin_local, session.current_local))
}

fn proof_collection_drag_payload_for_asset(
    assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    dragged: &ProofCollectionAsset,
) -> ProofCollectionDragPayload {
    let selected_assets = proof_collection_selected_assets(assets, selection);
    let payload_assets = if selection.is_selected(&dragged.id) && !selected_assets.is_empty() {
        selected_assets
    } else {
        vec![dragged]
    };
    let lead = payload_assets.first().copied().unwrap_or(dragged);
    let asset_ids = payload_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let asset_paths = payload_assets
        .iter()
        .map(|asset| asset.path.clone())
        .collect::<Vec<_>>();

    ProofCollectionDragPayload {
        lead_label: lead.label.clone(),
        lead_path: lead.path.clone(),
        asset_ids: asset_ids.into(),
        asset_paths: asset_paths.into(),
    }
}

fn proof_collection_drag_preview_title(payload: &ProofCollectionDragPayload) -> Arc<str> {
    if payload.asset_ids.len() == 1 {
        payload.lead_label.clone()
    } else {
        Arc::from(format!("{} selected assets", payload.asset_ids.len()))
    }
}

fn proof_collection_drag_preview_subtitle(
    payload: &ProofCollectionDragPayload,
) -> Option<Arc<str>> {
    if payload.asset_paths.len() == 1 {
        Some(payload.lead_path.clone())
    } else {
        Some(Arc::from(format!(
            "{} + {} more",
            payload.lead_path,
            payload.asset_paths.len() - 1
        )))
    }
}

fn proof_collection_drop_status(prefix: &str, payload: &ProofCollectionDragPayload) -> String {
    let paths = payload
        .asset_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{prefix} {} asset(s): {paths}", payload.asset_paths.len())
}

pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {
    vec![
        ProofCollectionAsset {
            id: Arc::from("stone-albedo"),
            label: Arc::from("Stone Albedo"),
            path: Arc::from("textures/stone/albedo.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 512,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-normal"),
            label: Arc::from("Stone Normal"),
            path: Arc::from("textures/stone/normal.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 384,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-orm"),
            label: Arc::from("Stone ORM"),
            path: Arc::from("textures/stone/orm.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 256,
        },
        ProofCollectionAsset {
            id: Arc::from("moss-overlay"),
            label: Arc::from("Moss Overlay"),
            path: Arc::from("textures/moss/overlay.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 196,
        },
        ProofCollectionAsset {
            id: Arc::from("pebble-height"),
            label: Arc::from("Pebble Height"),
            path: Arc::from("textures/pebble/height.ktx2"),
            kind: Arc::from("Height"),
            size_kib: 164,
        },
        ProofCollectionAsset {
            id: Arc::from("dust-mask"),
            label: Arc::from("Dust Mask"),
            path: Arc::from("textures/shared/dust-mask.ktx2"),
            kind: Arc::from("Mask"),
            size_kib: 72,
        },
    ]
    .into()
}

fn authoring_parity_collection_selection_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ImUiMultiSelectState<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_selection",
        |cx| {
            let assets = authoring_parity_collection_assets();
            let default_id = assets.first().map(|asset| asset.id.clone());
            let state = default_id
                .clone()
                .map(ImUiMultiSelectState::single)
                .unwrap_or_default();
            cx.app.models_mut().insert(state)
        },
    )
}

fn authoring_parity_collection_assets_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<ProofCollectionAsset>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_assets",
        |cx| {
            cx.app.models_mut().insert(
                authoring_parity_collection_assets()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        },
    )
}

fn authoring_parity_collection_reverse_order_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_reverse_order",
        |cx| cx.app.models_mut().insert(false),
    )
}

fn authoring_parity_collection_box_select_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ProofCollectionBoxSelectState> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
        |cx| {
            cx.app
                .models_mut()
                .insert(ProofCollectionBoxSelectState::default())
        },
    )
}

fn authoring_parity_collection_keyboard_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ProofCollectionKeyboardState> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_keyboard",
        |cx| {
            let active_id = authoring_parity_collection_assets()
                .first()
                .map(|asset| asset.id.clone());
            cx.app
                .models_mut()
                .insert(ProofCollectionKeyboardState { active_id })
        },
    )
}

fn authoring_parity_collection_zoom_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Px> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_zoom",
        |cx| {
            cx.app
                .models_mut()
                .insert(Px(PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX))
        },
    )
}

fn authoring_parity_collection_scroll_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> ScrollHandle {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
        |_cx| ScrollHandle::default(),
    )
}

fn authoring_parity_collection_context_menu_anchor_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Point>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_context_menu_anchor",
        |cx| cx.app.models_mut().insert(None::<Point>),
    )
}

fn authoring_parity_collection_rename_session_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<ProofCollectionRenameSession>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_session",
        |cx| {
            cx.app
                .models_mut()
                .insert(None::<ProofCollectionRenameSession>)
        },
    )
}

fn authoring_parity_collection_rename_draft_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_draft",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn authoring_parity_collection_rename_focus_pending_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_focus_pending",
        |cx| cx.app.models_mut().insert(false),
    )
}

fn authoring_parity_collection_active_focus_target_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<GlobalElementId>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_active_focus_target",
        |cx| cx.app.models_mut().insert(None::<GlobalElementId>),
    )
}

fn authoring_parity_collection_rename_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

fn authoring_parity_collection_command_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_command_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

fn authoring_parity_collection_drop_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_drop_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

fn proof_collection_readout_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
    test_id: &'static str,
) {
    let element =
        ui.with_cx_mut(|cx| proof_compact_readout_element(cx, text, Arc::<str>::from(test_id)));
    ui.add(element);
}

fn proof_collection_section_label(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: &'static str,
    test_id: &'static str,
) {
    let element = ui.with_cx_mut(|cx| proof_section_chrome_label(cx, text, test_id));
    ui.add(element);
}

pub(super) fn render_collection_first_asset_browser_proof(ui: &mut ImUi<'_, '_, KernelApp>) {
    proof_collection_section_label(
        ui,
        "Collection-first asset browser proof",
        "imui-editor-proof.authoring.imui.collection.title",
    );
    ui.text_wrapped(
        "Stable keys keep browser selection pinned while visible order flips and selected-set drag/drop stays app-defined.",
    );
    ui.text_wrapped(
        "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
    );

    let collection_selection_model = authoring_parity_collection_selection_model(ui.cx_mut());
    let collection_assets_model = authoring_parity_collection_assets_model(ui.cx_mut());
    let collection_reverse_order_model =
        authoring_parity_collection_reverse_order_model(ui.cx_mut());
    let collection_box_select_model = authoring_parity_collection_box_select_model(ui.cx_mut());
    let collection_keyboard_model = authoring_parity_collection_keyboard_model(ui.cx_mut());
    let collection_zoom_model = authoring_parity_collection_zoom_model(ui.cx_mut());
    let collection_context_menu_anchor_model =
        authoring_parity_collection_context_menu_anchor_model(ui.cx_mut());
    let collection_rename_session_model =
        authoring_parity_collection_rename_session_model(ui.cx_mut());
    let collection_rename_draft_model = authoring_parity_collection_rename_draft_model(ui.cx_mut());
    let collection_rename_focus_pending_model =
        authoring_parity_collection_rename_focus_pending_model(ui.cx_mut());
    let collection_active_focus_target_model =
        authoring_parity_collection_active_focus_target_model(ui.cx_mut());
    let collection_rename_status_model =
        authoring_parity_collection_rename_status_model(ui.cx_mut());
    let collection_command_status_model =
        authoring_parity_collection_command_status_model(ui.cx_mut());
    let collection_scroll_handle = authoring_parity_collection_scroll_handle(ui.cx_mut());
    let collection_drop_status_model = authoring_parity_collection_drop_status_model(ui.cx_mut());
    let stored_collection_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_assets_model, |state| state.clone());
    let collection_selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_selection_model, |state| state);
    let collection_box_select = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_box_select_model, |state| state);
    let collection_keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_keyboard_model, |state| state);
    let collection_tile_extent = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_zoom_model, |state| state);
    let mut collection_reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_reverse_order_model, |value| value);
    let collection_rename_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_status_model, |state| state.clone());
    let collection_command_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_command_status_model, |state| state.clone());
    let collection_rename_session = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_session_model, |state| state.clone());
    let collection_rename_focus_pending = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_focus_pending_model, |state| state);
    let collection_layout = proof_collection_layout_metrics(
        collection_scroll_handle.viewport_size().width,
        collection_tile_extent,
    );

    let order_toggle = ui.button_with_options(
        if collection_reverse_order {
            "Show folder order"
        } else {
            "Reverse visible order"
        },
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.order-toggle",
            )),
            ..Default::default()
        },
    );
    if order_toggle.clicked() {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_reverse_order_model, |value| *value = !*value);
        collection_reverse_order = !collection_reverse_order;
    }

    let collection_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_collection_assets.clone()),
        collection_reverse_order,
    );
    let collection_keys = collection_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let collection_active_id = proof_collection_active_id(
        &collection_keys,
        &collection_selection,
        &collection_keyboard,
    );
    let collection_rename_ready_session = proof_collection_begin_rename_session(
        &collection_assets,
        &collection_selection,
        &collection_keyboard,
    );

    proof_collection_readout_text(
        ui,
        proof_collection_assets_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.assets-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_visible_order_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.visible-order-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_selection_line(&collection_assets, &collection_selection),
        "imui-editor-proof.authoring.imui.collection.selection-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_active_line(
            &collection_assets,
            &collection_selection,
            &collection_keyboard,
        ),
        "imui-editor-proof.authoring.imui.collection.active-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_zoom_line(collection_layout),
        "imui-editor-proof.authoring.imui.collection.zoom-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_select_all_line(),
        "imui-editor-proof.authoring.imui.collection.select-all-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_line(),
        "imui-editor-proof.authoring.imui.collection.rename-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_context_menu_line(),
        "imui-editor-proof.authoring.imui.collection.context-menu-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_package_line(),
        "imui-editor-proof.authoring.imui.collection.command-package-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_status_line(&collection_rename_status),
        "imui-editor-proof.authoring.imui.collection.rename-status-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_status_line(&collection_command_status),
        "imui-editor-proof.authoring.imui.collection.command-status-readout",
    );
    let duplicate_selected = ui.button_with_options(
        "Duplicate selected assets",
        kit::ButtonOptions {
            enabled: !collection_selection.is_empty(),
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.duplicate-selected",
            )),
            ..Default::default()
        },
    );
    if duplicate_selected.clicked()
        && let Some(duplicate) = proof_collection_duplicate_selection(
            &collection_assets,
            &stored_collection_assets,
            &collection_selection,
            &collection_keyboard,
            collection_reverse_order,
        )
    {
        let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_assets_model, |state| {
                *state = duplicate.next_assets.clone();
            });
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_selection_model, |state| {
                *state = duplicate.next_selection.clone();
            });
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_keyboard_model, |state| {
                *state = duplicate.next_keyboard.clone();
            });
        proof_collection_set_command_status(
            ui.cx_mut().app,
            &collection_command_status_model,
            command_status,
        );
    }
    let rename_active = ui.button_with_options(
        "Rename active asset",
        kit::ButtonOptions {
            enabled: collection_rename_ready_session.is_some(),
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.rename-active",
            )),
            ..Default::default()
        },
    );
    if rename_active.clicked()
        && let Some(session) = collection_rename_ready_session.as_ref()
    {
        proof_collection_begin_inline_rename_in_app(
            ui.cx_mut().app,
            &collection_rename_session_model,
            &collection_rename_draft_model,
            &collection_rename_focus_pending_model,
            &collection_rename_status_model,
            session,
        );
    }
    let delete_selected = ui.button_with_options(
        "Delete selected assets",
        kit::ButtonOptions {
            enabled: !collection_selection.is_empty(),
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.delete-selected",
            )),
            ..Default::default()
        },
    );
    if delete_selected.clicked()
        && let Some(delete) = proof_collection_delete_selection(
            &collection_assets,
            &stored_collection_assets,
            &collection_selection,
            &collection_keyboard,
        )
    {
        let command_status = proof_collection_delete_status(&delete.deleted_assets);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_assets_model, |state| {
                *state = delete.remaining_assets.clone();
            });
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_selection_model, |state| {
                *state = delete.next_selection.clone();
            });
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_keyboard_model, |state| {
                *state = delete.next_keyboard.clone();
            });
        proof_collection_set_command_status(
            ui.cx_mut().app,
            &collection_command_status_model,
            command_status,
        );
    }

    ui.child_region_with_options(
        "imui-editor-proof.authoring.imui.collection.browser",
        kit::ChildRegionOptions {
            layout: fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .h_px(Px(220.0)),
            scroll: kit::ScrollOptions {
                handle: Some(collection_scroll_handle.clone()),
                viewport_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.browser.viewport",
                )),
                ..Default::default()
            },
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.browser",
            )),
            content_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.browser.content",
            )),
            ..Default::default()
        },
        |ui| {
            let collection_assets = collection_assets.clone();
            let collection_keys = collection_keys.clone();
            let collection_assets_model = collection_assets_model.clone();
            let collection_reverse_order_model = collection_reverse_order_model.clone();
            let collection_selection = collection_selection.clone();
            let collection_selection_model = collection_selection_model.clone();
            let collection_box_select_model = collection_box_select_model.clone();
            let collection_box_select = collection_box_select.clone();
            let collection_keyboard_model = collection_keyboard_model.clone();
            let collection_zoom_model = collection_zoom_model.clone();
            let collection_context_menu_anchor_model = collection_context_menu_anchor_model.clone();
            let collection_active_focus_target_model =
                collection_active_focus_target_model.clone();
            let collection_active_id = collection_active_id.clone();
            let collection_rename_session = collection_rename_session.clone();
            let collection_rename_session_model = collection_rename_session_model.clone();
            let collection_rename_draft_model = collection_rename_draft_model.clone();
            let collection_rename_focus_pending_model =
                collection_rename_focus_pending_model.clone();
            let collection_rename_focus_pending = collection_rename_focus_pending;
            let collection_rename_status_model = collection_rename_status_model.clone();
            let collection_command_status_model = collection_command_status_model.clone();
            let collection_scroll_handle = collection_scroll_handle.clone();
            let collection_layout = collection_layout;

            ui.add_ui(fret_ui_kit::ui::container_build(move |cx, out| {
                let rendered_items = Rc::new(RefCell::new(Vec::<ProofCollectionRenderedItem>::new()));
                let mut props = fret_ui::element::PointerRegionProps::default();
                props.layout.size.width = Length::Fill;
                props.capture_phase_pointer_moves = true;

                out.push(cx.pointer_region(props, move |cx| {
                    let scope_id = cx.root_id();
                    let scope_origin = cx
                        .last_visual_bounds_for_element(scope_id)
                        .or_else(|| cx.last_bounds_for_element(scope_id))
                        .map(|rect| rect.origin);

                    let rendered_items_for_move = rendered_items.clone();
                    let rendered_items_for_up = rendered_items.clone();
                    let assets_model_for_keys = collection_assets_model.clone();
                    let reverse_order_model_for_keys = collection_reverse_order_model.clone();
                    let selection_model_for_keys = collection_selection_model.clone();
                    let selection_model_for_down = collection_selection_model.clone();
                    let selection_model_for_move = collection_selection_model.clone();
                    let selection_model_for_up = collection_selection_model.clone();
                    let keyboard_model_for_keys = collection_keyboard_model.clone();
                    let keyboard_model_for_move = collection_keyboard_model.clone();
                    let keyboard_model_for_up = collection_keyboard_model.clone();
                    let keyboard_model_for_clear = collection_keyboard_model.clone();
                    let context_menu_anchor_model_for_up =
                        collection_context_menu_anchor_model.clone();
                    let box_select_model_for_down = collection_box_select_model.clone();
                    let box_select_model_for_move = collection_box_select_model.clone();
                    let box_select_model_for_up = collection_box_select_model.clone();
                    let box_select_model_for_cancel = collection_box_select_model.clone();
                    let collection_keys_for_move = collection_keys.clone();
                    let collection_keys_for_up = collection_keys.clone();
                    let collection_layout_columns = collection_layout.columns;
                    let collection_zoom_model_for_wheel = collection_zoom_model.clone();
                    let rename_session_model_for_keys = collection_rename_session_model.clone();
                    let rename_draft_model_for_keys = collection_rename_draft_model.clone();
                    let rename_focus_pending_model_for_keys =
                        collection_rename_focus_pending_model.clone();
                    let rename_status_model_for_keys = collection_rename_status_model.clone();
                    let command_status_model_for_keys = collection_command_status_model.clone();
                    let collection_scroll_handle_for_wheel = collection_scroll_handle.clone();
                    let collection_asset_count_for_wheel = collection_assets.len();

                    cx.key_on_key_down_for(scope_id, Arc::new(move |host, acx, down| {
                        if down.ime_composing {
                            return false;
                        }

                        let selection = host
                            .models_mut()
                            .read(&selection_model_for_keys, |state| state.clone())
                            .unwrap_or_default();
                        let keyboard = host
                            .models_mut()
                            .read(&keyboard_model_for_keys, |state| state.clone())
                            .unwrap_or_default();
                        let stored_assets = host
                            .models_mut()
                            .read(&assets_model_for_keys, |state| state.clone())
                            .unwrap_or_default();
                        let reverse_order = host
                            .models_mut()
                            .read(&reverse_order_model_for_keys, |value| *value)
                            .unwrap_or(false);
                        let visible_assets = proof_collection_assets_in_visible_order(
                            Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
                            reverse_order,
                        );
                        if host
                            .models_mut()
                            .read(&rename_session_model_for_keys, |state| state.is_some())
                            .unwrap_or(false)
                        {
                            return false;
                        }
                        let collection_keys_for_keys = visible_assets
                            .iter()
                            .map(|asset| asset.id.clone())
                            .collect::<Vec<_>>();
                        if down.modifiers == Modifiers::default()
                            && proof_collection_delete_key_matches(down.key)
                            && let Some(delete) = proof_collection_delete_selection(
                                &visible_assets,
                                &stored_assets,
                                &selection,
                                &keyboard,
                            )
                        {
                            let next_status = proof_collection_delete_status(&delete.deleted_assets);
                            let _ = host.update_model(&assets_model_for_keys, |state| {
                                *state = delete.remaining_assets.clone();
                            });
                            let _ = host.update_model(&selection_model_for_keys, |state| {
                                *state = delete.next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_keys, |state| {
                                *state = delete.next_keyboard.clone();
                            });
                            let _ = host.update_model(&command_status_model_for_keys, |status| {
                                status.clear();
                                status.push_str(&next_status);
                            });
                            host.notify(acx);
                            return true;
                        }

                        if proof_collection_rename_shortcut_matches(down.key, down.modifiers)
                            && let Some(session) = proof_collection_begin_rename_session(
                                &visible_assets,
                                &selection,
                                &keyboard,
                            )
                        {
                            let _ = host.update_model(&rename_session_model_for_keys, |state| {
                                *state = Some(session.clone());
                            });
                            let _ = host.update_model(&rename_draft_model_for_keys, |draft| {
                                draft.clear();
                                draft.push_str(session.original_label.as_ref());
                            });
                            let _ = host.update_model(&rename_focus_pending_model_for_keys, |state| {
                                *state = true;
                            });
                            let _ = host.update_model(&rename_status_model_for_keys, |status| {
                                status.clear();
                                status.push_str(&proof_collection_rename_ready_status(
                                    session.original_label.as_ref(),
                                ));
                            });
                            host.notify(acx);
                            return true;
                        }

                        if proof_collection_select_all_shortcut_matches(
                            down.key,
                            down.modifiers,
                        ) && let Some((next_selection, next_keyboard)) =
                            proof_collection_select_all_selection(
                                &collection_keys_for_keys,
                                &selection,
                                &keyboard,
                            )
                        {
                            let next_status =
                                proof_collection_select_all_status(next_selection.selected_count());
                            let _ = host.update_model(&selection_model_for_keys, |state| {
                                *state = next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_keys, |state| {
                                *state = next_keyboard.clone();
                            });
                            let _ = host.update_model(&command_status_model_for_keys, |status| {
                                status.clear();
                                status.push_str(&next_status);
                            });
                            host.notify(acx);
                            return true;
                        }

                        if proof_collection_duplicate_shortcut_matches(
                            down.key,
                            down.modifiers,
                        ) && let Some(duplicate) = proof_collection_duplicate_selection(
                            &visible_assets,
                            &stored_assets,
                            &selection,
                            &keyboard,
                            reverse_order,
                        ) {
                            let next_status =
                                proof_collection_duplicate_status(&duplicate.duplicated_assets);
                            let _ = host.update_model(&assets_model_for_keys, |state| {
                                *state = duplicate.next_assets.clone();
                            });
                            let _ = host.update_model(&selection_model_for_keys, |state| {
                                *state = duplicate.next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_keys, |state| {
                                *state = duplicate.next_keyboard.clone();
                            });
                            let _ = host.update_model(&command_status_model_for_keys, |status| {
                                status.clear();
                                status.push_str(&next_status);
                            });
                            host.notify(acx);
                            return true;
                        }

                        let Some((next_selection, next_keyboard)) = proof_collection_keyboard_selection(
                            &collection_keys_for_keys,
                            &selection,
                            &keyboard,
                            collection_layout_columns,
                            down.key,
                            down.modifiers,
                        ) else {
                            return false;
                        };

                        let _ = host.update_model(&selection_model_for_keys, |state| {
                            *state = next_selection.clone();
                        });
                        let _ = host.update_model(&keyboard_model_for_keys, |state| {
                            *state = next_keyboard.clone();
                        });
                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_wheel(Arc::new(move |host, acx, wheel| {
                        let Some(update) = proof_collection_zoom_request(
                            collection_layout,
                            collection_scroll_handle_for_wheel.offset(),
                            wheel.position_local,
                            wheel.delta,
                            wheel.modifiers,
                            collection_asset_count_for_wheel,
                        ) else {
                            return false;
                        };

                        let _ = host.update_model(&collection_zoom_model_for_wheel, |state| {
                            *state = update.next_tile_extent;
                        });
                        collection_scroll_handle_for_wheel.set_offset(update.next_scroll_offset);
                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                        if down.button != fret_core::MouseButton::Left {
                            return false;
                        }

                        host.request_focus(acx.target);
                        if down.hit_is_pressable {
                            return false;
                        }
                        let baseline_selected = host
                            .models_mut()
                            .read(&selection_model_for_down, |state| state.selected().to_vec())
                            .unwrap_or_default();
                        let append_mode = down.modifiers.ctrl || down.modifiers.meta;
                        let _ = host.update_model(&box_select_model_for_down, |state| {
                            state.session = Some(ProofCollectionBoxSelectSession {
                                pointer_id: down.pointer_id,
                                origin_local: down.position_local,
                                current_local: down.position_local,
                                baseline_selected,
                                append_mode,
                                threshold_met: false,
                            });
                        });
                        host.capture_pointer();
                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                        if !mv.buttons.left {
                            return false;
                        }

                        let session = host
                            .update_model(&box_select_model_for_move, |state| {
                                let Some(session) = state.session.as_mut() else {
                                    return None;
                                };
                                if session.pointer_id != mv.pointer_id {
                                    return None;
                                }

                                session.current_local = mv.position_local;
                                if !session.threshold_met {
                                    session.threshold_met = proof_collection_drag_threshold_met(
                                        session.origin_local,
                                        session.current_local,
                                    );
                                }

                                Some(session.clone())
                            })
                            .flatten();

                        let Some(session) = session else {
                            return false;
                        };

                        if session.threshold_met {
                            let next_selection = proof_collection_box_select_selection(
                                &collection_keys_for_move,
                                &rendered_items_for_move.borrow(),
                                &session,
                            );
                            let _ = host.update_model(&selection_model_for_move, |state| {
                                *state = next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_move, |state| {
                                state.active_id = next_selection.first_selected().cloned();
                            });
                        }

                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                        if up.button == fret_core::MouseButton::Right && up.is_click {
                            if up.down_hit_pressable_target.is_some()
                                || up.down_hit_pressable_target_in_descendant_subtree
                            {
                                return false;
                            }

                            host.request_focus(acx.target);
                            let position = up.position_window.unwrap_or(up.position);
                            let _ = host.update_model(&context_menu_anchor_model_for_up, |state| {
                                *state = Some(position);
                            });
                            host.notify(acx);
                            return true;
                        }

                        let session = host
                            .update_model(&box_select_model_for_up, |state| {
                                let Some(mut session) = state.session.take() else {
                                    return None;
                                };
                                if session.pointer_id != up.pointer_id {
                                    state.session = Some(session);
                                    return None;
                                }

                                session.current_local = up.position_local;
                                if !session.threshold_met {
                                    session.threshold_met = proof_collection_drag_threshold_met(
                                        session.origin_local,
                                        session.current_local,
                                    );
                                }

                                Some(session)
                            })
                            .flatten();

                        let Some(session) = session else {
                            return false;
                        };

                        host.release_pointer_capture();
                        if session.threshold_met {
                            let next_selection = proof_collection_box_select_selection(
                                &collection_keys_for_up,
                                &rendered_items_for_up.borrow(),
                                &session,
                            );
                            let _ = host.update_model(&selection_model_for_up, |state| {
                                *state = next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_up, |state| {
                                state.active_id = next_selection.first_selected().cloned();
                            });
                        } else if !session.append_mode {
                            let _ = host.update_model(&selection_model_for_up, |state| {
                                state.clear();
                            });
                            let _ = host.update_model(&keyboard_model_for_clear, |state| {
                                state.active_id = None;
                            });
                        }

                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_cancel(Arc::new(move |host, _acx, cancel| {
                        let cleared = host
                            .update_model(&box_select_model_for_cancel, |state| {
                                let matches_pointer = state
                                    .session
                                    .as_ref()
                                    .is_some_and(|session| session.pointer_id == cancel.pointer_id);
                                if matches_pointer {
                                    state.session = None;
                                }
                                matches_pointer
                            })
                            .unwrap_or(false);
                        if cleared {
                            host.release_pointer_capture();
                        }
                        cleared
                    }));

                    vec![fret_ui_kit::ui::stack(move |cx| {
                        let rendered_items_for_grid = rendered_items.clone();
                        let grid = fret_ui_kit::ui::container_build(
                            move |cx: &mut ElementContext<'_, KernelApp>, out| {
                                imui_build(cx, out, |ui| {
                                    ui.grid_with_options(
                                        kit::GridOptions {
                                            columns: collection_layout.columns,
                                            column_gap: fret_ui_kit::MetricRef::space(
                                                fret_ui_kit::Space::N2,
                                            ),
                                            row_gap: fret_ui_kit::MetricRef::space(
                                                fret_ui_kit::Space::N2,
                                            ),
                                            row_items: fret_ui_kit::Items::Stretch,
                                            test_id: Some(Arc::from(
                                                "imui-editor-proof.authoring.imui.collection.grid",
                                            )),
                                            ..Default::default()
                                        },
                                        |ui| {
                                            for asset in &collection_assets {
                                                let payload = proof_collection_drag_payload_for_asset(
                                                    &collection_assets,
                                                    &collection_selection,
                                                    asset,
                                                );
                                                let preview_title =
                                                    proof_collection_drag_preview_title(&payload);
                                                let preview_subtitle =
                                                    proof_collection_drag_preview_subtitle(&payload);
                                                let ghost_id = format!(
                                                    "imui-editor-proof.authoring.imui.collection.asset.{}.ghost",
                                                    asset.id
                                                );

                                                ui.id(asset.id.clone(), |ui| {
                                                    ui.vertical_with_options(
                                                        kit::VerticalOptions {
                                                            layout: fret_ui_kit::LayoutRefinement::default()
                                                                .flex_1()
                                                                .min_h(collection_layout.tile_min_height),
                                                            gap: fret_ui_kit::MetricRef::space(
                                                                fret_ui_kit::Space::N1,
                                                            ),
                                                            test_id: Some(Arc::from(format!(
                                                                "imui-editor-proof.authoring.imui.collection.asset.{}",
                                                                asset.id
                                                            ))),
                                                            ..Default::default()
                                                        },
                                                        |ui| {
                                                            let trigger = ui
                                                                .multi_selectable_with_options(
                                                                    asset.label.clone(),
                                                                    &collection_selection_model,
                                                                    &collection_keys,
                                                                    asset.id.clone(),
                                                                    kit::SelectableOptions {
                                                                        focusable: false,
                                                                        test_id: Some(Arc::from(format!(
                                                                            "imui-editor-proof.authoring.imui.collection.asset.{}.select",
                                                                            asset.id
                                                                        ))),
                                                                        ..Default::default()
                                                                    },
                                                                );
                                                            if collection_active_id
                                                                .as_ref()
                                                                .is_some_and(|active_id| active_id == &asset.id)
                                                                && let Some(focus_target) = trigger.id()
                                                            {
                                                                let _ = ui
                                                                    .cx_mut()
                                                                    .app
                                                                    .models_mut()
                                                                    .update(
                                                                        &collection_active_focus_target_model,
                                                                        |state| {
                                                                            *state = Some(focus_target);
                                                                        },
                                                                    );
                                                            }
                                                            if trigger.clicked() {
                                                                let _ = ui
                                                                    .cx_mut()
                                                                    .app
                                                                    .models_mut()
                                                                    .update(
                                                                        &collection_keyboard_model,
                                                                        |state| {
                                                                            state.active_id =
                                                                                Some(asset.id.clone());
                                                                        },
                                                                    );
                                                            }
                                                            if trigger.context_menu_requested() {
                                                                let (next_selection, next_keyboard) =
                                                                    proof_collection_context_menu_selection(
                                                                        &collection_selection,
                                                                        asset.id.clone(),
                                                                    );
                                                                let anchor = trigger
                                                                    .context_menu_anchor()
                                                                    .or(trigger.rect().map(|rect| rect.origin));
                                                                let _ = ui
                                                                    .cx_mut()
                                                                    .app
                                                                    .models_mut()
                                                                    .update(
                                                                        &collection_selection_model,
                                                                        |state| {
                                                                            *state = next_selection.clone();
                                                                        },
                                                                    );
                                                                let _ = ui
                                                                    .cx_mut()
                                                                    .app
                                                                    .models_mut()
                                                                    .update(
                                                                        &collection_keyboard_model,
                                                                        |state| {
                                                                            *state = next_keyboard.clone();
                                                                        },
                                                                    );
                                                                let _ = ui
                                                                    .cx_mut()
                                                                    .app
                                                                    .models_mut()
                                                                    .update(
                                                                        &collection_context_menu_anchor_model,
                                                                        |state| {
                                                                            *state = anchor;
                                                                        },
                                                                    );
                                                            }
                                                            if collection_rename_session
                                                                .as_ref()
                                                                .is_some_and(|session| session.target_id == asset.id)
                                                            {
                                                                let rename_input_id =
                                                                    Rc::new(Cell::new(None::<GlobalElementId>));
                                                                let rename_session_model_for_outcome =
                                                                    collection_rename_session_model.clone();
                                                                let rename_draft_model_for_outcome =
                                                                    collection_rename_draft_model.clone();
                                                                let rename_assets_model_for_outcome =
                                                                    collection_assets_model.clone();
                                                                let rename_status_model_for_outcome =
                                                                    collection_rename_status_model.clone();
                                                                let rename_focus_pending_model_for_outcome =
                                                                    collection_rename_focus_pending_model.clone();
                                                                let rename_restore_focus_target_model =
                                                                    collection_active_focus_target_model.clone();
                                                                let inline_test_id: Arc<str> = Arc::from(format!(
                                                                    "imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline",
                                                                    asset.id
                                                                ));
                                                                let inline_id_source: Arc<str> =
                                                                    Arc::from(format!(
                                                                        "imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline",
                                                                        asset.id
                                                                    ));
                                                                let field = TextField::new(
                                                                    collection_rename_draft_model.clone(),
                                                                )
                                                                .on_outcome(Some(Arc::new(
                                                                    move |host, action_cx, outcome: TextFieldOutcome| {
                                                                        let session = host
                                                                            .models_mut()
                                                                            .read(
                                                                                &rename_session_model_for_outcome,
                                                                                |state| state.clone(),
                                                                            )
                                                                            .ok()
                                                                            .flatten();
                                                                        let Some(session) = session else {
                                                                            return;
                                                                        };

                                                                        match outcome {
                                                                            EditSessionOutcome::Committed => {
                                                                                let draft = host
                                                                                    .models_mut()
                                                                                    .read(
                                                                                        &rename_draft_model_for_outcome,
                                                                                        |state| state.clone(),
                                                                                    )
                                                                                    .unwrap_or_default();
                                                                                let stored_assets = host
                                                                                    .models_mut()
                                                                                    .read(
                                                                                        &rename_assets_model_for_outcome,
                                                                                        |state| state.clone(),
                                                                                    )
                                                                                    .unwrap_or_default();
                                                                                if let Some(commit) =
                                                                                    proof_collection_commit_rename(
                                                                                        &stored_assets,
                                                                                        &session,
                                                                                        &draft,
                                                                                    )
                                                                                {
                                                                                    let _ = host.update_model(
                                                                                        &rename_assets_model_for_outcome,
                                                                                        |state| {
                                                                                            *state = commit.renamed_assets.clone();
                                                                                        },
                                                                                    );
                                                                                    let _ = host.update_model(
                                                                                        &rename_status_model_for_outcome,
                                                                                        |status| {
                                                                                            status.clear();
                                                                                            status.push_str(
                                                                                                &proof_collection_rename_commit_status(
                                                                                                    commit.previous_label.as_ref(),
                                                                                                    commit.next_label.as_ref(),
                                                                                                ),
                                                                                            );
                                                                                        },
                                                                                    );
                                                                                    let _ = host.update_model(
                                                                                        &rename_session_model_for_outcome,
                                                                                        |state| *state = None,
                                                                                    );
                                                                                    let _ = host.update_model(
                                                                                        &rename_focus_pending_model_for_outcome,
                                                                                        |state| *state = false,
                                                                                    );
                                                                                    proof_collection_restore_focus_after_inline_rename(
                                                                                        host,
                                                                                        action_cx,
                                                                                        &rename_restore_focus_target_model,
                                                                                    );
                                                                                } else {
                                                                                    let _ = host.update_model(
                                                                                        &rename_status_model_for_outcome,
                                                                                        |status| {
                                                                                            status.clear();
                                                                                            status.push_str(
                                                                                                &proof_collection_rename_invalid_status(
                                                                                                    session.original_label.as_ref(),
                                                                                                ),
                                                                                            );
                                                                                        },
                                                                                    );
                                                                                    let _ = host.update_model(
                                                                                        &rename_focus_pending_model_for_outcome,
                                                                                        |state| *state = true,
                                                                                    );
                                                                                    host.request_redraw(action_cx.window);
                                                                                }
                                                                            }
                                                                            EditSessionOutcome::Canceled => {
                                                                                let _ = host.update_model(
                                                                                    &rename_status_model_for_outcome,
                                                                                    |status| {
                                                                                        status.clear();
                                                                                        status.push_str(
                                                                                            &proof_collection_rename_cancel_status(
                                                                                                session.original_label.as_ref(),
                                                                                            ),
                                                                                        );
                                                                                    },
                                                                                );
                                                                                let _ = host.update_model(
                                                                                    &rename_session_model_for_outcome,
                                                                                    |state| *state = None,
                                                                                );
                                                                                let _ = host.update_model(
                                                                                    &rename_focus_pending_model_for_outcome,
                                                                                    |state| *state = false,
                                                                                );
                                                                                proof_collection_restore_focus_after_inline_rename(
                                                                                    host,
                                                                                    action_cx,
                                                                                    &rename_restore_focus_target_model,
                                                                                );
                                                                            }
                                                                        }
                                                                    },
                                                                )))
                                                                .options(TextFieldOptions {
                                                                    id_source: Some(inline_id_source),
                                                                    placeholder: Some(Arc::from(
                                                                        "Rename active asset",
                                                                    )),
                                                                    selection_behavior:
                                                                        EditorTextSelectionBehavior::SelectAllOnFocus,
                                                                    blur_behavior:
                                                                        TextFieldBlurBehavior::Cancel,
                                                                    test_id: Some(inline_test_id),
                                                                    input_id_out: Some(
                                                                        rename_input_id.clone(),
                                                                    ),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(ui.cx_mut());
                                                                ui.add(field);
                                                                if let Some(input_id) =
                                                                    rename_input_id.get()
                                                                {
                                                                    let focus_state =
                                                                        proof_collection_inline_rename_focus_state(
                                                                            ui.cx_mut(),
                                                                        );
                                                                    proof_collection_sync_inline_rename_focus(
                                                                        ui.cx_mut(),
                                                                        input_id,
                                                                        collection_rename_focus_pending,
                                                                        &collection_rename_focus_pending_model,
                                                                        &focus_state,
                                                                    );
                                                                }
                                                                ui.text_wrapped(
                                                                    "Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.",
                                                                );
                                                            }
                                                            let source = ui
                                                                .drag_source_with_options(
                                                                    trigger,
                                                                    payload.clone(),
                                                                    kit::DragSourceOptions::default(),
                                                                );
                                                            let _ = drag_preview_ghost_with_options(
                                                                ui,
                                                                ghost_id.as_str(),
                                                                source,
                                                                DragPreviewGhostOptions {
                                                                    test_id: Some(Arc::from(format!(
                                                                        "imui-editor-proof.authoring.imui.collection.asset.{}.ghost",
                                                                        asset.id
                                                                    ))),
                                                                    ..Default::default()
                                                                },
                                                                proof_drag_preview_card(
                                                                    preview_title.clone(),
                                                                    preview_subtitle.clone(),
                                                                ),
                                                            );

                                                            if let Some(scope_origin) = scope_origin
                                                                && let Some(bounds) = trigger
                                                                    .id()
                                                                    .and_then(|element_id| {
                                                                        ui.cx_mut()
                                                                            .last_visual_bounds_for_element(element_id)
                                                                    })
                                                                    .or(trigger.rect())
                                                            {
                                                                rendered_items_for_grid.borrow_mut().push(
                                                                    ProofCollectionRenderedItem {
                                                                        id: asset.id.clone(),
                                                                        local_bounds:
                                                                            proof_collection_localize_rect(
                                                                                bounds,
                                                                                scope_origin,
                                                                            ),
                                                                    },
                                                                );
                                                            }

                                                            proof_collection_readout_text(
                                                                ui,
                                                                format!(
                                                                    "{} | {} KiB",
                                                                    asset.kind, asset.size_kib
                                                                ),
                                                                "imui-editor-proof.authoring.imui.collection.asset.metadata",
                                                            );
                                                            proof_collection_readout_text(
                                                                ui,
                                                                asset.path.clone(),
                                                                "imui-editor-proof.authoring.imui.collection.asset.path",
                                                            );
                                                        },
                                                    );
                                                });
                                            }
                                        },
                                    );
                                });
                            },
                        )
                        .w_full()
                        .into_element(cx);

                        let mut layers = vec![grid];
                        if let Some(drag_rect) =
                            proof_collection_box_select_active_rect(&collection_box_select)
                        {
                            let theme = fret_ui::Theme::global(&*cx.app);
                            let ring = theme.color_token("ring");
                            let fill = Color { a: 0.14, ..ring };
                            let border = Color { a: 0.88, ..ring };
                            layers.push(
                                fret_ui_kit::ui::container(
                                    |_cx| Vec::<fret_ui::element::AnyElement>::new(),
                                )
                                .absolute()
                                .left_px(drag_rect.origin.x)
                                .top_px(drag_rect.origin.y)
                                .w_px(drag_rect.size.width)
                                .h_px(drag_rect.size.height)
                                .bg(fret_ui_kit::ColorRef::Color(fill))
                                .border_1()
                                .border_color(fret_ui_kit::ColorRef::Color(border))
                                .test_id(
                                    "imui-editor-proof.authoring.imui.collection.box-select.marquee",
                                )
                                .into_element(cx),
                            );
                        }
                        layers
                    })
                    .relative()
                    .w_full()
                    .h_full()
                    .test_id("imui-editor-proof.authoring.imui.collection.box-select.scope")
                    .into_element(cx)]
                }));
            }));
        },
    );

    let collection_context_menu_popup_id =
        "imui-editor-proof.authoring.imui.collection.context-menu";
    let collection_context_menu_anchor = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_context_menu_anchor_model, |state| state);
    if let Some(anchor) = collection_context_menu_anchor {
        ui.open_popup_at(
            collection_context_menu_popup_id,
            Rect::new(anchor, Size::new(Px(1.0), Px(1.0))),
        );
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_context_menu_anchor_model, |state| *state = None);
    }

    let collection_context_menu_open = ui.popup_open_model(collection_context_menu_popup_id);
    let popup_collection_selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_selection_model, |state| state);
    let popup_collection_keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_keyboard_model, |state| state);
    let popup_collection_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_assets_model, |state| state.clone());
    let popup_collection_reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_reverse_order_model, |state| state);
    let popup_visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(popup_collection_assets.clone()),
        popup_collection_reverse_order,
    );
    ui.begin_popup_menu(collection_context_menu_popup_id, None, |ui| {
        let rename_session = proof_collection_begin_rename_session(
            &popup_visible_assets,
            &popup_collection_selection,
            &popup_collection_keyboard,
        );
        proof_collection_readout_text(
            ui,
            format!(
                "Selection: {} item(s)",
                popup_collection_selection.selected_count()
            ),
            "imui-editor-proof.authoring.imui.collection.context-menu.selection-readout",
        );
        ui.separator();

        let duplicate_from_menu = ui.menu_item_with_options(
            "Duplicate selected assets",
            kit::MenuItemOptions {
                enabled: !popup_collection_selection.is_empty(),
                close_popup: Some(collection_context_menu_open.clone()),
                shortcut: Some(Arc::from("Primary+D")),
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected",
                )),
                ..Default::default()
            },
        );
        if duplicate_from_menu.clicked()
            && let Some(duplicate) = proof_collection_duplicate_selection(
                &popup_visible_assets,
                &popup_collection_assets,
                &popup_collection_selection,
                &popup_collection_keyboard,
                popup_collection_reverse_order,
            )
        {
            let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_assets_model, |state| {
                    *state = duplicate.next_assets.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_selection_model, |state| {
                    *state = duplicate.next_selection.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_keyboard_model, |state| {
                    *state = duplicate.next_keyboard.clone();
                });
            proof_collection_set_command_status(
                ui.cx_mut().app,
                &collection_command_status_model,
                command_status,
            );
        }

        let rename_from_menu = ui.menu_item_with_options(
            "Rename active asset",
            kit::MenuItemOptions {
                enabled: rename_session.is_some(),
                close_popup: Some(collection_context_menu_open.clone()),
                shortcut: Some(Arc::from("F2")),
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.context-menu.rename",
                )),
                ..Default::default()
            },
        );
        if rename_from_menu.clicked()
            && let Some(session) = rename_session
        {
            proof_collection_begin_inline_rename_in_app(
                ui.cx_mut().app,
                &collection_rename_session_model,
                &collection_rename_draft_model,
                &collection_rename_focus_pending_model,
                &collection_rename_status_model,
                &session,
            );
        }

        let delete_from_menu = ui.menu_item_with_options(
            "Delete selected assets",
            kit::MenuItemOptions {
                enabled: !popup_collection_selection.is_empty(),
                close_popup: Some(collection_context_menu_open.clone()),
                shortcut: Some(Arc::from("Del")),
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.context-menu.delete-selected",
                )),
                ..Default::default()
            },
        );
        if delete_from_menu.clicked()
            && let Some(delete) = proof_collection_delete_selection(
                &popup_visible_assets,
                &popup_collection_assets,
                &popup_collection_selection,
                &popup_collection_keyboard,
            )
        {
            let command_status = proof_collection_delete_status(&delete.deleted_assets);
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_assets_model, |state| {
                    *state = delete.remaining_assets.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_selection_model, |state| {
                    *state = delete.next_selection.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&collection_keyboard_model, |state| {
                    *state = delete.next_keyboard.clone();
                });
            proof_collection_set_command_status(
                ui.cx_mut().app,
                &collection_command_status_model,
                command_status,
            );
        }

        let _ = ui.menu_item_with_options(
            "Dismiss quick actions",
            kit::MenuItemOptions {
                close_popup: Some(collection_context_menu_open.clone()),
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.context-menu.dismiss",
                )),
                ..Default::default()
            },
        );
    });

    if let Some(session) = collection_rename_session.as_ref()
        && !collection_assets
            .iter()
            .any(|asset| asset.id == session.target_id)
    {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_session_model, |state| *state = None);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_focus_pending_model, |state| {
                *state = false
            });
    }

    let import_trigger = ui.button_with_options(
        "Import selected set to bundle",
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.import-target",
            )),
            ..Default::default()
        },
    );
    let import_drop = ui.drop_target::<ProofCollectionDragPayload>(import_trigger);
    if let Some(payload) = import_drop.delivered_payload() {
        let next_status = proof_collection_drop_status("Delivered", &payload);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_drop_status_model, |status| {
                status.clear();
                status.push_str(&next_status);
            });
    }

    let persisted_collection_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_drop_status_model, |value| value);
    let visible_collection_status = if let Some(payload) = import_drop.delivered_payload() {
        proof_collection_drop_status("Delivered", &payload)
    } else if let Some(payload) = import_drop.preview_payload() {
        proof_collection_drop_status("Preview", &payload)
    } else if import_drop.active() {
        "Compatible collection drag active".to_string()
    } else {
        persisted_collection_status
    };
    proof_collection_readout_text(
        ui,
        visible_collection_status,
        "imui-editor-proof.authoring.imui.collection.drop-status-readout",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
        selection.selected().iter().map(|id| id.as_ref()).collect()
    }

    fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
        selection.anchor().map(|id| id.as_ref())
    }

    #[test]
    fn proof_collection_box_select_replace_uses_visible_collection_order() {
        let assets = authoring_parity_collection_assets();
        let collection_keys = assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let rendered_items = vec![
            ProofCollectionRenderedItem {
                id: Arc::from("stone-orm"),
                local_bounds: Rect::new(
                    Point::new(Px(112.0), Px(0.0)),
                    Size::new(Px(96.0), Px(72.0)),
                ),
            },
            ProofCollectionRenderedItem {
                id: Arc::from("stone-albedo"),
                local_bounds: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(96.0), Px(72.0)),
                ),
            },
            ProofCollectionRenderedItem {
                id: Arc::from("stone-normal"),
                local_bounds: Rect::new(
                    Point::new(Px(0.0), Px(84.0)),
                    Size::new(Px(96.0), Px(72.0)),
                ),
            },
        ];
        let session = ProofCollectionBoxSelectSession {
            pointer_id: PointerId(0),
            origin_local: Point::new(Px(4.0), Px(4.0)),
            current_local: Point::new(Px(124.0), Px(152.0)),
            baseline_selected: vec![Arc::from("dust-mask")],
            append_mode: false,
            threshold_met: true,
        };

        let selection =
            proof_collection_box_select_selection(&collection_keys, &rendered_items, &session);

        assert_eq!(
            selected_ids(&selection),
            vec!["stone-albedo", "stone-normal", "stone-orm",]
        );
        assert_eq!(anchor_id(&selection), Some("stone-albedo"));
    }

    #[test]
    fn proof_collection_box_select_append_preserves_baseline_and_adds_hits() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let hits = vec![Arc::from("stone-albedo"), Arc::from("stone-orm")];

        let selection = proof_collection_box_select_state_for_hits(
            &collection_keys,
            &[Arc::from("dust-mask")],
            &hits,
            true,
        );

        assert_eq!(
            selected_ids(&selection),
            vec!["stone-albedo", "stone-orm", "dust-mask",]
        );
        assert_eq!(anchor_id(&selection), Some("stone-albedo"));
    }

    #[test]
    fn proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowRight,
            Modifiers::default(),
        )
        .expect("plain arrow navigation should be handled");

        assert_eq!(selected_ids(&next_selection), vec!["stone-normal"]);
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-normal")));
    }

    #[test]
    fn proof_collection_keyboard_shift_navigation_extends_range_from_anchor() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowDown,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        )
        .expect("shift+arrow navigation should be handled");

        assert_eq!(
            selected_ids(&next_selection),
            vec!["stone-normal", "stone-orm", "moss-overlay", "pebble-height",]
        );
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("pebble-height")));
    }

    #[test]
    fn proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-orm")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::Escape,
            Modifiers::default(),
        )
        .expect("escape should be handled by the collection scope");

        assert!(next_selection.is_empty());
        assert_eq!(next_selection.anchor(), None);
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-orm")));
    }

    #[test]
    fn proof_collection_keyboard_ignores_primary_modifier_shortcuts() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        assert!(
            proof_collection_keyboard_selection(
                &collection_keys,
                &selection,
                &keyboard,
                PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
                KeyCode::ArrowRight,
                Modifiers {
                    meta: true,
                    ..Default::default()
                },
            )
            .is_none(),
            "collection keyboard owner should stay app-local and avoid claiming primary-modifier shortcuts"
        );
    }

    #[test]
    fn proof_collection_begin_rename_session_prefers_active_visible_asset() {
        let visible_assets = authoring_parity_collection_assets();
        let selection = selection_state(&["stone-albedo", "stone-normal"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
            .expect("rename should target the active visible asset");

        assert_eq!(session.target_id, Arc::from("stone-normal"));
        assert_eq!(session.original_label, Arc::from("Stone Normal"));
    }

    #[test]
    fn proof_collection_begin_rename_session_falls_back_to_first_visible_asset() {
        let visible_assets = authoring_parity_collection_assets();
        let selection = ImUiMultiSelectState::default();
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("missing")),
        };

        let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
            .expect("rename should fall back to the first visible asset");

        assert_eq!(session.target_id, Arc::from("stone-albedo"));
        assert_eq!(session.original_label, Arc::from("Stone Albedo"));
    }

    #[test]
    fn proof_collection_rename_shortcut_matches_plain_f2_only() {
        assert!(proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers::default(),
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::KeyA,
            Modifiers::default(),
        ));
    }

    #[test]
    fn proof_collection_commit_rename_updates_label_without_touching_order_or_ids() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone-normal"),
            original_label: Arc::from("Stone Normal"),
        };

        let commit =
            proof_collection_commit_rename(&stored_assets, &session, "Stone Detail Normal")
                .expect("non-empty rename should commit");

        assert_eq!(commit.target_id, Arc::from("stone-normal"));
        assert_eq!(commit.previous_label, Arc::from("Stone Normal"));
        assert_eq!(commit.next_label, Arc::from("Stone Detail Normal"));
        assert_eq!(
            commit
                .renamed_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            stored_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            commit
                .renamed_assets
                .iter()
                .find(|asset| asset.id == Arc::from("stone-normal"))
                .map(|asset| asset.label.clone()),
            Some(Arc::from("Stone Detail Normal"))
        );
    }

    #[test]
    fn proof_collection_commit_rename_rejects_empty_trimmed_label() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone-normal"),
            original_label: Arc::from("Stone Normal"),
        };

        assert!(
            proof_collection_commit_rename(&stored_assets, &session, "   ").is_none(),
            "inline rename should reject empty trimmed labels so the app-local editor can stay open"
        );
    }

    #[test]
    fn proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile() {
        let collection_keys = vec![
            Arc::from("dust-mask"),
            Arc::from("pebble-height"),
            Arc::from("moss-overlay"),
        ];
        let selection = selection_state(&["moss-overlay"], Some("moss-overlay"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("pebble-height")),
        };

        let (next_selection, next_keyboard) =
            proof_collection_select_all_selection(&collection_keys, &selection, &keyboard)
                .expect("select-all should run when visible assets exist");

        assert_eq!(next_selection.selected(), collection_keys.as_slice());
        assert_eq!(anchor_id(&next_selection), Some("moss-overlay"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("pebble-height")));
    }

    #[test]
    fn proof_collection_select_all_selection_falls_back_to_first_visible_asset() {
        let collection_keys = vec![Arc::from("stone-albedo"), Arc::from("stone-normal")];
        let selection = ImUiMultiSelectState::default();
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("missing")),
        };

        let (next_selection, next_keyboard) =
            proof_collection_select_all_selection(&collection_keys, &selection, &keyboard)
                .expect("select-all should fall back to the first visible asset");

        assert_eq!(next_selection.selected(), collection_keys.as_slice());
        assert_eq!(anchor_id(&next_selection), Some("stone-albedo"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-albedo")));
    }

    #[test]
    fn proof_collection_select_all_shortcut_matches_primary_a_only() {
        assert!(proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        ));
        assert!(proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers::default(),
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                ctrl: true,
                alt: true,
                ..Default::default()
            },
        ));
    }

    #[test]
    fn proof_collection_duplicate_shortcut_matches_primary_d_only() {
        assert!(proof_collection_duplicate_shortcut_matches(
            KeyCode::KeyD,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        ));
        assert!(proof_collection_duplicate_shortcut_matches(
            KeyCode::KeyD,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_duplicate_shortcut_matches(
            KeyCode::KeyD,
            Modifiers::default(),
        ));
        assert!(!proof_collection_duplicate_shortcut_matches(
            KeyCode::KeyD,
            Modifiers {
                shift: true,
                meta: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_duplicate_shortcut_matches(
            KeyCode::KeyD,
            Modifiers {
                alt: true,
                ctrl: true,
                ..Default::default()
            },
        ));
    }

    #[test]
    fn proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let visible_assets = proof_collection_assets_in_visible_order(
            Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
            false,
        );
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-orm")),
        };

        let duplicate = proof_collection_duplicate_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
            false,
        )
        .expect("duplicate should run when selected assets exist");

        assert_eq!(
            duplicate
                .duplicated_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            vec![Arc::from("stone-normal-copy"), Arc::from("stone-orm-copy")]
        );
        assert_eq!(
            duplicate
                .duplicated_assets
                .iter()
                .map(|asset| asset.label.clone())
                .collect::<Vec<_>>(),
            vec![Arc::from("Stone Normal Copy"), Arc::from("Stone ORM Copy")]
        );
        assert_eq!(
            duplicate
                .next_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            vec![
                Arc::from("stone-albedo"),
                Arc::from("stone-normal"),
                Arc::from("stone-normal-copy"),
                Arc::from("stone-orm"),
                Arc::from("stone-orm-copy"),
                Arc::from("moss-overlay"),
                Arc::from("pebble-height"),
                Arc::from("dust-mask"),
            ]
        );
        assert_eq!(
            selected_ids(&duplicate.next_selection),
            vec!["stone-normal-copy", "stone-orm-copy"]
        );
        assert_eq!(
            anchor_id(&duplicate.next_selection),
            Some("stone-normal-copy")
        );
        assert_eq!(
            duplicate.next_keyboard.active_id,
            Some(Arc::from("stone-orm-copy"))
        );
    }

    #[test]
    fn proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists() {
        let mut stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        stored_assets.push(ProofCollectionAsset {
            id: Arc::from("stone-normal-copy"),
            label: Arc::from("Stone Normal Copy"),
            path: Arc::from("textures/stone/normal-copy.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 384,
        });
        let visible_assets = proof_collection_assets_in_visible_order(
            Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
            false,
        );
        let selection = selection_state(&["stone-normal"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let duplicate = proof_collection_duplicate_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
            false,
        )
        .expect("duplicate should generate a unique copy even when one already exists");

        assert_eq!(
            duplicate
                .duplicated_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            vec![Arc::from("stone-normal-copy-2")]
        );
        assert_eq!(
            duplicate
                .duplicated_assets
                .iter()
                .map(|asset| asset.label.clone())
                .collect::<Vec<_>>(),
            vec![Arc::from("Stone Normal Copy 2")]
        );
        assert_eq!(
            duplicate
                .duplicated_assets
                .iter()
                .map(|asset| asset.path.clone())
                .collect::<Vec<_>>(),
            vec![Arc::from("textures/stone/normal-copy-2.ktx2")]
        );
    }

    #[test]
    fn proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let visible_assets = proof_collection_assets_in_visible_order(
            Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
            false,
        );
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let delete = proof_collection_delete_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
        )
        .expect("delete should run when selected assets exist");

        assert_eq!(
            delete.deleted_ids,
            vec![Arc::from("stone-normal"), Arc::from("stone-orm")]
        );
        assert_eq!(
            delete
                .remaining_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            vec![
                Arc::from("stone-albedo"),
                Arc::from("moss-overlay"),
                Arc::from("pebble-height"),
                Arc::from("dust-mask"),
            ]
        );
        assert_eq!(selected_ids(&delete.next_selection), vec!["moss-overlay"]);
        assert_eq!(anchor_id(&delete.next_selection), Some("moss-overlay"));
        assert_eq!(
            delete.next_keyboard.active_id,
            Some(Arc::from("moss-overlay"))
        );
    }

    #[test]
    fn proof_collection_delete_selection_picks_previous_visible_item_at_end() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let visible_assets = proof_collection_assets_in_visible_order(
            Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
            false,
        );
        let selection = selection_state(&["dust-mask"], Some("dust-mask"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("dust-mask")),
        };

        let delete = proof_collection_delete_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
        )
        .expect("delete should run when the tail item is selected");

        assert_eq!(delete.deleted_ids, vec![Arc::from("dust-mask")]);
        assert_eq!(selected_ids(&delete.next_selection), vec!["pebble-height"]);
        assert_eq!(anchor_id(&delete.next_selection), Some("pebble-height"));
        assert_eq!(
            delete.next_keyboard.active_id,
            Some(Arc::from("pebble-height"))
        );
    }

    #[test]
    fn proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile() {
        let selection = selection_state(&["stone-albedo", "stone-normal"], Some("stone-albedo"));

        let (next_selection, next_keyboard) =
            proof_collection_context_menu_selection(&selection, Arc::from("dust-mask"));

        assert_eq!(selected_ids(&next_selection), vec!["dust-mask"]);
        assert_eq!(anchor_id(&next_selection), Some("dust-mask"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("dust-mask")));
    }

    #[test]
    fn proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile() {
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));

        let (next_selection, next_keyboard) =
            proof_collection_context_menu_selection(&selection, Arc::from("stone-orm"));

        assert_eq!(
            selected_ids(&next_selection),
            vec!["stone-normal", "stone-orm"]
        );
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-orm")));
    }
}
