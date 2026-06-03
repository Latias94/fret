use super::super::*;

fn env_flag_is_true(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| {
        let v = v.to_string_lossy();
        let v = v.trim();
        !(v.eq_ignore_ascii_case("0")
            || v.eq_ignore_ascii_case("false")
            || v.eq_ignore_ascii_case("off")
            || v.eq_ignore_ascii_case("no"))
    })
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(in crate::runner::desktop::runner) fn update_dock_tearoff_follow(&mut self) -> bool {
        let pointer_id = self.dock_drag_pointer_id();
        if self.dock_tearoff_follow.is_some() && pointer_id.is_none() {
            // If the dock drag session was canceled (e.g. Escape), ensure we do not keep moving a
            // dock tear-off window indefinitely.
            self.stop_dock_tearoff_follow(Instant::now(), false);
            return true;
        }

        let settings = self
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let want_transparent_payload = settings.transparent_payload_during_follow
            || env_flag_is_true("FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD");
        let allow_diag_follow = env_flag_is_true("FRET_DOCK_TEAROFF_FOLLOW_IN_DIAG");
        let diag_pointer_input_isolation_active = self.diag_pointer_input_isolation_active();

        // Scripted diagnostics drive cursor position via overrides. Tear-off follow intentionally
        // moves OS windows to keep a real cursor "inside" the moving window; during scripted runs
        // this causes the runner to chase synthetic cursor updates and can prevent docking-back
        // gestures from ever reaching a stable overlap/hover state.
        //
        // Preserve transparent payload state when explicitly requested, but freeze the actual
        // follow motion so overlap diagnostics can still observe peek-behind behavior without
        // chasing the synthetic cursor.
        if diag_pointer_input_isolation_active && !want_transparent_payload && !allow_diag_follow {
            if self.dock_tearoff_follow.is_some() {
                self.stop_dock_tearoff_follow(Instant::now(), false);
                return true;
            }
            return false;
        }

        if self.dock_tearoff_follow.is_none()
            && let Some(pointer_id) = pointer_id
            && let Some(drag) = self.app.drag(pointer_id)
        {
            let grab_offset = drag
                .cursor_grab_offset
                .unwrap_or(Point::new(Px(40.0), Px(20.0)));

            // Transparent payload is primarily an ImGui-style "peek behind moving window" aid. It
            // only makes sense to force-follow OS windows that are already dock-floating (tear-off
            // windows). For normal app windows, forcing follow prevents out-of-bounds tear-off
            // heuristics from ever stabilizing.
            let force_follow_for_transparent_payload = want_transparent_payload
                && matches!(
                    drag.kind,
                    fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                )
                && self.dock_floating_windows.contains(&drag.source_window);

            let follow_window = if force_follow_for_transparent_payload {
                Some(drag.source_window)
            } else {
                drag.follow_window
            };

            if let Some(window) = follow_window {
                self.dock_tearoff_follow = Some(DockTearoffFollow {
                    window,
                    source_window: drag.source_window,
                    grab_offset,
                    manual_follow: true,
                    last_outer_pos: None,
                    transparent_payload_applied: false,
                    hit_test_passthrough_all_applied: false,
                    always_on_top_applied: false,
                });
            }
        }

        let (window, grab_offset, manual_follow, last_outer_pos, transparent_payload_applied) =
            match self.dock_tearoff_follow {
                Some(follow) => (
                    follow.window,
                    follow.grab_offset,
                    follow.manual_follow,
                    follow.last_outer_pos,
                    follow.transparent_payload_applied,
                ),
                None => return false,
            };

        if !manual_follow {
            return false;
        }

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if caps.ui.window_set_outer_position
            != fret_runtime::WindowSetOuterPositionQuality::Reliable
        {
            return false;
        }

        if self.windows.get(window).is_none() {
            self.dock_tearoff_follow = None;
            return false;
        }

        // Optional ImGui-style "transparent payload" behavior while following the cursor:
        // - make the dock-floating window semi-transparent
        // - (best-effort) make the dock-floating window ignore mouse events (click-through)
        //
        // This is conservatively disabled by default (see `DockingInteractionSettings`), and can
        // be forced on via env var for quick experimentation.
        if want_transparent_payload != transparent_payload_applied {
            let opacity = if want_transparent_payload {
                fret_runtime::WindowOpacity::from_f32(settings.transparent_payload_alpha)
            } else {
                fret_runtime::WindowOpacity(255)
            };
            self.app.push_effect(fret_app::Effect::Window(
                fret_app::WindowRequest::SetStyle {
                    window,
                    style: fret_runtime::WindowStyleRequest {
                        hit_test: Some(if want_transparent_payload {
                            fret_runtime::WindowHitTestRequestV1::PassthroughAll
                        } else {
                            fret_runtime::WindowHitTestRequestV1::Normal
                        }),
                        opacity: Some(opacity),
                        ..Default::default()
                    },
                },
            ));
            if let Some(follow) = self.dock_tearoff_follow.as_mut() {
                follow.transparent_payload_applied = want_transparent_payload;
            }
            if let Some(pointer_id) = pointer_id
                && let Some(drag) = self.app.drag_mut(pointer_id)
            {
                drag.transparent_payload_applied = want_transparent_payload;
                if !want_transparent_payload {
                    drag.transparent_payload_hit_test_passthrough_applied = false;
                }
            }
        }

        let Some(pos) = self.compute_window_outer_position_from_cursor_grab(window, grab_offset)
        else {
            return false;
        };

        let next_phys = {
            let Some(state) = self.windows.get(window) else {
                self.dock_tearoff_follow = None;
                return false;
            };
            let scale_factor = state.window.scale_factor();
            match pos {
                WindowPosition::Physical(p) => winit::dpi::PhysicalPosition::new(p.x, p.y),
                WindowPosition::Logical(p) => {
                    winit::dpi::LogicalPosition::new(p.x as f64, p.y as f64)
                        .to_physical::<i32>(scale_factor)
                }
            }
        };

        // Avoid spamming redundant position updates (helps reduce stutter on high-frequency
        // input devices).
        if last_outer_pos.is_some_and(|prev| prev == next_phys) {
            return false;
        }

        if let Some(state) = self.windows.get(window) {
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

        dock_tearoff_log(format_args!(
            "[follow-move] window={:?} cursor={:?} outer_pos={:?}",
            window, self.cursor_screen_pos, next_phys
        ));

        if let Some(follow) = self.dock_tearoff_follow.as_mut() {
            follow.last_outer_pos = Some(next_phys);
        }

        true
    }

    pub(in crate::runner::desktop::runner) fn stop_dock_tearoff_follow(
        &mut self,
        _now: Instant,
        _raise_on_macos: bool,
    ) {
        let Some(follow) = self.dock_tearoff_follow.take() else {
            return;
        };

        dock_tearoff_log(format_args!(
            "[follow-stop] window={:?} source={:?} cursor={:?} raise_on_macos={}",
            follow.window, follow.source_window, self.cursor_screen_pos, _raise_on_macos
        ));

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();

        if follow.transparent_payload_applied {
            self.app.push_effect(fret_app::Effect::Window(
                fret_app::WindowRequest::SetStyle {
                    window: follow.window,
                    style: fret_runtime::WindowStyleRequest {
                        hit_test: Some(fret_runtime::WindowHitTestRequestV1::Normal),
                        opacity: Some(fret_runtime::WindowOpacity(255)),
                        ..Default::default()
                    },
                },
            ));
        }
        if let Some(pointer_id) = self.dock_drag_pointer_id()
            && let Some(drag) = self.app.drag_mut(pointer_id)
        {
            drag.transparent_payload_applied = false;
            drag.transparent_payload_hit_test_passthrough_applied = false;
        }

        if let Some(state) = self.windows.get(follow.window) {
            if caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None
                && follow.always_on_top_applied
            {
                self.app.push_effect(fret_app::Effect::Window(
                    fret_app::WindowRequest::SetStyle {
                        window: follow.window,
                        style: fret_runtime::WindowStyleRequest {
                            z_level: Some(fret_runtime::WindowZLevel::Normal),
                            ..Default::default()
                        },
                    },
                ));
            }
            if caps.ui.window_set_outer_position
                == fret_runtime::WindowSetOuterPositionQuality::Reliable
                && let Some(pos) =
                    self.settle_window_outer_position(state.window.as_ref(), self.cursor_screen_pos)
            {
                state.window.set_outer_position(Position::Physical(pos));
            }
        }

        #[cfg(target_os = "macos")]
        if _raise_on_macos {
            self.enqueue_window_front(follow.window, Some(follow.source_window), None, _now);
        }
    }
}
