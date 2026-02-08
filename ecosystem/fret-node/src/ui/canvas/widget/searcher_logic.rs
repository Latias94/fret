use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn record_recent_kind(&mut self, kind: &NodeKindKey) {
        const MAX_RECENT: usize = 20;

        self.interaction.recent_kinds.retain(|k| k != kind);
        self.interaction.recent_kinds.insert(0, kind.clone());
        if self.interaction.recent_kinds.len() > MAX_RECENT {
            self.interaction.recent_kinds.truncate(MAX_RECENT);
        }
    }

    pub(super) fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
        matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
    }

    pub(super) fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
        rows.iter()
            .position(Self::searcher_is_selectable_row)
            .unwrap_or(0)
    }

    pub(super) fn rebuild_searcher_rows(searcher: &mut SearcherState) {
        let rows = match &searcher.target {
            ContextMenuTarget::ConnectionConvertPicker { .. } => {
                crate::ui::canvas::searcher::build_rows_flat(&searcher.candidates, &searcher.query)
            }
            _ => crate::ui::canvas::searcher::build_rows(
                &searcher.candidates,
                &searcher.query,
                &searcher.recent_kinds,
            ),
        };

        searcher.rows = rows;
        searcher.scroll = searcher.scroll.min(
            searcher
                .rows
                .len()
                .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS),
        );
        searcher.active_row = Self::searcher_first_selectable_row(&searcher.rows)
            .min(searcher.rows.len().saturating_sub(1));
        Self::ensure_searcher_active_visible(searcher);
    }

    pub(super) fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
        let n = searcher.rows.len();
        if n == 0 {
            searcher.active_row = 0;
            searcher.scroll = 0;
            return;
        }

        let visible = SEARCHER_MAX_VISIBLE_ROWS.min(n);
        let max_scroll = n.saturating_sub(visible);
        searcher.scroll = searcher.scroll.min(max_scroll);

        if searcher.active_row < searcher.scroll {
            searcher.scroll = searcher.active_row;
        } else if searcher.active_row >= searcher.scroll + visible {
            searcher.scroll = (searcher.active_row + 1).saturating_sub(visible);
        }
        searcher.scroll = searcher.scroll.min(max_scroll);
    }

    pub(super) fn try_activate_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
    ) -> bool {
        let Some(searcher) = self.interaction.searcher.take() else {
            return false;
        };

        let Some(row) = searcher.rows.get(row_ix).cloned() else {
            self.interaction.searcher = Some(searcher);
            return false;
        };

        let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
            self.interaction.searcher = Some(searcher);
            return false;
        };
        if !row.enabled {
            self.interaction.searcher = Some(searcher);
            return false;
        }

        let item = NodeGraphContextMenuItem {
            label: row.label,
            enabled: true,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
        };
        self.activate_context_menu_item(
            cx,
            &searcher.target,
            searcher.invoked_at,
            item,
            &searcher.candidates,
        );
        true
    }

    pub(super) fn open_insert_node_picker<H: UiHost>(&mut self, host: &mut H, at: CanvasPoint) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| presenter.list_insertable_nodes(graph))
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows = crate::ui::canvas::searcher::build_rows(
            &menu_candidates,
            "",
            &self.interaction.recent_kinds,
        );
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::BackgroundInsertNodePicker { at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    pub(super) fn open_connection_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
    ) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_connection(graph, from)
                })
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows = crate::ui::canvas::searcher::build_rows(
            &menu_candidates,
            "",
            &self.interaction.recent_kinds,
        );
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::ConnectionInsertNodePicker { from, at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    pub(super) fn open_edge_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        edge: EdgeId,
        invoked_at: Point,
    ) {
        super::edge_insert::open_edge_insert_node_picker(self, host, window, edge, invoked_at);
    }
}
