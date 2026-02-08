use super::*;

impl DockGraph {
    pub fn apply_op_checked(&mut self, op: &DockOp) -> Result<bool, DockOpApplyError> {
        match op {
            DockOp::SetActiveTab { tabs, active } => {
                let Some(node) = self.nodes.get(*tabs) else {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::TabsNodeNotFound { tabs: *tabs },
                    });
                };
                let DockNode::Tabs { tabs: list, .. } = node else {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::NodeIsNotTabs { node: *tabs },
                    });
                };
                if *active >= list.len() {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::ActiveOutOfBounds {
                            tabs: *tabs,
                            active: *active,
                            len: list.len(),
                        },
                    });
                }
                Ok(self.set_active_tab(*tabs, *active))
            }
            DockOp::ClosePanel { window, panel } => {
                if self.close_panel(*window, panel.clone()) {
                    Ok(true)
                } else {
                    Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::PanelNotFound {
                            window: *window,
                            panel: panel.clone(),
                        },
                    })
                }
            }
            DockOp::RequestFloatPanelToNewWindow { .. } => Err(DockOpApplyError {
                kind: DockOpApplyErrorKind::UnsupportedOp,
            }),
            _ => Ok(self.apply_op(op)),
        }
    }

    pub fn apply_op(&mut self, op: &DockOp) -> bool {
        match op {
            DockOp::SetActiveTab { tabs, active } => self.set_active_tab(*tabs, *active),
            DockOp::ClosePanel { window, panel } => self.close_panel(*window, panel.clone()),
            DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs,
                zone,
                insert_index,
            } => self.move_panel_between_windows(
                *source_window,
                panel.clone(),
                *target_window,
                *target_tabs,
                *zone,
                *insert_index,
            ),
            DockOp::MoveTabs {
                source_window,
                source_tabs,
                target_window,
                target_tabs,
                zone,
                insert_index,
            } => self.move_tabs_between_windows(
                *source_window,
                *source_tabs,
                *target_window,
                *target_tabs,
                *zone,
                *insert_index,
            ),
            DockOp::FloatPanelToWindow {
                source_window,
                panel,
                new_window,
            } => self.float_panel_to_window(*source_window, panel.clone(), *new_window),
            DockOp::RequestFloatPanelToNewWindow { .. } => false,
            DockOp::FloatPanelInWindow {
                source_window,
                panel,
                target_window,
                rect,
            } => self.float_panel_in_window(*source_window, panel.clone(), *target_window, *rect),
            DockOp::FloatTabsInWindow {
                source_window,
                source_tabs,
                target_window,
                rect,
            } => self.float_tabs_in_window(*source_window, *source_tabs, *target_window, *rect),
            DockOp::SetFloatingRect {
                window,
                floating,
                rect,
            } => self.set_floating_rect(*window, *floating, *rect),
            DockOp::RaiseFloating { window, floating } => self.raise_floating(*window, *floating),
            DockOp::MergeFloatingInto {
                window,
                floating,
                target_tabs,
            } => self.merge_floating_into(*window, *floating, *target_tabs),
            DockOp::MergeWindowInto {
                source_window,
                target_window,
                target_tabs,
            } => {
                let panels = self.collect_panels_in_window(*source_window);
                for panel in panels {
                    let _ = self.move_panel_between_windows(
                        *source_window,
                        panel,
                        *target_window,
                        *target_tabs,
                        DropZone::Center,
                        None,
                    );
                }
                let _ = self.remove_window_root(*source_window);
                let _ = self.window_floatings.remove(source_window);
                true
            }
            DockOp::SetSplitFractions { split, fractions } => {
                self.update_split_fractions(*split, fractions.clone())
            }
            DockOp::SetSplitFractionsMany { updates } => {
                let mut changed = false;
                for u in updates {
                    changed |= self.update_split_fractions(u.split, u.fractions.clone());
                }
                changed
            }
            DockOp::SetSplitFractionTwo {
                split,
                first_fraction,
            } => self.update_split_two(*split, *first_fraction),
        }
    }
}
