use super::super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(in crate::runner::desktop::runner) fn handle_created_docking_window(
        &mut self,
        create: &CreateWindowRequest,
        new_window: fret_core::AppWindowId,
        now: Instant,
    ) {
        if matches!(
            create.kind,
            CreateWindowKind::DockFloating { .. } | CreateWindowKind::DockRestore { .. }
        ) {
            self.dock_floating_windows.insert(new_window);
        }

        let CreateWindowKind::DockFloating { source_window, .. } = &create.kind else {
            return;
        };

        #[cfg(target_os = "macos")]
        {
            // When tearing off during an active drag, macOS may create the new window behind the
            // source window. Bring it to front immediately so the subsequent follow movement keeps
            // ImGui-style multi-viewport hand feel.
            let sender = self.windows.get(*source_window).map(|w| w.window.as_ref());
            if let Some(state) = self.windows.get(new_window) {
                let _ = bring_window_to_front(state.window.as_ref(), sender);
            }
        }

        if let Some(anchor) = create.anchor
            && let Some(state) = self.windows.get(new_window)
            && let Some(pos) =
                self.compute_window_outer_position_from_cursor_grab(new_window, anchor.position)
        {
            let pos = match pos {
                WindowPosition::Logical(p) => winit::dpi::Position::Logical(
                    winit::dpi::LogicalPosition::new(p.x as f64, p.y as f64),
                ),
                WindowPosition::Physical(p) => {
                    winit::dpi::Position::Physical(winit::dpi::PhysicalPosition::new(p.x, p.y))
                }
            };
            state.window.set_outer_position(pos);
        }

        if self.is_left_mouse_down_for_window(*source_window) {
            let grab_offset = create
                .anchor
                .map(|a| a.position)
                .unwrap_or(Point::new(Px(40.0), Px(20.0)));
            let caps = self
                .app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let allow_follow = caps.ui.window_set_outer_position
                == fret_runtime::WindowSetOuterPositionQuality::Reliable;
            if allow_follow {
                let mut always_on_top_applied = false;
                if caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None
                    && let Some(state) = self.windows.get(new_window)
                {
                    state.window.set_window_level(WindowLevel::AlwaysOnTop);
                    always_on_top_applied = true;
                    self.app.with_global_mut(
                        fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
                        |svc, _app| {
                            svc.apply_style_patch(
                                new_window,
                                fret_runtime::WindowStyleRequest {
                                    z_level: Some(fret_runtime::WindowZLevel::AlwaysOnTop),
                                    ..Default::default()
                                },
                                &caps,
                            );
                        },
                    );
                }

                self.dock_tearoff_follow = Some(DockTearoffFollow {
                    window: new_window,
                    source_window: *source_window,
                    grab_offset,
                    manual_follow: true,
                    last_outer_pos: None,
                    transparent_payload_applied: false,
                    hit_test_passthrough_all_applied: false,
                    always_on_top_applied,
                });
                // Do not call `drag_window()` here. ImGui drives multi-viewport window movement by
                // updating the platform window position in response to mouse motion; native OS
                // dragging tends to introduce a fixed cursor offset and prevents reliable
                // hit-testing of other windows under the moving viewport.
            }
        }

        let panel = match &create.kind {
            CreateWindowKind::DockFloating { panel, .. } => Some(panel),
            _ => None,
        };
        self.enqueue_window_front(new_window, Some(*source_window), panel.cloned(), now);
    }
}
