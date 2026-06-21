impl UiTreeDebugSnapshotV1 {
    fn from_tree(
        app: &App,
        window: AppWindowId,
        ui: &UiTree<App>,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
        wgpu_hub_report: Option<fret_render::WgpuHubReportFrameSample>,
        wgpu_allocator_report: Option<fret_render::WgpuAllocatorReportFrameSample>,
        element_runtime_state: Option<&ElementRuntime>,
        hit_test: Option<UiHitTestSnapshotV1>,
        element_runtime_snapshot: Option<ElementDiagnosticsSnapshotV1>,
        semantics: Option<UiSemanticsSnapshotV1>,
        max_gating_trace_entries: usize,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        let contained_relayout_roots: HashSet<fret_core::NodeId> = ui
            .debug_view_cache_contained_relayout_roots()
            .iter()
            .copied()
            .collect();
        let environment = element_runtime_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.environment.clone());

        let window_insets = app.global::<fret_core::WindowMetricsService>().map(|svc| {
            let safe_area_known = svc.safe_area_insets_is_known(window);
            let safe_area_insets_px = svc.safe_area_insets(window).map(|e| UiPaddingInsetsV1 {
                left_px: e.left.0,
                top_px: e.top.0,
                right_px: e.right.0,
                bottom_px: e.bottom.0,
            });
            let occlusion_known = svc.occlusion_insets_is_known(window);
            let occlusion_insets_px = svc.occlusion_insets(window).map(|e| UiPaddingInsetsV1 {
                left_px: e.left.0,
                top_px: e.top.0,
                right_px: e.right.0,
                bottom_px: e.bottom.0,
            });
            UiWindowInsetsSnapshotV1 {
                safe_area_known,
                safe_area_insets_px,
                occlusion_known,
                occlusion_insets_px,
            }
        });

        let text_input = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .map(|snapshot| UiWindowTextInputSnapshotV1 {
                focus_is_text_input: snapshot.focus_is_text_input,
                is_composing: snapshot.is_composing,
                text_len_utf16: snapshot.text_len_utf16,
                selection_utf16: snapshot.selection_utf16,
                marked_utf16: snapshot.marked_utf16,
                ime_cursor_area: snapshot.ime_cursor_area.map(RectV1::from),
                visual: snapshot.visual.map(|visual| UiTextInputVisualSnapshotV1 {
                    viewport_bounds: UiRectV1 {
                        x_px: visual.viewport_bounds.origin.x.0,
                        y_px: visual.viewport_bounds.origin.y.0,
                        w_px: visual.viewport_bounds.size.width.0,
                        h_px: visual.viewport_bounds.size.height.0,
                    },
                    clip_bounds: UiRectV1 {
                        x_px: visual.clip_bounds.origin.x.0,
                        y_px: visual.clip_bounds.origin.y.0,
                        w_px: visual.clip_bounds.size.width.0,
                        h_px: visual.clip_bounds.size.height.0,
                    },
                    unclipped_text_bounds: UiRectV1 {
                        x_px: visual.unclipped_text_bounds.origin.x.0,
                        y_px: visual.unclipped_text_bounds.origin.y.0,
                        w_px: visual.unclipped_text_bounds.size.width.0,
                        h_px: visual.unclipped_text_bounds.size.height.0,
                    },
                    visible_text_bounds: visual.visible_text_bounds.map(|bounds| UiRectV1 {
                        x_px: bounds.origin.x.0,
                        y_px: bounds.origin.y.0,
                        w_px: bounds.size.width.0,
                        h_px: bounds.size.height.0,
                    }),
                    content_width_px: visual.content_width_px,
                    viewport_width_px: visual.viewport_width_px,
                    offset_x_px: visual.offset_x_px,
                    max_offset_x_px: visual.max_offset_x_px,
                }),
            });

        let runner_surface_lifecycle = app
            .global::<fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore>()
            .map(|store| store.snapshot())
            .map(|snapshot| UiRunnerSurfaceLifecycleSnapshotV1 {
                can_create_surfaces_calls: snapshot.can_create_surfaces_calls,
                destroy_surfaces_calls: snapshot.destroy_surfaces_calls,
                last_can_create_surfaces_unix_ms: snapshot.last_can_create_surfaces_unix_ms,
                last_destroy_surfaces_unix_ms: snapshot.last_destroy_surfaces_unix_ms,
                surfaces_available: snapshot.surfaces_available,
            });

        let runner_accessibility = Some({
            let snapshot = app
                .global::<fret_runtime::RunnerAccessibilityDiagnosticsStore>()
                .and_then(|store| store.snapshot(window))
                .unwrap_or_default();
            UiRunnerAccessibilitySnapshotV1 {
                activation_requests: snapshot.activation_requests,
                last_activation_unix_ms: snapshot.last_activation_unix_ms,
                last_activation_frame_id: snapshot.last_activation_frame_id.map(|id| id.0),
            }
        });

        let resource_loading = {
            let asset_load = fret_runtime::asset_resolver(app)
                .map(|service| service.diagnostics_snapshot())
                .filter(|snapshot| snapshot.total_requests > 0)
                .map(UiAssetLoadDiagnosticsSnapshotV1::from_runtime);
            let asset_reload = UiAssetReloadDiagnosticsSnapshotV1::from_runtime(
                fret_runtime::asset_reload_epoch(app),
                fret_runtime::asset_reload_support(app),
                fret_runtime::asset_reload_status(app),
            );
            let font_environment = UiFontEnvironmentDiagnosticsSnapshotV1::from_runtime(
                app.global::<fret_runtime::BundledFontBaselineSnapshot>(),
                app.global::<fret_runtime::FontCatalog>(),
                app.global::<fret_runtime::RendererFontEnvironmentSnapshot>(),
                app.global::<fret_runtime::TextFontStackKey>()
                    .map(|key| key.0),
                app.global::<fret_runtime::SystemFontRescanState>().copied(),
            );
            let svg_text_bridge = UiSvgTextBridgeDiagnosticsSnapshotV1::from_runtime(
                app.global::<fret_runtime::RendererSvgTextBridgeDiagnosticsSnapshot>(),
            );
            (asset_load.is_some()
                || asset_reload.is_some()
                || font_environment.is_some()
                || svg_text_bridge.is_some())
                .then_some(UiResourceLoadingDiagnosticsSnapshotV1 {
                    asset_load,
                    asset_reload,
                    font_environment,
                    svg_text_bridge,
                })
        };

        let debug_cache_root_stats = ui.debug_cache_root_stats();
        let cache_root_outcome_by_root: HashMap<u64, UiCacheRootBoundaryOutcomeV1> =
            debug_cache_root_stats
                .iter()
                .map(|stats| {
                    let root = stats.root.data().as_ffi();
                    let contained_relayout_in_frame = contained_relayout_roots.contains(&stats.root);
                    (
                        root,
                        UiBoundaryDiagnosticsV1::cache_root_outcome_from_debug_stats(
                            stats,
                            contained_relayout_in_frame,
                        ),
                    )
                })
                .collect();
        let cache_roots: Vec<UiCacheRootStatsV1> = debug_cache_root_stats
            .iter()
            .map(|stats| {
                UiCacheRootStatsV1::from_stats(
                    window,
                    ui,
                    element_runtime_state,
                    semantics.as_ref(),
                    &contained_relayout_roots,
                    stats,
                    max_debug_string_bytes,
                )
            })
            .collect();
        let cache_root_by_root: HashMap<u64, &UiCacheRootStatsV1> =
            cache_roots.iter().map(|root| (root.root, root)).collect();
        let boundaries: Vec<UiBoundaryDiagnosticsV1> = ui
            .debug_boundary_stats()
            .iter()
            .map(|boundary| {
                UiBoundaryDiagnosticsV1::from_boundary_stats(
                    window,
                    element_runtime_state,
                    boundary,
                    cache_root_by_root.get(&boundary.id.data().as_ffi()).copied(),
                    cache_root_outcome_by_root
                        .get(&boundary.id.data().as_ffi())
                        .copied(),
                    max_debug_string_bytes,
                )
            })
            .collect();

        let removed_subtrees: Vec<UiRemovedSubtreeV1> = ui
            .debug_removed_subtrees()
            .iter()
            .map(|r| {
                UiRemovedSubtreeV1::from_record(
                    window,
                    ui,
                    element_runtime_state,
                    r,
                    max_debug_string_bytes,
                )
            })
            .collect();

        let mut layout_engine_solves: Vec<UiLayoutEngineSolveV1> = ui
            .debug_layout_engine_solves()
            .iter()
            .map(UiLayoutEngineSolveV1::from_solve)
            .collect();
        for s in &mut layout_engine_solves {
            truncate_opt_string_bytes(&mut s.root_element_path, max_debug_string_bytes);
        }

        let mut layout_request_build_roots: Vec<UiLayoutRequestBuildRootV1> = ui
            .debug_layout_request_build_roots()
            .iter()
            .map(UiLayoutRequestBuildRootV1::from_record)
            .collect();
        for r in &mut layout_request_build_roots {
            truncate_opt_string_bytes(&mut r.root_element_path, max_debug_string_bytes);
            for d in &mut r.dirty_descendants {
                truncate_opt_string_bytes(&mut d.element_path, max_debug_string_bytes);
            }
        }

        let mut layout_root_applies: Vec<UiLayoutRootApplyV1> = ui
            .debug_layout_root_applies()
            .iter()
            .map(UiLayoutRootApplyV1::from_record)
            .collect();
        for r in &mut layout_root_applies {
            truncate_opt_string_bytes(&mut r.root_element_path, max_debug_string_bytes);
        }

        let mut layout_hotspots: Vec<UiLayoutHotspotV1> = ui
            .debug_layout_hotspots()
            .iter()
            .map(UiLayoutHotspotV1::from_hotspot)
            .collect();
        for h in &mut layout_hotspots {
            truncate_opt_string_bytes(&mut h.element_path, max_debug_string_bytes);
        }

        let mut widget_measure_hotspots: Vec<UiWidgetMeasureHotspotV1> = ui
            .debug_widget_measure_hotspots()
            .iter()
            .map(UiWidgetMeasureHotspotV1::from_hotspot)
            .collect();
        for h in &mut widget_measure_hotspots {
            truncate_opt_string_bytes(&mut h.element_path, max_debug_string_bytes);
        }

        Self {
            stats: UiFrameStatsV1::from_stats(
                ui.debug_stats(),
                renderer_perf,
                wgpu_hub_report,
                wgpu_allocator_report,
            ),
            invalidation_walks: ui
                .debug_invalidation_walks()
                .iter()
                .map(|w| UiInvalidationWalkV1::from_walk(w, window, element_runtime_state))
                .collect(),
            hover_declarative_invalidation_hotspots: ui
                .debug_hover_declarative_invalidation_hotspots(20)
                .into_iter()
                .map(UiHoverDeclarativeInvalidationHotspotV1::from_hotspot)
                .collect(),
            dirty_views: ui
                .debug_dirty_views()
                .iter()
                .map(UiDirtyViewV1::from_dirty_view)
                .collect(),
            notify_requests: ui
                .debug_notify_requests()
                .iter()
                .map(UiNotifyRequestV1::from_notify_request)
                .collect(),
            virtual_list_windows: ui
                .debug_virtual_list_windows()
                .iter()
                .map(UiVirtualListWindowV1::from_window)
                .collect(),
            virtual_list_window_shift_samples: ui
                .debug_virtual_list_window_shift_samples()
                .iter()
                .map(UiVirtualListWindowShiftSampleV1::from_sample)
                .collect(),
            windowed_rows_surfaces: app
                .global::<fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceDiagnosticsStore>(
                )
                .and_then(|store| store.windows_for_window(window, app.frame_id()))
                .map(|windows| {
                    windows
                        .iter()
                        .map(UiWindowedRowsSurfaceWindowV1::from_telemetry)
                        .collect()
                })
                .unwrap_or_default(),
            retained_virtual_list_reconciles: ui
                .debug_retained_virtual_list_reconciles()
                .iter()
                .map(UiRetainedVirtualListReconcileV1::from_record)
                .collect(),
            scroll_handle_changes: ui
                .debug_scroll_handle_changes()
                .iter()
                .map(UiScrollHandleChangeV1::from_change)
                .collect(),
            scroll_nodes: ui
                .debug_scroll_nodes()
                .iter()
                .map(UiScrollNodeTelemetryV1::from_record)
                .collect(),
            prepaint_actions: ui
                .debug_prepaint_actions()
                .iter()
                .map(UiPrepaintActionV1::from_action)
                .collect(),
            model_change_hotspots: ui
                .debug_model_change_hotspots()
                .iter()
                .map(UiModelChangeHotspotV1::from_hotspot)
                .collect(),
            model_change_unobserved: ui
                .debug_model_change_unobserved()
                .iter()
                .map(UiModelChangeUnobservedV1::from_unobserved)
                .collect(),
            global_change_hotspots: ui
                .debug_global_change_hotspots()
                .iter()
                .map(|h| UiGlobalChangeHotspotV1::from_hotspot(app, h))
                .collect(),
            global_change_unobserved: ui
                .debug_global_change_unobserved()
                .iter()
                .map(|u| UiGlobalChangeUnobservedV1::from_unobserved(app, u))
                .collect(),
            cache_roots,
            boundaries,
            overlay_synthesis: app
                .global::<fret_ui_kit::WindowOverlaySynthesisDiagnosticsStore>()
                .and_then(|diag| diag.events_for_window(window, app.frame_id()))
                .map(|events| {
                    events
                        .iter()
                        .copied()
                        .map(UiOverlaySynthesisEventV1::from_event)
                        .collect()
                })
                .unwrap_or_default(),
            viewport_input: Vec::new(),
            web_ime_bridge: app
                .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                .filter(|snapshot| {
                    **snapshot != fret_core::input::WebImeBridgeDebugSnapshot::default()
                })
                .map(|snapshot| {
                    UiWebImeBridgeDebugSnapshotV1::from_snapshot(
                        snapshot,
                        redact_text,
                        max_debug_string_bytes,
                    )
                }),
            docking_interaction: app
                .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                .and_then(|store| {
                    let frame = store.docking_for_window(window, app.frame_id());
                    let has_frame_data = frame.is_some_and(|d| {
                        d.dock_drag.is_some()
                            || d.floating_drag.is_some()
                            || d.dock_drop_resolve.is_some()
                            || d.viewport_capture.is_some()
                            || d.tab_strip_active_visibility.is_some()
                            || d.dock_graph_stats.is_some()
                            || d.dock_graph_signature.is_some()
                    });
                    if has_frame_data {
                        frame
                    } else {
                        store.docking_latest_for_window(window)
                    }
                })
                .map(UiDockingInteractionSnapshotV1::from_snapshot),
            workspace_interaction: app
                .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                .and_then(|store| {
                    let frame = store.workspace_for_window(window, app.frame_id());
                    let has_frame_data = frame.is_some_and(|w| !w.tab_strip_active_visibility.is_empty());
                    if has_frame_data {
                        frame
                    } else {
                        store.workspace_latest_for_window(window)
                    }
                })
                .map(UiWorkspaceInteractionSnapshotV1::from_snapshot),
            removed_subtrees,
            layout_request_build_roots,
            layout_root_applies,
            layout_engine_solves,
            layout_hotspots,
            widget_measure_hotspots,
            paint_widget_hotspots: ui
                .debug_paint_widget_hotspots()
                .iter()
                .map(UiPaintWidgetHotspotV1::from_hotspot)
                .collect(),
            paint_text_prepare_hotspots: ui
                .debug_paint_text_prepare_hotspots()
                .iter()
                .map(UiPaintTextPrepareHotspotV1::from_hotspot)
                .collect(),
            command_availability_hotspots: ui
                .debug_command_availability_hotspots()
                .iter()
                .map(|h| {
                    UiCommandAvailabilityHotspotV1::from_hotspot(
                        h,
                        window,
                        element_runtime_state,
                    )
                })
                .collect(),
            input_arbitration: UiInputArbitrationSnapshotV1::from_snapshot(
                ui.input_arbitration_snapshot(),
            ),
            command_gating_trace: command_gating_trace_for_window(
                app,
                window,
                max_gating_trace_entries,
            ),
            command_dispatch_trace: app
                .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
                .map(|store| {
                    store
                        .decisions_for_frame(window, app.frame_id(), max_gating_trace_entries)
                         .into_iter()
                         .map(|decision| UiCommandDispatchTraceEntryV1 {
                             command: decision.command.as_str().to_string(),
                             handled: decision.handled,
                             handled_by_scope: decision.handled_by_scope.map(|s| match s {
                                 fret_runtime::CommandScope::Widget => "widget".to_string(),
                                 fret_runtime::CommandScope::Window => "window".to_string(),
                                 fret_runtime::CommandScope::App => "app".to_string(),
                             }),
                             handled_by_driver: decision.handled_by_driver,
                             stopped: decision.stopped,
                             source_kind: match decision.source.kind {
                                 fret_runtime::CommandDispatchSourceKindV1::Pointer => {
                                     "pointer".to_string()
                                 }
                                 fret_runtime::CommandDispatchSourceKindV1::Keyboard => {
                                     "keyboard".to_string()
                                 }
                                 fret_runtime::CommandDispatchSourceKindV1::Shortcut => {
                                     "shortcut".to_string()
                                 }
                                 fret_runtime::CommandDispatchSourceKindV1::Programmatic => {
                                     "programmatic".to_string()
                                 }
                             },
                             source_element: decision.source.element,
                             handled_by_element: decision.handled_by_element,
                             started_from_focus: decision.started_from_focus,
                             used_default_root_fallback: decision.used_default_root_fallback,
                         })
                         .collect()
                 })
                 .unwrap_or_default(),
            layers_in_paint_order: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(UiLayerInfoV1::from_layer)
                .collect(),
            all_layer_roots: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(|l| l.root.data().as_ffi())
                .collect(),
            layer_visible_writes: ui
                .debug_layer_visible_writes()
                .iter()
                .map(UiLayerVisibleWriteV1::from_write)
                .collect(),
            overlay_policy_decisions: ui
                .debug_overlay_policy_decisions()
                .iter()
                .map(UiOverlayPolicyDecisionV1::from_decision)
                .collect(),
            environment,
            extensions: None,
            window_insets,
            text_input,
            runner_surface_lifecycle,
            runner_accessibility,
            resource_loading,
            hit_test,
            element_runtime: element_runtime_snapshot,
            semantics,
        }
    }
}
