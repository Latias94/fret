import fs from "fs"
import path from "path"
import { createRequire } from "module"
import { execSync } from "child_process"

type CaseId =
  | "demo_process"
  | "auto_reset"
  | "resets"
  | "pagination"
  | "sort_undefined"
  | "sorting_fns"
  | "filtering_fns"
  | "headers_cells"
  | "headers_inventory_deep"
  | "visibility_ordering"
  | "pinning"
  | "pinning_tree"
  | "pinning_grouped_rows"
  | "column_pinning"
  | "faceting"
  | "column_sizing"
  | "column_resizing_group_headers"
  | "state_shapes"
  | "selection"
  | "selection_tree"
  | "expanding"
  | "grouping"
  | "grouping_aggregation_fns"
  | "row_id_state_ops"
  | "render_fallback"

type SnapshotId =
  | "baseline"
  | "headers_inventory_deep_baseline"
  | "headers_inventory_deep_pin_one_leaf_left"
  | "headers_inventory_deep_hide_deep_leaf"
  | "headers_inventory_deep_hide_whole_branch"
  | "headers_inventory_deep_order_reorders_across_depths"
  | "auto_reset_sorting_default_resets"
  | "auto_reset_sorting_manual_pagination_true_no_reset"
  | "auto_reset_sorting_manual_pagination_true_auto_reset_page_index_true_overrides_manual"
  | "auto_reset_sorting_auto_reset_all_false_disables"
  | "auto_reset_global_filter_default_resets"
  | "auto_reset_global_filter_manual_pagination_true_no_reset"
  | "auto_reset_global_filter_manual_pagination_true_auto_reset_page_index_true_overrides_manual"
  | "auto_reset_global_filter_auto_reset_all_false_disables"
  | "resets_reset_sorting_restores_initial"
  | "resets_reset_sorting_default_true_clears"
  | "resets_reset_column_filters_restores_initial"
  | "resets_reset_column_filters_default_true_clears"
  | "resets_reset_global_filter_restores_initial"
  | "resets_reset_global_filter_default_true_clears"
  | "resets_reset_grouping_restores_initial"
  | "resets_reset_grouping_default_true_clears"
  | "resets_reset_column_visibility_restores_initial"
  | "resets_reset_column_visibility_default_true_clears"
  | "resets_reset_column_order_restores_initial"
  | "resets_reset_column_order_default_true_clears"
  | "resets_reset_row_selection_restores_initial"
  | "resets_reset_row_selection_default_true_clears"
  | "sorted_cpu_desc"
  | "sorted_cpu_invert_asc"
  | "sorted_cpu_toggle_desc_first"
  | "sorted_cpu_toggle_no_removal"
  | "sorted_multi_max_1_keeps_latest"
  | "sorted_multi_max_2_drops_oldest"
  | "sorted_multi_disabled_replaces"
  | "sorted_multi_column_disabled_replaces"
  | "sorted_handler_table_sorting_disabled_noop"
  | "sorted_handler_column_sorting_disabled_noop"
  | "sorted_handler_multi_event_adds_when_allowed"
  | "filter_status_run"
  | "page_0_size_2"
  | "sort_undefined_first_asc"
  | "sort_undefined_last_asc"
  | "sort_undefined_1_asc"
  | "sort_undefined_1_desc"
  | "sort_undefined_neg1_asc"
  | "sort_undefined_neg1_desc"
  | "sort_undefined_false_text_asc"
  | "sort_undefined_false_text_desc"
  | "colsize_pinned_order_defaults"
  | "colsize_override_and_clamp"
  | "colsize_resize_on_change_move_updates"
  | "colsize_resize_on_change_end_resets"
  | "colsize_resize_on_end_move_no_sizing"
  | "colsize_resize_on_end_end_writes"
  | "colsize_resize_rtl_move_flips"
  | "colsize_enable_column_resizing_false_noops"
  | "colsize_reset_column_size_removes_override"
  | "colsize_reset_column_sizing_default_true_clears"
  | "colsize_reset_column_sizing_restores_initial"
  | "colsize_reset_header_size_info_default_true_clears"
  | "colsize_hook_noop_sizing_move_keeps_sizing"
  | "colsize_hook_noop_sizing_reset_column_sizing_keeps_state"
  | "colsize_hook_noop_info_move_keeps_info_and_sizing"
  | "colsize_hook_noop_info_reset_header_size_info_keeps_state"
  | "group_resize_on_change_move_updates"
  | "group_resize_on_change_end_resets"
  | "group_resize_on_end_end_writes"
  | "group_resize_pinned_on_change_move_updates"
  | "group_resize_pinned_on_change_end_resets"
  | "group_resize_pinned_on_end_end_writes"
  | "sorting_fns_builtin_basic"
  | "sorting_fns_builtin_datetime"
  | "sorting_fns_builtin_text"
  | "sorting_fns_builtin_text_case_sensitive"
  | "sorting_fns_builtin_alphanumeric"
  | "sorting_fns_builtin_alphanumeric_case_sensitive"
  | "sorting_fns_auto_basic"
  | "sorting_fns_auto_datetime"
  | "sorting_fns_auto_text"
  | "sorting_fns_auto_alphanumeric"
  | "sorting_fns_registry_custom_text"
  | "sorting_fns_toggle_num_auto_first"
  | "sorting_fns_toggle_text_auto_first"
  | "filtering_fns_text_auto_includes"
  | "filtering_fns_text_equals_string"
  | "filtering_fns_num_in_number_range"
  | "filtering_fns_tags_arr_includes_all"
  | "filtering_fns_bool_equals"
  | "filtering_fns_weak_equals_string_number"
  | "filtering_fns_global_filter_includes"
  | "filtering_fns_global_filter_default_excludes_bool"
  | "filtering_fns_global_filter_disabled_when_enable_filters_false"
  | "filtering_fns_registry_custom_text_case_sensitive"
  | "filtering_fns_action_set_empty_removes"
  | "headers_cells_order_and_pinning"
  | "headers_cells_hide_right_leaf"
  | "headers_cells_hide_left_leaf"
  | "headers_cells_column_order_reorders"
  | "headers_cells_grouped_column_mode_reorder_moves_grouped_first"
  | "headers_cells_grouped_column_mode_remove_hides_grouped_column"
  | "headers_cells_grouped_column_mode_remove_drops_pinned_grouped_column"
  | "headers_cells_grouped_column_mode_reorder_respects_column_order_after_grouping"
  | "visord_baseline"
  | "visord_toggle_column_a_off"
  | "visord_toggle_all_off_keeps_non_hideable"
  | "visord_toggle_all_on_clears_state"
  | "visord_set_column_order_reorders"
  | "visord_set_column_order_with_duplicates"
  | "visord_set_order_then_hide"
  | "visord_toggle_noop_when_enable_hiding_false"
  | "state_shapes_baseline"
  | "state_shapes_grouping_two_columns"
  | "state_shapes_expanded_all"
  | "state_shapes_expanded_map"
  | "state_shapes_row_pinning_top_bottom"
  | "state_shapes_global_filter_json"
  | "selection_baseline"
  | "selection_state_two_rows"
  | "selection_filtered_selected_intersects"
  | "selection_toggle_row_multi_disabled_keeps_latest"
  | "selection_toggle_all_rows_disabled_noop"
  | "selection_toggle_all_page_rows_respects_pagination"
  | "selection_enable_row_selection_fn_odd_ids_toggle_all_rows_selects_selectable"
  | "selection_enable_row_selection_fn_odd_ids_toggle_row_unselectable_noop"
  | "faceting_baseline"
  | "faceting_cpu_own_filter_ignored"
  | "faceting_cpu_other_filter_applied"
  | "faceting_manual_filtering_bypasses"
  | "selection_enable_row_selection_fn_odd_ids_toggle_all_page_rows_selects_selectable"
  | "selection_tree_baseline"
  | "selection_tree_state_child_selected_marks_parent_some_selected"
  | "selection_tree_state_all_children_selected_marks_parent_all_sub_rows_selected"
  | "selection_tree_action_toggle_root_selects_children_default"
  | "selection_tree_action_toggle_root_select_children_false_only_root"
  | "selection_tree_action_toggle_root_enable_sub_row_selection_false_only_root"
  | "selection_tree_action_toggle_root_enable_multi_row_selection_false_clears_previous"
  | "selection_tree_action_toggle_on_row_selection_change_noop_ignores"
  | "selection_tree_enable_row_selection_fn_except_11_root_all_sub_rows_selected"
  | "selection_tree_enable_sub_row_selection_fn_disable_root_1_only_root"
  | "selection_tree_enable_multi_row_selection_fn_always_false_clears_previous"
  | "expanding_baseline"
  | "expanding_enable_expanding_false_disables_can_expand"
  | "expanding_hook_get_row_can_expand_overrides_enable_expanding_false"
  | "expanding_state_row_1"
  | "expanding_override_get_expanded_row_model_pre_expanded"
  | "expanding_hook_get_is_row_expanded_overrides_state"
  | "expanding_state_all_true"
  | "expanding_paginate_expanded_rows_true_counts_children"
  | "expanding_paginate_expanded_rows_false_expands_within_page"
  | "expanding_action_toggle_row"
  | "expanding_action_toggle_row_on_expanded_change_noop_ignores"
  | "expanding_action_toggle_row_enable_expanding_false_still_updates_state"
  | "expanding_action_toggle_all"
  | "row_id_state_ops_leaf_selection_prefixed"
  | "row_id_state_ops_group_selection"
  | "row_id_state_ops_group_selection_select_children_false"
  | "row_id_state_ops_group_selection_toggle_off"
  | "row_id_state_ops_nested_group_selection"
  | "row_id_state_ops_group_selection_on_row_selection_change_noop"
  | "row_id_state_ops_group_expanding"
  | "row_id_state_ops_group_expanding_on_expanded_change_noop"
  | "row_id_state_ops_group_pinning_on_row_pinning_change_noop"
  | "row_id_state_ops_group_pinning"
  | "row_id_state_ops_nested_group_pinning"
  | "row_id_state_ops_group_mixed_select_expand_pin"
  | "row_id_state_ops_nested_group_mixed_select_expand_pin"
  | "row_id_state_ops_group_mixed_selection_noop_expand_pin"
  | "pagination_baseline"
  | "pagination_set_page_index_out_of_range_uncontrolled"
  | "pagination_set_page_index_clamps_when_page_count_is_set"
  | "pagination_set_page_size_recomputes_page_index"
  | "pagination_manual_pagination_true_returns_pre_pagination"
  | "pagination_on_pagination_change_noop_ignores"
  | "pagination_page_count_minus_one_allows_next"
  | "pagination_row_count_infers_page_count"
  | "pagination_override_get_pagination_row_model_pre_pagination"
  | "pinning_keep_true_page_0"
  | "pinning_keep_false_page_0"
  | "pinning_keep_true_multi_pinned_index_page_0"
  | "pinning_keep_false_multi_pinned_index_page_0"
  | "pinning_keep_true_sorted_page_0"
  | "pinning_keep_false_sorted_page_0"
  | "pinning_keep_true_filter_excludes_pinned"
  | "pinning_keep_false_filter_excludes_pinned"
  | "pinning_enable_row_pinning_false_disables_can_pin"
  | "pinning_enable_pinning_false_disables_can_pin"
  | "pinning_enable_pinning_false_enable_row_pinning_true_overrides"
  | "pinning_enable_pinning_false_enable_row_pinning_fn_overrides"
  | "pinning_action_pin_top_bottom"
  | "pinning_action_unpin_top"
  | "pinning_action_on_row_pinning_change_noop_ignores"
  | "pinning_action_reset_row_pinning_restores_initial"
  | "pinning_action_reset_row_pinning_default_true_clears"
  | "pinning_tree_keep_true_child_hidden_when_parent_collapsed"
  | "pinning_tree_keep_true_child_visible_when_parent_expanded"
  | "pinning_tree_keep_false_never_surfaces_child_row"
  | "pinning_tree_action_pin_root_includes_leaf_rows"
  | "pinning_tree_action_pin_grandchild_includes_parent_rows"
  | "pinning_grouped_rows_baseline_page_0"
  | "pinning_grouped_rows_action_pin_group_role_1_top"
  | "pinning_grouped_rows_action_pin_group_role_1_top_include_leaf_rows"
  | "pinning_grouped_rows_action_pin_leaf_1_top_include_parent_rows"
  | "pinning_grouped_rows_state_page_1_pinned_role_1"
  | "column_pinning_default_can_pin"
  | "column_pinning_enable_column_pinning_false_disables_can_pin"
  | "column_pinning_enable_pinning_false_disables_can_pin"
  | "column_pinning_action_pin_left_right_unpin"
  | "column_pinning_action_pin_group_pins_leaf_columns"
  | "column_pinning_action_pins_when_enable_column_pinning_false"
  | "column_pinning_action_pins_when_enable_pinning_false"
  | "column_pinning_action_on_column_pinning_change_noop_ignores"
  | "column_pinning_action_reset_column_pinning_restores_initial"
  | "column_pinning_action_reset_column_pinning_default_true_clears"
  | "grouping_baseline"
  | "grouping_state_one_column"
  | "grouping_state_two_columns"
  | "grouping_manual_grouping_true_noops"
  | "grouping_enable_grouping_false_state_noops"
  | "grouping_override_get_grouped_row_model_pre_grouped"
  | "grouping_action_toggle_role_on"
  | "grouping_action_toggle_role_off"
  | "grouping_autoreset_expanded_default_resets"
  | "grouping_autoreset_expanded_manual_expanding_true_no_reset"
  | "grouping_autoreset_expanded_auto_reset_expanded_true_overrides_manual"
  | "grouping_autoreset_page_index_default_resets"
  | "grouping_autoreset_page_index_manual_pagination_true_no_reset"
  | "grouping_autoreset_page_index_auto_reset_page_index_true_overrides_manual"
  | "grouping_autoreset_page_index_auto_reset_all_false_disables"
  | "grouping_action_toggle_noop_when_enable_grouping_false"
  | "grouping_action_toggle_ignores_enable_grouping_false"
  | "grouping_state_one_column_sort_role_desc"
  | "grouping_state_one_column_sort_score_desc"
  | "grouping_state_two_columns_sort_score_desc"
  | "grouping_aggregation_fns_builtin_mix"
  | "grouping_aggregation_fns_custom_registry"
  | "render_fallback_baseline"

type DemoProcessRow = {
  id: number
  name: string
  status: string
  cpu: number
  mem_mb: number
  rank?: number
  subRows?: DemoProcessRow[]
}

type TanStackSorting = { id: string; desc: boolean }
type TanStackColumnFilter = { id: string; value: unknown }
type TanStackPagination = { pageIndex: number; pageSize: number }
type TanStackColumnPinning = { left?: string[]; right?: string[] }
type TanStackColumnSizing = Record<string, number>
type TanStackExpanded = true | Record<string, boolean>
type TanStackRowPinning = { top?: string[]; bottom?: string[] }
type TanStackColumnSizingInfo = {
  columnSizingStart: [string, number][]
  deltaOffset: null | number
  deltaPercentage: null | number
  isResizingColumn: false | string
  startOffset: null | number
  startSize: null | number
}

type TanStackState = {
  sorting?: TanStackSorting[]
  columnFilters?: TanStackColumnFilter[]
  globalFilter?: unknown
  pagination?: TanStackPagination
  grouping?: string[]
  expanded?: TanStackExpanded
  rowPinning?: TanStackRowPinning
  rowSelection?: Record<string, boolean>
  columnVisibility?: Record<string, boolean>
  columnSizing?: TanStackColumnSizing
  columnSizingInfo?: TanStackColumnSizingInfo
  columnPinning?: TanStackColumnPinning
  columnOrder?: string[]
}

type TanStackOptions = {
  initialState?: Partial<TanStackState>
  autoResetAll?: boolean
  manualFiltering?: boolean
  manualSorting?: boolean
  manualPagination?: boolean
  autoResetPageIndex?: boolean
  pageCount?: number
  rowCount?: number
  autoResetExpanded?: boolean
  manualExpanding?: boolean
  enableExpanding?: boolean
  manualGrouping?: boolean
  paginateExpandedRows?: boolean
  keepPinnedRows?: boolean
  enableFilters?: boolean
  enableColumnFilters?: boolean
  enableGlobalFilter?: boolean
  enableSorting?: boolean
  enableMultiSort?: boolean
  maxMultiSortColCount?: number
  enableSortingRemoval?: boolean
  enableMultiRemove?: boolean
  sortDescFirst?: boolean
  enableRowSelection?: boolean
  enableMultiRowSelection?: boolean
  enableSubRowSelection?: boolean
  enableColumnResizing?: boolean
  enableHiding?: boolean
  enableRowPinning?: boolean
  enableGrouping?: boolean
  enableColumnPinning?: boolean
  enablePinning?: boolean
  groupedColumnMode?: "reorder" | "remove" | false
  columnResizeMode?: "onChange" | "onEnd"
  columnResizeDirection?: "ltr" | "rtl"
  renderFallbackValue?: unknown
  // Fixture-only: when set, the generator injects a deterministic `options.aggregationFns` map.
  aggregationFnsMode?: "custom_plus_one"
  // Fixture-only: override `getGroupedRowModel` with a deterministic implementation.
  __getGroupedRowModel?: "pre_grouped"
  // Fixture-only: override `getRowCanExpand` with a deterministic implementation.
  __getRowCanExpand?: "only_root_1"
  // Fixture-only: override `getIsRowExpanded` with a deterministic implementation.
  __getIsRowExpanded?: "always_false"
  // Fixture-only: override `getExpandedRowModel` with a deterministic implementation.
  __getExpandedRowModel?: "pre_expanded"
  // Fixture-only: override `getPaginationRowModel` with a deterministic implementation.
  __getPaginationRowModel?: "pre_pagination"
  // Fixture-only: inject `enableRowPinning` as a deterministic per-row predicate.
  __enableRowPinning?: "odd_ids"
  // Fixture-only: inject `enableRowSelection` as a deterministic per-row predicate.
  __enableRowSelection?: "odd_ids" | "except_11" | "always_false"
  // Fixture-only: inject `enableSubRowSelection` as a deterministic per-row predicate.
  __enableSubRowSelection?: "disable_root_1" | "always_false"
  // Fixture-only: inject `enableMultiRowSelection` as a deterministic per-row predicate.
  __enableMultiRowSelection?: "always_false"
  // Fixture-only: when set, the generator injects a deterministic `options.sortingFns` map.
  sortingFnsMode?: "custom_text"
  // Fixture-only: when set, the generator injects a deterministic `options.filterFns` map.
  filterFnsMode?: "custom_text_case_sensitive"
  globalFilterFn?: "auto" | string
  // Fixture-only: simulate controlled state hooks that ignore the updater.
  __onColumnSizingChange?: "noop"
  __onColumnSizingInfoChange?: "noop"
  __onExpandedChange?: "noop"
  __onPaginationChange?: "noop"
  __onColumnVisibilityChange?: "noop"
  __onColumnFiltersChange?: "noop"
  __onGlobalFilterChange?: "noop"
  __onColumnPinningChange?: "noop"
  __onColumnOrderChange?: "noop"
  __onRowPinningChange?: "noop"
  __onRowSelectionChange?: "noop"
  // Fixture-only: choose a deterministic custom row id strategy.
  __getRowId?: "prefixed"
}

type RowModelSnapshot = { root: string[]; flat: string[] }

type RowSelectionDetail = {
  is_selected: Record<string, boolean>
  is_some_selected: Record<string, boolean>
  is_all_sub_rows_selected: Record<string, boolean>
  can_select: Record<string, boolean>
  can_multi_select: Record<string, boolean>
  can_select_sub_rows: Record<string, boolean>
}

type FilteringHelpersSnapshot = {
  columns: Record<
    string,
    {
      can_filter: boolean
      filter_value: unknown | null
      is_filtered: boolean
      filter_index: number
      can_global_filter: boolean
    }
  >
  global_filter: unknown | null
}

type SortingHelpersSnapshot = {
  columns: Record<
    string,
    {
      can_sort: boolean
      can_multi_sort: boolean
      is_sorted: "asc" | "desc" | null
      sort_index: number
      auto_sort_dir: "asc" | "desc" | null
      first_sort_dir: "asc" | "desc" | null
      next_sorting_order: "asc" | "desc" | null
      next_sorting_order_multi: "asc" | "desc" | null
    }
  >
}

type FixtureSnapshot = {
  id: SnapshotId
  options: TanStackOptions
  state: TanStackState
  actions?: FixtureAction[]
  expect: {
    core: RowModelSnapshot
    filtered: RowModelSnapshot
    sorted: RowModelSnapshot
    expanded?: RowModelSnapshot
    paginated: RowModelSnapshot
    row_model: RowModelSnapshot
    page_count?: number
    row_count?: number
    can_previous_page?: boolean
    can_next_page?: boolean
    page_options?: number[]
    selected?: RowModelSnapshot
    filtered_selected?: RowModelSnapshot
    grouped_selected?: RowModelSnapshot
    is_all_rows_selected?: boolean
    is_some_rows_selected?: boolean
    is_all_page_rows_selected?: boolean
    is_some_page_rows_selected?: boolean
    row_selection_detail?: RowSelectionDetail
    is_all_rows_expanded?: boolean
    is_some_rows_expanded?: boolean
    can_some_rows_expand?: boolean
    is_all_columns_visible?: boolean
    is_some_columns_visible?: boolean
    sorting_helpers?: SortingHelpersSnapshot
    filtering_helpers?: FilteringHelpersSnapshot
    headers_cells?: {
      header_groups: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      footer_groups?: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      left_header_groups: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      left_footer_groups?: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      center_header_groups: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      center_footer_groups?: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      right_header_groups: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      right_footer_groups?: {
        id: string
        depth: number
        headers: {
          id: string
          column_id: string
          depth: number
          index: number
          is_placeholder: boolean
          placeholder_id: string | null
          col_span: number
          row_span: number
          sub_header_ids: string[]
        }[]
      }[]
      flat_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      left_flat_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      center_flat_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      right_flat_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      leaf_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      left_leaf_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      center_leaf_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      right_leaf_headers?: {
        id: string
        column_id: string
        depth: number
        index: number
        is_placeholder: boolean
        placeholder_id: string | null
        col_span: number
        row_span: number
        sub_header_ids: string[]
      }[]
      cells: Record<
        string,
        {
          all: { id: string; column_id: string }[]
          visible: { id: string; column_id: string }[]
          left: { id: string; column_id: string }[]
          center: { id: string; column_id: string }[]
          right: { id: string; column_id: string }[]
        }
      >
    }
    core_model?: {
      column_tree: {
        id: string
        depth: number
        parent_id: string | null
        child_ids: string[]
      }[]
      flat_columns?: {
        all: string[]
        visible: string[]
      }
      leaf_columns: {
        all: string[]
        visible: string[]
        left_visible: string[]
        center_visible: string[]
        right_visible: string[]
      }
      header_groups: NonNullable<NonNullable<FixtureSnapshot["expect"]["headers_cells"]>["header_groups"]>
      left_header_groups: NonNullable<
        NonNullable<FixtureSnapshot["expect"]["headers_cells"]>["left_header_groups"]
      >
      center_header_groups: NonNullable<
        NonNullable<FixtureSnapshot["expect"]["headers_cells"]>["center_header_groups"]
      >
      right_header_groups: NonNullable<
        NonNullable<FixtureSnapshot["expect"]["headers_cells"]>["right_header_groups"]
      >
      rows: {
        core: RowModelSnapshot
        row_model: RowModelSnapshot
      }
      cells: NonNullable<NonNullable<FixtureSnapshot["expect"]["headers_cells"]>["cells"]>
    }
    flat_columns?: {
      all: string[]
      visible: string[]
    }
    column_sizing?: {
      total_size: number
      left_total_size: number
      center_total_size: number
      right_total_size: number
    }
    column_start?: {
      all: Record<string, number>
      left: Record<string, number | null>
      center: Record<string, number | null>
      right: Record<string, number | null>
    }
    column_after?: {
      all: Record<string, number>
      left: Record<string, number | null>
      center: Record<string, number | null>
      right: Record<string, number | null>
    }
    header_sizing?: {
      size: Record<string, number>
      start: Record<string, number>
    }
    row_pinning?: {
      top: string[]
      center: string[]
      bottom: string[]
      can_pin: Record<string, boolean>
      pin_position: Record<string, "top" | "bottom" | null>
      pinned_index: Record<string, number>
      is_some_rows_pinned: boolean
      is_some_top_rows_pinned: boolean
      is_some_bottom_rows_pinned: boolean
    }
    grouped_row_model?: {
      root: {
        kind: "group" | "leaf"
        depth: number
        path: { column_id: string; value: unknown }[]
        grouping_column_id?: string
        grouping_value?: unknown
        leaf_row_count?: number
        first_leaf_row_id?: string | null
        row_id?: string
      }[]
      flat: {
        kind: "group" | "leaf"
        depth: number
        path: { column_id: string; value: unknown }[]
        grouping_column_id?: string
        grouping_value?: unknown
        leaf_row_count?: number
        first_leaf_row_id?: string | null
        row_id?: string
      }[]
    }
    sorted_grouped_row_model?: {
      root: {
        kind: "group" | "leaf"
        depth: number
        path: { column_id: string; value: unknown }[]
        grouping_column_id?: string
        grouping_value?: unknown
        leaf_row_count?: number
        first_leaf_row_id?: string | null
        row_id?: string
      }[]
      flat: {
        kind: "group" | "leaf"
        depth: number
        path: { column_id: string; value: unknown }[]
        grouping_column_id?: string
        grouping_value?: unknown
        leaf_row_count?: number
        first_leaf_row_id?: string | null
        row_id?: string
      }[]
    }
    grouped_aggregations_u64?: {
      path: { column_id: string; value: unknown }[]
      values: Record<string, number | null>
    }[]
    grouped_aggregations_any?: {
      path: { column_id: string; value: unknown }[]
      values: Record<string, unknown>
    }[]
    render_fallback?: {
      row_id: string
      column_id: string
      value: unknown
      render_value: unknown
    }[]
    column_pinning?: {
      left: string[]
      center: string[]
      right: string[]
      can_pin: Record<string, boolean>
      pin_position: Record<string, "left" | "right" | null>
      pinned_index: Record<string, number>
      is_some_columns_pinned: boolean
      is_some_left_columns_pinned: boolean
      is_some_right_columns_pinned: boolean
    }
    faceting?: {
      cpu: {
        row_model: RowModelSnapshot
        unique_values: Record<string, number>
        min_max: [number, number] | null
      }
      global: {
        row_model: RowModelSnapshot
        unique_values: Record<string, number>
        min_max: [number, number] | null
      }
    }
    next_state?: TanStackState
  }
}

type Fixture = {
  upstream: {
    package: string
    version: string
    commit?: string
    commit_short?: string
    source: string
  }
  case_id: CaseId
  data: unknown[]
  columns_meta?: unknown
  snapshots: FixtureSnapshot[]
}

type FixtureAction =
  | {
      type: "toggleSorting"
      column_id: string
      multi?: boolean
    }
  | {
      type: "toggleSortingHandler"
      column_id: string
      event_multi?: boolean
    }
  | {
      type: "clearSorting"
      column_id: string
    }
  | {
      type: "setColumnFilterValue"
      column_id: string
      value: unknown
    }
  | {
      type: "setGlobalFilterValue"
      value: unknown
    }
  | {
      type: "toggleColumnVisibility"
      column_id: string
      value?: boolean
    }
  | {
      type: "toggleAllColumnsVisible"
      value?: boolean
    }
  | {
      type: "setColumnOrder"
      order: string[]
    }
  | {
      type: "pinRow"
      row_id: string
      position: "top" | "bottom" | null
      include_leaf_rows?: boolean
      include_parent_rows?: boolean
    }
  | {
      type: "pinColumn"
      column_id: string
      position: "left" | "right" | null
    }
  | {
      type: "resetRowPinning"
      default_state?: boolean
    }
  | {
      type: "resetColumnPinning"
      default_state?: boolean
    }
  | {
      type: "toggleRowSelected"
      row_id: string
      value?: boolean
      select_children?: boolean
    }
  | {
      type: "toggleAllRowsSelected"
      value?: boolean
    }
  | {
      type: "toggleAllPageRowsSelected"
      value?: boolean
    }
  | {
      type: "resetRowSelection"
      default_state?: boolean
    }
  | {
      type: "toggleRowExpanded"
      row_id: string
      value?: boolean
    }
  | {
      type: "toggleAllRowsExpanded"
      value?: boolean
    }
  | {
      type: "setPageIndex"
      page_index: number
    }
  | {
      type: "setPageSize"
      page_size: number
    }
  | {
      type: "nextPage"
    }
  | {
      type: "previousPage"
    }
  | {
      type: "firstPage"
    }
  | {
      type: "lastPage"
    }
  | {
      type: "resetPageIndex"
      default_state?: boolean
    }
  | {
      type: "resetPageSize"
      default_state?: boolean
    }
  | {
      type: "resetPagination"
      default_state?: boolean
    }
  | {
      type: "resetSorting"
      default_state?: boolean
    }
  | {
      type: "resetColumnFilters"
      default_state?: boolean
    }
  | {
      type: "resetGlobalFilter"
      default_state?: boolean
    }
  | {
      type: "resetGrouping"
      default_state?: boolean
    }
  | {
      type: "resetColumnVisibility"
      default_state?: boolean
    }
  | {
      type: "resetColumnOrder"
      default_state?: boolean
    }
  | {
      type: "toggleGrouping"
      column_id: string
      value?: boolean
    }
  | {
      type: "toggleGroupingHandler"
      column_id: string
    }
  | {
      type: "setGrouping"
      grouping: string[]
    }
  | {
      type: "columnResizeBegin"
      column_id: string
      client_x: number
    }
  | {
      type: "columnResizeMove"
      client_x: number
    }
  | {
      type: "columnResizeEnd"
      client_x: number
    }
  | {
      type: "resetColumnSize"
      column_id: string
    }
  | {
      type: "resetColumnSizing"
      default_state?: boolean
    }
  | {
      type: "resetHeaderSizeInfo"
      default_state?: boolean
    }

function parseArgs(argv: string[]): { out: string; case_id: CaseId } {
  let out: string | undefined
  let case_id: CaseId = "demo_process"
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === "--out") {
      out = argv[i + 1]
      i++
      continue
    }
    if (a === "--case") {
      const v = argv[i + 1]
      if (
        v !== "demo_process" &&
        v !== "auto_reset" &&
        v !== "resets" &&
        v !== "pagination" &&
        v !== "sort_undefined" &&
        v !== "sorting_fns" &&
        v !== "filtering_fns" &&
        v !== "headers_cells" &&
        v !== "headers_inventory_deep" &&
        v !== "visibility_ordering" &&
        v !== "pinning" &&
        v !== "pinning_tree" &&
        v !== "pinning_grouped_rows" &&
        v !== "column_pinning" &&
        v !== "faceting" &&
        v !== "column_sizing" &&
        v !== "column_resizing_group_headers" &&
        v !== "state_shapes" &&
        v !== "selection" &&
        v !== "selection_tree" &&
        v !== "expanding" &&
        v !== "grouping" &&
        v !== "grouping_aggregation_fns" &&
        v !== "row_id_state_ops" &&
        v !== "render_fallback"
      ) {
        throw new Error(`unknown --case ${v}`)
      }
      case_id = v
      i++
      continue
    }
  }
  if (!out) {
    throw new Error(
      "usage: node extract-fixtures.mts --out <path> [--case demo_process|auto_reset|resets|pagination|sort_undefined|sorting_fns|filtering_fns|headers_cells|headers_inventory_deep|visibility_ordering|pinning|pinning_tree|pinning_grouped_rows|column_pinning|faceting|column_sizing|column_resizing_group_headers|state_shapes|selection|selection_tree|expanding|grouping|grouping_aggregation_fns|row_id_state_ops|render_fallback]",
    )
  }
  return { out, case_id }
}

const JSON_UNDEFINED = { __fret: "undefined" }

function jsonSafe(value: any): any {
  if (value === undefined) {
    return JSON_UNDEFINED
  }
  if (Array.isArray(value)) {
    return value.map(jsonSafe)
  }
  return value
}

function fileExists(p: string): boolean {
  try {
    fs.accessSync(p, fs.constants.F_OK)
    return true
  } catch {
    return false
  }
}

function resolveRepoRefTableRoot(): string {
  const explicit = process.env.FRET_REPO_REF_TABLE
  if (explicit && fileExists(explicit)) {
    return explicit
  }
  const root = process.env.FRET_REPO_REF_ROOT
  if (root) {
    const candidate = path.join(root, "table")
    if (fileExists(candidate)) {
      return candidate
    }
  }

  const cwd = process.cwd()
  const parent = path.dirname(cwd)
  if (path.basename(parent) === "fret-worktrees") {
    const sibling = path.join(path.dirname(parent), "fret", "repo-ref", "table")
    if (fileExists(sibling)) {
      return sibling
    }
  }

  const candidate = path.join(cwd, "repo-ref", "table")
  if (fileExists(candidate)) {
    return candidate
  }

  throw new Error(
    "Cannot resolve repo-ref/table. Set FRET_REPO_REF_TABLE or FRET_REPO_REF_ROOT.",
  )
}

function snapshotRowModel(model: any): RowModelSnapshot {
  const root = (model.rows ?? []).map((r: any) => String(r.id))
  const flat = (model.flatRows ?? []).map((r: any) => String(r.id))
  return { root, flat }
}

function snapshotRowSelectionDetail(table: any, rowIds: string[]): RowSelectionDetail {
  const out: RowSelectionDetail = {
    is_selected: {},
    is_some_selected: {},
    is_all_sub_rows_selected: {},
    can_select: {},
    can_multi_select: {},
    can_select_sub_rows: {},
  }

  for (const id of rowIds) {
    const row = table.getRow?.(id, true)
    if (!row) {
      out.is_selected[id] = false
      out.is_some_selected[id] = false
      out.is_all_sub_rows_selected[id] = false
      out.can_select[id] = false
      out.can_multi_select[id] = false
      out.can_select_sub_rows[id] = false
      continue
    }
    out.is_selected[id] = Boolean(row.getIsSelected?.())
    out.is_some_selected[id] = Boolean(row.getIsSomeSelected?.())
    out.is_all_sub_rows_selected[id] = Boolean(row.getIsAllSubRowsSelected?.())
    out.can_select[id] = Boolean(row.getCanSelect?.())
    out.can_multi_select[id] = Boolean(row.getCanMultiSelect?.())
    out.can_select_sub_rows[id] = Boolean(row.getCanSelectSubRows?.())
  }

  return out
}

function getGitHeadCommit(dir: string): { commit?: string; commit_short?: string } {
  try {
    const commit = String(execSync("git rev-parse HEAD", { cwd: dir })).trim()
    const commit_short = String(execSync("git rev-parse --short HEAD", { cwd: dir })).trim()
    return { commit, commit_short }
  } catch {
    return {}
  }
}

function sortString(a: any, b: any, id: string): number {
  const av = String(a.getValue(id))
  const bv = String(b.getValue(id))
  return av.localeCompare(bv)
}

function sortNumber(a: any, b: any, id: string): number {
  const ra = a.getValue(id)
  const rb = b.getValue(id)
  if (ra === undefined || rb === undefined) {
    return 0
  }
  const av = Number(ra)
  const bv = Number(rb)
  return av === bv ? 0 : av < bv ? -1 : 1
}

function compareBasic(a: any, b: any): number {
  return a === b ? 0 : a > b ? 1 : -1
}

function toTextKey(a: any): string {
  if (typeof a === "number") {
    if (Number.isNaN(a) || a === Infinity || a === -Infinity) {
      return ""
    }
    return String(a)
  }
  if (typeof a === "string") {
    return a
  }
  return ""
}

function sortTextBuiltin(a: any, b: any, id: string): number {
  const av = toTextKey(a.getValue(id)).toLowerCase()
  const bv = toTextKey(b.getValue(id)).toLowerCase()
  return compareBasic(av, bv)
}

class FakeDocument {
  private listeners: Map<string, Set<(e: any) => void>> = new Map()

  addEventListener(type: string, handler: (e: any) => void, _opts?: any): void {
    const set = this.listeners.get(type) ?? new Set()
    set.add(handler)
    this.listeners.set(type, set)
  }

  removeEventListener(type: string, handler: (e: any) => void, _opts?: any): void {
    const set = this.listeners.get(type)
    if (!set) {
      return
    }
    set.delete(handler)
  }

  dispatch(type: string, event: any): void {
    const set = this.listeners.get(type)
    if (!set) {
      return
    }
    for (const handler of [...set]) {
      handler(event)
    }
  }
}

function filterContainsAsciiCI(row: any, id: string, value: any): boolean {
  const needle = String(value ?? "").toLowerCase()
  if (!needle) {
    return true
  }
  const hay = String(row.getValue(id) ?? "").toLowerCase()
  return hay.includes(needle)
}

function filterContainsAsciiCS(row: any, id: string, value: any): boolean {
  const needle = String(value ?? "")
  if (!needle) {
    return true
  }
  const hay = String(row.getValue(id) ?? "")
  return hay.includes(needle)
}

// Mimic TanStack built-in `testFalsey` auto-remove rules for string-like filters.
;(filterContainsAsciiCS as any).autoRemove = (val: any) =>
  val === undefined || val === null || val === ""

async function main(): Promise<void> {
  const { out, case_id } = parseArgs(process.argv.slice(2))

  const repoRefTable = resolveRepoRefTableRoot()
  const tableCoreRoot = path.join(repoRefTable, "packages", "table-core")
  const tableCoreBuild = path.join(tableCoreRoot, "build", "lib", "index.js")
  if (!fileExists(tableCoreBuild)) {
    throw new Error(
      [
        `Missing built table-core entry: ${tableCoreBuild}`,
        "Build the upstream package first:",
        `  pnpm -C ${repoRefTable} install --frozen-lockfile`,
        `  pnpm -C ${repoRefTable} -F @tanstack/table-core build`,
      ].join("\n"),
    )
  }

  const tableCorePkgJson = JSON.parse(
    fs.readFileSync(path.join(tableCoreRoot, "package.json"), "utf8"),
  ) as { name: string; version: string }
  const tableRepoCommit = getGitHeadCommit(repoRefTable)

  const require = createRequire(import.meta.url)
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const tableCore = require(tableCoreBuild)

  let data: unknown[]
  let columns: any[]
  let columns_meta: unknown | undefined

  if (
    case_id === "demo_process" ||
    case_id === "auto_reset" ||
    case_id === "resets" ||
    case_id === "pagination" ||
    case_id === "state_shapes" ||
    case_id === "selection" ||
    case_id === "pinning" ||
    case_id === "row_id_state_ops"
  ) {
    const demo: DemoProcessRow[] = [
      { id: 1, name: "Renderer", status: "Running", cpu: 12, mem_mb: 420 },
      { id: 2, name: "Asset Cache", status: "Idle", cpu: 0, mem_mb: 128 },
      { id: 3, name: "Indexer", status: "Running", cpu: 38, mem_mb: 860 },
      { id: 4, name: "Spellcheck", status: "Disabled", cpu: 0, mem_mb: 0 },
      { id: 5, name: "Language Server", status: "Running", cpu: 7, mem_mb: 512 },
    ]

    data = demo

    columns = [
      {
        id: "name",
        accessorFn: (row: DemoProcessRow) => row.name,
        sortingFn: sortString,
        filterFn: filterContainsAsciiCI,
      },
      {
        id: "status",
        accessorFn: (row: DemoProcessRow) => row.status,
        sortingFn: sortString,
        filterFn: filterContainsAsciiCI,
      },
      {
        id: "cpu",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
      },
      {
        id: "cpu_desc_first",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
        sortDescFirst: true,
      },
      {
        id: "cpu_no_multi",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
        enableMultiSort: false,
      },
      {
        id: "cpu_no_sort",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
        enableSorting: false,
      },
      {
        id: "cpu_invert",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
        invertSorting: true,
      },
      {
        id: "mem_mb",
        accessorFn: (row: DemoProcessRow) => row.mem_mb,
        sortingFn: sortNumber,
      },
    ]
  } else if (case_id === "faceting") {
    const demo: DemoProcessRow[] = [
      { id: 1, name: "Renderer", status: "Running", cpu: 12, mem_mb: 420 },
      { id: 2, name: "Asset Cache", status: "Idle", cpu: 0, mem_mb: 128 },
      { id: 3, name: "Indexer", status: "Running", cpu: 38, mem_mb: 860 },
      { id: 4, name: "Spellcheck", status: "Disabled", cpu: 0, mem_mb: 0 },
      { id: 5, name: "Language Server", status: "Running", cpu: 7, mem_mb: 512 },
    ]

    data = demo

    columns = [
      {
        id: "status",
        accessorFn: (row: DemoProcessRow) => row.status,
        filterFn: filterContainsAsciiCI,
      },
      {
        id: "cpu",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        filterFn: "inNumberRange",
      },
    ]
  } else if (case_id === "grouping") {
    const rows: { id: number; role: number; team: number; score: number }[] = [
      { id: 1, role: 1, team: 10, score: 5 },
      { id: 2, role: 2, team: 20, score: 7 },
      { id: 3, role: 1, team: 20, score: 1 },
      { id: 4, role: 2, team: 10, score: 3 },
      { id: 5, role: 1, team: 10, score: 2 },
    ]
    data = rows
    columns = [
      { id: "role", accessorFn: (row: any) => row.role },
      { id: "team", accessorFn: (row: any) => row.team },
      { id: "score", accessorFn: (row: any) => row.score },
    ]
  } else if (case_id === "pinning_grouped_rows") {
    const rows: { id: number; role: number; team: number; score: number }[] = [
      { id: 1, role: 1, team: 10, score: 5 },
      { id: 2, role: 2, team: 20, score: 7 },
      { id: 3, role: 1, team: 20, score: 1 },
      { id: 4, role: 2, team: 10, score: 3 },
      { id: 5, role: 1, team: 10, score: 2 },
    ]
    data = rows
    columns = [
      { id: "role", accessorFn: (row: any) => row.role },
      { id: "team", accessorFn: (row: any) => row.team },
      { id: "score", accessorFn: (row: any) => row.score },
    ]
  } else if (case_id === "grouping_aggregation_fns") {
    const rows: any[] = [
      { id: 1, role: 1, team: "x", score: 5, tag: "alpha" },
      { id: 2, role: 2, team: "y", score: 7, tag: "beta" },
      { id: 3, role: 1, team: "y", score: 1, tag: "alpha" },
      { id: 4, role: 2, team: "x", score: 3, tag: null },
      { id: 5, role: 1, team: "x", score: 2 },
    ]
    data = rows
    columns = [
      { id: "role", accessorFn: (row: any) => row.role },
      { id: "team", accessorFn: (row: any) => row.team },
      { id: "score_sum", accessorFn: (row: any) => row.score, aggregationFn: "sum" },
      { id: "score_min", accessorFn: (row: any) => row.score, aggregationFn: "min" },
      { id: "score_max", accessorFn: (row: any) => row.score, aggregationFn: "max" },
      { id: "score_extent", accessorFn: (row: any) => row.score, aggregationFn: "extent" },
      { id: "score_mean", accessorFn: (row: any) => row.score, aggregationFn: "mean" },
      { id: "score_median", accessorFn: (row: any) => row.score, aggregationFn: "median" },
      { id: "tag_unique", accessorFn: (row: any) => row.tag, aggregationFn: "unique" },
      { id: "tag_unique_count", accessorFn: (row: any) => row.tag, aggregationFn: "uniqueCount" },
      { id: "tag_count", accessorFn: (row: any) => row.tag, aggregationFn: "count" },
      {
        id: "score_custom",
        accessorFn: (row: any) => row.score,
        aggregationFn: "custom_plus_one",
      },
    ]
  } else if (case_id === "render_fallback") {
    const rows: any[] = [
      { id: 1 },
      { id: 2, value: null },
      { id: 3, value: 0 },
      { id: 4, value: "" },
      { id: 5, value: false },
    ]
    data = rows
    columns = [{ id: "value", accessorFn: (row: any) => row.value }]
  } else if (case_id === "expanding" || case_id === "pinning_tree" || case_id === "selection_tree") {
    const tree: DemoProcessRow[] = [
      {
        id: 1,
        name: "Root 1",
        status: "Running",
        cpu: 1,
        mem_mb: 100,
        subRows: [
          { id: 11, name: "Child 11", status: "Running", cpu: 2, mem_mb: 10 },
          {
            id: 12,
            name: "Child 12",
            status: "Idle",
            cpu: 3,
            mem_mb: 20,
            subRows: [
              {
                id: 121,
                name: "Grandchild 121",
                status: "Running",
                cpu: 4,
                mem_mb: 5,
              },
            ],
          },
        ],
      },
      { id: 2, name: "Root 2", status: "Idle", cpu: 0, mem_mb: 200 },
      {
        id: 3,
        name: "Root 3",
        status: "Running",
        cpu: 5,
        mem_mb: 300,
        subRows: [{ id: 31, name: "Child 31", status: "Running", cpu: 6, mem_mb: 30 }],
      },
      { id: 4, name: "Root 4", status: "Disabled", cpu: 0, mem_mb: 0 },
    ]

    data = tree

    columns = [
      {
        id: "name",
        accessorFn: (row: DemoProcessRow) => row.name,
        sortingFn: sortString,
        filterFn: filterContainsAsciiCI,
      },
      {
        id: "status",
        accessorFn: (row: DemoProcessRow) => row.status,
        sortingFn: sortString,
        filterFn: filterContainsAsciiCI,
      },
      {
        id: "cpu",
        accessorFn: (row: DemoProcessRow) => row.cpu,
        sortingFn: sortNumber,
      },
      {
        id: "mem_mb",
        accessorFn: (row: DemoProcessRow) => row.mem_mb,
        sortingFn: sortNumber,
      },
    ]
  } else if (case_id === "sort_undefined") {
    const sortUndefinedRows: DemoProcessRow[] = [
      { id: 1, name: "A", status: "ok", cpu: 0, mem_mb: 0, rank: 3 },
      { id: 2, name: "B", status: "ok", cpu: 0, mem_mb: 0, rank: 1 },
      { id: 3, name: "C", status: "ok", cpu: 0, mem_mb: 0 },
      { id: 4, name: "D", status: "ok", cpu: 0, mem_mb: 0, rank: 2 },
      { id: 5, name: "E", status: "ok", cpu: 0, mem_mb: 0, rank: 4 },
    ]

    data = sortUndefinedRows

    columns = [
      {
        id: "rank_first",
        accessorFn: (row: DemoProcessRow) => row.rank,
        sortingFn: sortNumber,
        sortUndefined: "first",
      },
      {
        id: "rank_last",
        accessorFn: (row: DemoProcessRow) => row.rank,
        sortingFn: sortNumber,
        sortUndefined: "last",
      },
      {
        id: "rank_1",
        accessorFn: (row: DemoProcessRow) => row.rank,
        sortingFn: sortNumber,
        sortUndefined: 1,
      },
      {
        id: "rank_neg1",
        accessorFn: (row: DemoProcessRow) => row.rank,
        sortingFn: sortNumber,
        sortUndefined: -1,
      },
      {
        id: "rank_false_text",
        accessorFn: (row: DemoProcessRow) => row.rank,
        sortingFn: "text",
        sortUndefined: false,
      },
    ]
  } else if (case_id === "sorting_fns") {
    const rows = Array.from({ length: 12 }, (_, i) => {
      const id = i + 1
      if (id <= 10) {
        return {
          id,
          text: "zzz",
          alpha: "a0",
          num: id,
          dt_ms: undefined as undefined | number,
        }
      }
      if (id === 11) {
        return {
          id,
          text: "apple",
          alpha: "a2",
          num: id,
          dt_ms: 1700000000000,
        }
      }
      return {
        id,
        text: "Banana",
        alpha: "A10",
        num: id,
        dt_ms: 1600000000000,
      }
    })

    data = rows

    columns = [
      {
        id: "num_basic",
        accessorFn: (row: any) => row.num,
        sortingFn: "basic",
      },
      {
        id: "num_auto",
        accessorFn: (row: any) => row.num,
        sortingFn: "auto",
      },
      {
        id: "text_text",
        accessorFn: (row: any) => row.text,
        sortingFn: "text",
      },
      {
        id: "text_text_cs",
        accessorFn: (row: any) => row.text,
        sortingFn: "textCaseSensitive",
      },
      {
        id: "text_auto",
        accessorFn: (row: any) => row.text,
        sortingFn: "auto",
      },
      {
        id: "alpha_alphanumeric",
        accessorFn: (row: any) => row.alpha,
        sortingFn: "alphanumeric",
      },
      {
        id: "alpha_alphanumeric_cs",
        accessorFn: (row: any) => row.alpha,
        sortingFn: "alphanumericCaseSensitive",
      },
      {
        id: "alpha_auto",
        accessorFn: (row: any) => row.alpha,
        sortingFn: "auto",
      },
      {
        id: "dt_datetime",
        accessorFn: (row: any) =>
          row.dt_ms === undefined ? undefined : new Date(row.dt_ms),
        sortingFn: "datetime",
      },
      {
        id: "dt_auto",
        accessorFn: (row: any) =>
          row.dt_ms === undefined ? undefined : new Date(row.dt_ms),
        sortingFn: "auto",
      },
      {
        id: "text_custom",
        accessorFn: (row: any) => row.text,
        sortingFn: "custom_text",
      },
    ]
  } else if (case_id === "filtering_fns") {
    const rows = [
      { id: 1, text: "apple", num: 3, flag: true, tags: ["a", "b"] as string[] },
      { id: 2, text: "Banana", num: 5, flag: false, tags: ["b"] as string[] },
      { id: 3, text: "carrot", num: 10, flag: true, tags: [] as string[] },
      { id: 4, text: null as null | string, num: 0, flag: false, tags: ["x", "y"] as string[] },
      { id: 5, text: "apricot", num: 7, flag: true, tags: ["a"] as string[] },
    ]

    data = rows

    columns = [
      {
        id: "text_auto",
        accessorFn: (row: any) => row.text,
        filterFn: "auto",
      },
      {
        id: "text_equals_string",
        accessorFn: (row: any) => row.text,
        filterFn: "equalsString",
      },
      {
        id: "num_range",
        accessorFn: (row: any) => row.num,
        filterFn: "inNumberRange",
      },
      {
        id: "num_weak",
        accessorFn: (row: any) => row.num,
        filterFn: "weakEquals",
      },
      {
        id: "tags_all",
        accessorFn: (row: any) => row.tags,
        filterFn: "arrIncludesAll",
      },
      {
        id: "flag_equals",
        accessorFn: (row: any) => row.flag,
        filterFn: "equals",
      },
      {
        id: "text_custom",
        accessorFn: (row: any) => row.text,
        filterFn: "custom_text",
      },
    ]
  } else if (case_id === "headers_cells") {
    const rows: DemoProcessRow[] = [
      { id: 1, name: "Renderer", status: "Running", cpu: 12, mem_mb: 420 },
      { id: 2, name: "Asset Cache", status: "Idle", cpu: 0, mem_mb: 128 },
      { id: 3, name: "Indexer", status: "Running", cpu: 38, mem_mb: 860 },
    ]

    data = rows

    columns = [
      {
        id: "name",
        accessorFn: (row: DemoProcessRow) => row.name,
      },
      {
        id: "stats",
        columns: [
          {
            id: "perf",
            columns: [
              {
                id: "cpu",
                accessorFn: (row: DemoProcessRow) => row.cpu,
              },
            ],
          },
          {
            id: "mem",
            columns: [
              {
                id: "mem_mb",
                accessorFn: (row: DemoProcessRow) => row.mem_mb,
              },
            ],
          },
        ],
      },
    ]
  } else if (case_id === "headers_inventory_deep") {
    const rows: DemoProcessRow[] = [
      { id: 1, name: "Renderer", status: "Running", cpu: 12, mem_mb: 420 },
      { id: 2, name: "Asset Cache", status: "Idle", cpu: 0, mem_mb: 128 },
      { id: 3, name: "Indexer", status: "Running", cpu: 38, mem_mb: 860 },
    ]

    data = rows

    // Deeper column nesting to exercise placeholder generation and leaf/flat traversal.
    columns = [
      {
        id: "name",
        accessorFn: (row: DemoProcessRow) => row.name,
      },
      {
        id: "metrics",
        columns: [
          {
            id: "stats",
            columns: [
              {
                id: "perf",
                columns: [
                  {
                    id: "cpu",
                    accessorFn: (row: DemoProcessRow) => row.cpu,
                  },
                  {
                    id: "cpu2",
                    accessorFn: (row: DemoProcessRow) => row.cpu,
                  },
                ],
              },
              {
                id: "mem",
                columns: [
                  {
                    id: "mem_mb",
                    accessorFn: (row: DemoProcessRow) => row.mem_mb,
                  },
                ],
              },
            ],
          },
          {
            id: "status",
            accessorFn: (row: DemoProcessRow) => row.status,
          },
        ],
      },
    ]
  } else if (case_id === "column_resizing_group_headers") {
    // Column sizing + resizing interactions against a grouped header that fans out to multiple leaf columns.
    // We keep a 1-row dataset so table-core initializes consistently.
    const rows: DemoProcessRow[] = [{ id: 1, name: "x", status: "x", cpu: 0, mem_mb: 0 }]
    data = rows

    const sizingColumns = [
      { id: "a", size: 100, minSize: 20, maxSize: 300, enablePinning: true },
      { id: "b", size: 50, enablePinning: false },
      { id: "c", size: 25, enablePinning: true },
    ]
    columns_meta = sizingColumns

    const leaf = (c: any) => ({
      id: c.id,
      accessorFn: (row: DemoProcessRow) => row.id,
      size: c.size,
      minSize: c.minSize,
      maxSize: c.maxSize,
      enableResizing: true,
    })

    columns = [
      {
        id: "ab",
        columns: [leaf(sizingColumns[0]), leaf(sizingColumns[1])],
      },
      leaf(sizingColumns[2]),
    ]
  } else if (case_id === "visibility_ordering") {
    // Column visibility + ordering state transitions.
    // We keep a 1-row dataset so table-core initializes consistently.
    const rows: DemoProcessRow[] = [{ id: 1, name: "x", status: "x", cpu: 0, mem_mb: 0 }]
    data = rows

    const visColumns = [
      { id: "a", size: 100, minSize: 20, maxSize: 300, enableHiding: true },
      { id: "b", size: 50, enableHiding: false },
      { id: "c", size: 25, enableHiding: true },
    ]
    columns_meta = visColumns.map(({ id, size, minSize, maxSize }) => ({
      id,
      size,
      minSize,
      maxSize,
    }))

    columns = visColumns.map((c) => ({
      id: c.id,
      accessorFn: (row: DemoProcessRow) => row.id,
      size: c.size,
      minSize: c.minSize,
      maxSize: c.maxSize,
      enableHiding: c.enableHiding,
    }))
  } else if (case_id === "column_pinning") {
    // Column pinning against a grouped column that pins multiple leaf columns at once.
    // We keep a 1-row dataset so table-core initializes consistently.
    const rows: DemoProcessRow[] = [{ id: 1, name: "x", status: "x", cpu: 0, mem_mb: 0 }]
    data = rows

    const pinColumns = [
      { id: "a", size: 100, minSize: 20, maxSize: 300, enablePinning: true },
      { id: "b", size: 50, enablePinning: false },
      { id: "c", size: 25, enablePinning: true },
    ]
    columns_meta = pinColumns.map(({ id, size, minSize, maxSize }) => ({
      id,
      size,
      minSize,
      maxSize,
    }))

    const leaf = (c: any) => ({
      id: c.id,
      accessorFn: (row: DemoProcessRow) => row.id,
      size: c.size,
      minSize: c.minSize,
      maxSize: c.maxSize,
      enableResizing: true,
      enablePinning: c.enablePinning,
    })

    columns = [
      {
        id: "ab",
        columns: [leaf(pinColumns[0]), leaf(pinColumns[1])],
      },
      leaf(pinColumns[2]),
    ]
  } else {
    // Column sizing / pinning / ordering outputs (no row model expectations needed).
    // We keep a 1-row dataset so table-core initializes consistently.
    const rows: DemoProcessRow[] = [{ id: 1, name: "x", status: "x", cpu: 0, mem_mb: 0 }]
    data = rows

    const sizingColumns = [
      { id: "a", size: 100, minSize: 20, maxSize: 300 },
      { id: "b", size: 50 },
      { id: "c", size: 25 },
    ]
    columns_meta = sizingColumns

    columns = sizingColumns.map((c) => ({
      id: c.id,
      accessorFn: (row: DemoProcessRow) => row.id,
      size: c.size,
      minSize: c.minSize,
      maxSize: c.maxSize,
      enableResizing: true,
      enablePinning: c.id !== "b",
    }))
  }

  function buildTable(options: TanStackOptions, state: TanStackState) {
    const currentState = {
      sorting: [],
      columnFilters: [],
      globalFilter: undefined,
      pagination: { pageIndex: 0, pageSize: 10 },
      grouping: [],
      expanded: {},
      rowPinning: { top: [], bottom: [] },
      rowSelection: {},
      columnVisibility: {},
      columnSizing: {},
      columnSizingInfo: {
        startOffset: null,
        startSize: null,
        deltaOffset: null,
        deltaPercentage: null,
        isResizingColumn: false,
        columnSizingStart: [],
      },
      columnPinning: { left: [], right: [] },
      columnOrder: [],
      ...state,
    }

    const getRowId =
      options.__getRowId === "prefixed"
        ? (row: DemoProcessRow) => `row:${String(row.id)}`
        : (row: DemoProcessRow) => String(row.id)

    const table = tableCore.createTable<DemoProcessRow>({
    data,
    columns,
    getRowId,
    getSubRows: (row: DemoProcessRow) => (row as any).subRows,
      initialState: options.initialState,
      autoResetAll: options.autoResetAll,
      manualFiltering: options.manualFiltering ?? false,
      manualSorting: options.manualSorting ?? false,
      manualPagination: options.manualPagination ?? false,
      autoResetPageIndex: options.autoResetPageIndex,
      pageCount: options.pageCount,
      rowCount: options.rowCount,
      autoResetExpanded: options.autoResetExpanded,
      manualExpanding: options.manualExpanding ?? false,
      enableExpanding: options.enableExpanding,
      manualGrouping: options.manualGrouping ?? false,
      paginateExpandedRows: options.paginateExpandedRows ?? true,
      keepPinnedRows: options.keepPinnedRows ?? true,
      enableFilters: options.enableFilters ?? true,
      enableColumnFilters: options.enableColumnFilters ?? true,
      enableGlobalFilter: options.enableGlobalFilter ?? true,
      enableSorting: options.enableSorting ?? true,
      enableMultiSort: options.enableMultiSort ?? true,
      maxMultiSortColCount: options.maxMultiSortColCount,
      enableSortingRemoval: options.enableSortingRemoval ?? true,
      enableMultiRemove: options.enableMultiRemove ?? true,
      sortDescFirst: options.sortDescFirst,
      enableRowSelection:
        options.__enableRowSelection === "odd_ids"
          ? (row: any) => {
              const id = Number.parseInt(String(row?.id ?? ""), 10)
              return Number.isFinite(id) ? id % 2 === 1 : false
            }
          : options.__enableRowSelection === "except_11"
            ? (row: any) => String(row?.id ?? "") !== "11"
            : options.__enableRowSelection === "always_false"
              ? (_row: any) => false
              : options.enableRowSelection ?? true,
      enableMultiRowSelection:
        options.__enableMultiRowSelection === "always_false"
          ? (_row: any) => false
          : options.enableMultiRowSelection ?? true,
      enableSubRowSelection:
        options.__enableSubRowSelection === "disable_root_1"
          ? (row: any) => String(row?.id ?? "") !== "1"
          : options.__enableSubRowSelection === "always_false"
            ? (_row: any) => false
            : options.enableSubRowSelection ?? true,
      enableColumnResizing: options.enableColumnResizing ?? true,
      enableHiding: options.enableHiding ?? true,
      enableRowPinning:
        options.__enableRowPinning === "odd_ids"
          ? (row: any) => {
              const id = Number.parseInt(String(row?.id ?? ""), 10)
              return Number.isFinite(id) ? id % 2 === 1 : false
            }
          : options.enableRowPinning,
      getRowCanExpand:
        options.__getRowCanExpand === "only_root_1"
          ? (row: any) => String(row?.id ?? "") === "1"
          : undefined,
      getIsRowExpanded:
        options.__getIsRowExpanded === "always_false" ? (_row: any) => false : undefined,
      enableGrouping: options.enableGrouping,
      enableColumnPinning: options.enableColumnPinning,
      enablePinning: options.enablePinning,
      groupedColumnMode: options.groupedColumnMode,
      columnResizeMode: options.columnResizeMode,
      columnResizeDirection: options.columnResizeDirection,
      renderFallbackValue: options.renderFallbackValue,
      ...(options.globalFilterFn !== undefined
        ? { globalFilterFn: options.globalFilterFn }
        : {}),
      sortingFns:
        options.sortingFnsMode === "custom_text"
          ? {
              custom_text: sortTextBuiltin,
            }
          : undefined,
      filterFns:
        options.filterFnsMode === "custom_text_case_sensitive"
          ? {
              custom_text: filterContainsAsciiCS,
            }
          : undefined,
      aggregationFns:
        options.aggregationFnsMode === "custom_plus_one"
          ? {
              custom_plus_one: (columnId: string, leafRows: any[]) => {
                let sum = 0
                for (const row of leafRows) {
                  const v = row.getValue?.(columnId)
                  sum += typeof v === "number" ? v : 0
                }
                return sum + 1
              },
            }
          : undefined,
      isMultiSortEvent: (e: unknown) => {
        return !!(e as any)?.multi
      },
      state: currentState,
      getCoreRowModel: tableCore.getCoreRowModel(),
      getFilteredRowModel: tableCore.getFilteredRowModel(),
      ...(case_id === "faceting"
        ? {
            getFacetedRowModel: tableCore.getFacetedRowModel(),
            getFacetedUniqueValues: tableCore.getFacetedUniqueValues(),
            getFacetedMinMaxValues: tableCore.getFacetedMinMaxValues(),
          }
        : {}),
      ...(case_id === "grouping" ||
      case_id === "grouping_aggregation_fns" ||
      case_id === "row_id_state_ops" ||
      case_id === "pinning_grouped_rows"
        ? {
            getGroupedRowModel:
              options.__getGroupedRowModel === "pre_grouped"
                ? (t: any) => () => t.getPreGroupedRowModel?.()
                : tableCore.getGroupedRowModel(),
          }
        : {}),
      getSortedRowModel: tableCore.getSortedRowModel(),
      getPaginationRowModel:
        options.__getPaginationRowModel === "pre_pagination"
          ? (t: any) => () => t.getPrePaginationRowModel?.()
          : tableCore.getPaginationRowModel(),
      getExpandedRowModel:
        options.__getExpandedRowModel === "pre_expanded"
          ? (t: any) => () => t.getPreExpandedRowModel?.()
          : tableCore.getExpandedRowModel(),
      onSortingChange: (updater: any) => {
        const next = typeof updater === "function" ? updater(currentState.sorting) : updater
        currentState.sorting = next ?? []
      },
      onColumnFiltersChange: (updater: any) => {
        if (options.__onColumnFiltersChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.columnFilters) : updater
        currentState.columnFilters = next ?? []
      },
      onGlobalFilterChange: (updater: any) => {
        if (options.__onGlobalFilterChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.globalFilter) : updater
        currentState.globalFilter = next
      },
      onPaginationChange: (updater: any) => {
        if (options.__onPaginationChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.pagination) : updater
        currentState.pagination = next ?? currentState.pagination
      },
      onColumnVisibilityChange: (updater: any) => {
        if (options.__onColumnVisibilityChange === "noop") {
          return
        }
        const next =
          typeof updater === "function"
            ? updater(currentState.columnVisibility)
            : updater
        currentState.columnVisibility = next ?? {}
      },
      onColumnSizingChange: (updater: any) => {
        if (options.__onColumnSizingChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.columnSizing) : updater
        currentState.columnSizing = next ?? {}
      },
      onColumnSizingInfoChange: (updater: any) => {
        if (options.__onColumnSizingInfoChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.columnSizingInfo) : updater
        currentState.columnSizingInfo = next ?? currentState.columnSizingInfo
      },
      onColumnPinningChange: (updater: any) => {
        if (options.__onColumnPinningChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.columnPinning) : updater
        currentState.columnPinning = next ?? { left: [], right: [] }
      },
      onColumnOrderChange: (updater: any) => {
        if (options.__onColumnOrderChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.columnOrder) : updater
        currentState.columnOrder = next ?? []
      },
      onRowPinningChange: (updater: any) => {
        if (options.__onRowPinningChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.rowPinning) : updater
        currentState.rowPinning = next ?? { top: [], bottom: [] }
      },
      onRowSelectionChange: (updater: any) => {
        if (options.__onRowSelectionChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.rowSelection) : updater
        currentState.rowSelection = next ?? {}
      },
      onExpandedChange: (updater: any) => {
        if (options.__onExpandedChange === "noop") {
          return
        }
        const next =
          typeof updater === "function" ? updater(currentState.expanded) : updater
        currentState.expanded = next ?? {}
      },
      onGroupingChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.grouping) : updater
        currentState.grouping = next ?? []
      },
      onStateChange: () => {},
    })

    return { table, currentState }
  }

function snapshotForState(
  options: TanStackOptions,
  state: TanStackState,
  extras?: (table: any) => Partial<FixtureSnapshot["expect"]>,
): FixtureSnapshot["expect"] {
  const { table } = buildTable(options, state)
  const out: FixtureSnapshot["expect"] = {
      core: snapshotRowModel(table.getCoreRowModel()),
      filtered: snapshotRowModel(table.getFilteredRowModel()),
      sorted: snapshotRowModel(table.getSortedRowModel()),
      expanded: snapshotRowModel(table.getExpandedRowModel()),
      paginated: snapshotRowModel(table.getPaginationRowModel()),
      row_model: snapshotRowModel(table.getRowModel()),
      page_count: Number(table.getPageCount?.() ?? 0),
      row_count: Number(table.getRowCount?.() ?? 0),
      can_previous_page: Boolean(table.getCanPreviousPage?.()),
      can_next_page: Boolean(table.getCanNextPage?.()),
      page_options: (table.getPageOptions?.() ?? []).map((v: any) => Number(v)),
      selected: snapshotRowModel(table.getSelectedRowModel?.() ?? emptyRowModelSnapshot()),
      filtered_selected: snapshotRowModel(
        table.getFilteredSelectedRowModel?.() ?? emptyRowModelSnapshot(),
      ),
    grouped_selected: snapshotRowModel(table.getGroupedSelectedRowModel?.() ?? emptyRowModelSnapshot()),
    is_all_rows_selected: Boolean(table.getIsAllRowsSelected?.()),
    is_some_rows_selected: Boolean(table.getIsSomeRowsSelected?.()),
    is_all_page_rows_selected: Boolean(table.getIsAllPageRowsSelected?.()),
    is_some_page_rows_selected: Boolean(table.getIsSomePageRowsSelected?.()),
    is_all_rows_expanded: Boolean(table.getIsAllRowsExpanded?.()),
    is_some_rows_expanded: Boolean(table.getIsSomeRowsExpanded?.()),
    can_some_rows_expand: Boolean(table.getCanSomeRowsExpand?.()),
  }

  return { ...out, ...(extras ? extras(table) : {}) }
}

function emptyRowModelSnapshot(): any {
  return { rows: [], flatRows: [] }
}

function snapshotFilteringHelpers(table: any): FilteringHelpersSnapshot {
  const out: FilteringHelpersSnapshot = {
    columns: {},
    global_filter: table.getState?.().globalFilter ?? null,
  }

  const cols: any[] = table.getAllLeafColumns?.() ?? []
  for (const col of cols) {
    const id = String(col.id)

    const filterValue = col.getFilterValue?.()
    out.columns[id] = {
      can_filter: Boolean(col.getCanFilter?.()),
      filter_value: filterValue === undefined ? null : filterValue,
      is_filtered: Boolean(col.getIsFiltered?.()),
      filter_index: Number(col.getFilterIndex?.() ?? -1),
      can_global_filter: Boolean(col.getCanGlobalFilter?.()),
    }
  }

  return out
}

function snapshotSortingHelpers(table: any): SortingHelpersSnapshot {
  const out: SortingHelpersSnapshot = {
    columns: {},
  }

  const cols: any[] = table.getAllLeafColumns?.() ?? []
  for (const col of cols) {
    const id = String(col.id)
    const raw = col.getIsSorted?.()
    const is_sorted =
      raw === undefined || raw === false ? null : (String(raw) as "asc" | "desc")

    const auto_sort_dir_raw = col.getAutoSortDir?.()
    const auto_sort_dir =
      auto_sort_dir_raw === undefined || auto_sort_dir_raw === null
        ? null
        : (String(auto_sort_dir_raw) as "asc" | "desc")

    const first_sort_dir_raw = col.getFirstSortDir?.()
    const first_sort_dir =
      first_sort_dir_raw === undefined || first_sort_dir_raw === null
        ? null
        : (String(first_sort_dir_raw) as "asc" | "desc")

    const next_sorting_order_raw = col.getNextSortingOrder?.()
    const next_sorting_order =
      next_sorting_order_raw === undefined ||
      next_sorting_order_raw === null ||
      next_sorting_order_raw === false
        ? null
        : (String(next_sorting_order_raw) as "asc" | "desc")

    const next_sorting_order_multi_raw = col.getNextSortingOrder?.(true)
    const next_sorting_order_multi =
      next_sorting_order_multi_raw === undefined ||
      next_sorting_order_multi_raw === null ||
      next_sorting_order_multi_raw === false
        ? null
        : (String(next_sorting_order_multi_raw) as "asc" | "desc")

    out.columns[id] = {
      can_sort: Boolean(col.getCanSort?.()),
      can_multi_sort: Boolean(col.getCanMultiSort?.()),
      is_sorted,
      sort_index: Number(col.getSortIndex?.() ?? -1),
      auto_sort_dir,
      first_sort_dir,
      next_sorting_order,
      next_sorting_order_multi,
    }
  }

  return out
}

function snapshotHeaderGroups(groups: any[]): {
  id: string
  depth: number
  headers: {
    id: string
    column_id: string
    depth: number
    index: number
    is_placeholder: boolean
    placeholder_id: string | null
    col_span: number
    row_span: number
    sub_header_ids: string[]
  }[]
}[] {
  return groups.map((g) => ({
    id: String(g.id),
    depth: Number(g.depth),
    headers: (g.headers ?? []).map((h: any) => ({
      id: String(h.id),
      column_id: String(h.column?.id),
      depth: Number(h.depth),
      index: Number(h.index),
      is_placeholder: Boolean(h.isPlaceholder),
      placeholder_id: h.placeholderId === undefined ? null : String(h.placeholderId),
      col_span: Number(h.colSpan),
      row_span: Number(h.rowSpan),
      sub_header_ids: (h.subHeaders ?? []).map((sh: any) => String(sh.id)),
    })),
  }))
}

function snapshotHeaders(headers: any[]): {
  id: string
  column_id: string
  depth: number
  index: number
  is_placeholder: boolean
  placeholder_id: string | null
  col_span: number
  row_span: number
  sub_header_ids: string[]
}[] {
  return (headers ?? []).map((h: any) => ({
    id: String(h.id),
    column_id: String(h.column?.id),
    depth: Number(h.depth),
    index: Number(h.index),
    is_placeholder: Boolean(h.isPlaceholder),
    placeholder_id: h.placeholderId === undefined ? null : String(h.placeholderId),
    col_span: Number(h.colSpan),
    row_span: Number(h.rowSpan),
    sub_header_ids: (h.subHeaders ?? []).map((sh: any) => String(sh.id)),
  }))
}

function snapshotCells(table: any): Record<
  string,
  {
    all: { id: string; column_id: string }[]
    visible: { id: string; column_id: string }[]
    left: { id: string; column_id: string }[]
    center: { id: string; column_id: string }[]
    right: { id: string; column_id: string }[]
  }
> {
  const rows = table.getRowModel().rows ?? []
  const out: Record<
    string,
    {
      all: { id: string; column_id: string }[]
      visible: { id: string; column_id: string }[]
      left: { id: string; column_id: string }[]
      center: { id: string; column_id: string }[]
      right: { id: string; column_id: string }[]
    }
  > = {}
  for (const r of rows) {
    const rowId = String(r.id)
    out[rowId] = {
      all: (r.getAllCells?.() ?? []).map((c: any) => ({
        id: String(c.id),
        column_id: String(c.column?.id),
      })),
      visible: (r.getVisibleCells?.() ?? []).map((c: any) => ({
        id: String(c.id),
        column_id: String(c.column?.id),
      })),
      left: (r.getLeftVisibleCells?.() ?? []).map((c: any) => ({
        id: String(c.id),
        column_id: String(c.column?.id),
      })),
      center: (r.getCenterVisibleCells?.() ?? []).map((c: any) => ({
        id: String(c.id),
        column_id: String(c.column?.id),
      })),
      right: (r.getRightVisibleCells?.() ?? []).map((c: any) => ({
        id: String(c.id),
        column_id: String(c.column?.id),
      })),
    }
  }
  return out
}

function snapshotColumnTree(columns: any[], parent_id: string | null = null): {
  id: string
  depth: number
  parent_id: string | null
  child_ids: string[]
}[] {
  const out: {
    id: string
    depth: number
    parent_id: string | null
    child_ids: string[]
  }[] = []

  for (const c of columns) {
    const id = String(c.id)
    const depth = Number(c.depth ?? 0)
    const childIds: string[] = (c.columns ?? []).map((cc: any) => String(cc.id))
    out.push({ id, depth, parent_id, child_ids: childIds })
    if (c.columns && c.columns.length) {
      out.push(...snapshotColumnTree(c.columns, id))
    }
  }

  return out
}

function snapshotColumnSizing(table: any): {
  column_sizing: NonNullable<FixtureSnapshot["expect"]["column_sizing"]>
  column_start: NonNullable<FixtureSnapshot["expect"]["column_start"]>
  column_after: NonNullable<FixtureSnapshot["expect"]["column_after"]>
} {
  const pinning: TanStackColumnPinning = table.getState().columnPinning ?? {}
  const leftPins = new Set(pinning.left ?? [])
  const rightPins = new Set(pinning.right ?? [])

  const cols: any[] = table.getAllLeafColumns()

  const starts_all: Record<string, number> = {}
  const starts_left: Record<string, number | null> = {}
  const starts_center: Record<string, number | null> = {}
  const starts_right: Record<string, number | null> = {}
  const after_all: Record<string, number> = {}
  const after_left: Record<string, number | null> = {}
  const after_center: Record<string, number | null> = {}
  const after_right: Record<string, number | null> = {}

  for (const col of cols) {
    const id = String(col.id)
    starts_all[id] = Number(col.getStart())
    after_all[id] = Number(col.getAfter())

    const isLeft = leftPins.has(id)
    const isRight = rightPins.has(id)
    const isCenter = !isLeft && !isRight

    starts_left[id] = isLeft ? Number(col.getStart("left")) : null
    starts_center[id] = isCenter ? Number(col.getStart("center")) : null
    starts_right[id] = isRight ? Number(col.getStart("right")) : null

    after_left[id] = isLeft ? Number(col.getAfter("left")) : null
    after_center[id] = isCenter ? Number(col.getAfter("center")) : null
    after_right[id] = isRight ? Number(col.getAfter("right")) : null
  }

  return {
    column_sizing: {
      total_size: Number(table.getTotalSize()),
      left_total_size: Number(table.getLeftTotalSize()),
      center_total_size: Number(table.getCenterTotalSize()),
      right_total_size: Number(table.getRightTotalSize()),
    },
    column_start: {
      all: starts_all,
      left: starts_left,
      center: starts_center,
      right: starts_right,
    },
    column_after: {
      all: after_all,
      left: after_left,
      center: after_center,
      right: after_right,
    },
  }
}

function snapshotHeaderSizing(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["header_sizing"]> {
  const outSize: Record<string, number> = {}
  const outStart: Record<string, number> = {}

  const groupsLists = [
    table.getHeaderGroups?.(),
    table.getLeftHeaderGroups?.(),
    table.getCenterHeaderGroups?.(),
    table.getRightHeaderGroups?.(),
  ].filter(Boolean)

  for (const groups of groupsLists) {
    for (const group of groups ?? []) {
      for (const header of group?.headers ?? []) {
        const id = String(header?.id)
        outSize[id] = Number(header.getSize?.() ?? 0)
        outStart[id] = Number(header.getStart?.() ?? 0)
      }
    }
  }

  return { size: outSize, start: outStart }
}

function snapshotRenderFallback(table: any): NonNullable<FixtureSnapshot["expect"]["render_fallback"]> {
  const model = table.getRowModel?.()
  const flat = model?.flatRows ?? []
  const out: NonNullable<FixtureSnapshot["expect"]["render_fallback"]> = []

  for (const row of flat) {
    const cells = row.getAllCells?.() ?? []
    for (const cell of cells) {
      const columnId = String(cell?.column?.id)
      const value = jsonSafe(cell.getValue?.())
      const render_value = jsonSafe(cell.renderValue?.())
      out.push({
        row_id: String(row.id),
        column_id: columnId,
        value,
        render_value,
      })
    }
  }

  return out
}

function snapshotRowPinning(table: any): NonNullable<FixtureSnapshot["expect"]["row_pinning"]> {
  if (
    typeof table.getTopRows !== "function" ||
    typeof table.getCenterRows !== "function" ||
    typeof table.getBottomRows !== "function"
  ) {
    throw new Error("Row pinning APIs are not available on this table instance")
  }

  const top = (table.getTopRows?.() ?? []).map((r: any) => String(r.id))
  const center = (table.getCenterRows?.() ?? []).map((r: any) => String(r.id))
  const bottom = (table.getBottomRows?.() ?? []).map((r: any) => String(r.id))

  const coreRows = table.getCoreRowModel?.()?.flatRows ?? []
  const can_pin: Record<string, boolean> = {}
  const pin_position: Record<string, "top" | "bottom" | null> = {}
  const pinned_index: Record<string, number> = {}
  for (const row of coreRows) {
    const id = String(row.id)
    const r = table.getRow?.(id, true)
    if (!r) {
      continue
    }
    can_pin[id] = Boolean(r.getCanPin?.())
    const pos = r.getIsPinned?.()
    pin_position[id] = pos === "top" ? "top" : pos === "bottom" ? "bottom" : null
    pinned_index[id] = Number(r.getPinnedIndex?.() ?? -1)
  }

  return {
    top,
    center,
    bottom,
    can_pin,
    pin_position,
    pinned_index,
    is_some_rows_pinned: Boolean(table.getIsSomeRowsPinned?.()),
    is_some_top_rows_pinned: Boolean(table.getIsSomeRowsPinned?.("top")),
    is_some_bottom_rows_pinned: Boolean(table.getIsSomeRowsPinned?.("bottom")),
  }
}

function snapshotFaceting(table: any, column_id: string): {
  row_model: NonNullable<FixtureSnapshot["expect"]["core"]>
  unique_values: Record<string, number>
  min_max: [number, number] | null
} {
  const col = table.getColumn?.(column_id)
  if (!col) {
    throw new Error(`Unknown column in faceting snapshot: ${column_id}`)
  }

  if (typeof col.getFacetedRowModel !== "function") {
    throw new Error("Faceting APIs are not available on this column instance")
  }

  const faceted = col.getFacetedRowModel?.()
  const unique = col.getFacetedUniqueValues?.()
  const minmax = col.getFacetedMinMaxValues?.()

  const unique_values: Record<string, number> = {}
  for (const [k, v] of unique?.entries?.() ?? []) {
    unique_values[String(k)] = Number(v)
  }

  return {
    row_model: snapshotRowModel(faceted),
    unique_values,
    min_max:
      Array.isArray(minmax) && minmax.length === 2
        ? [Number(minmax[0]), Number(minmax[1])]
        : null,
  }
}

function snapshotGlobalFaceting(table: any): {
  row_model: NonNullable<FixtureSnapshot["expect"]["core"]>
  unique_values: Record<string, number>
  min_max: [number, number] | null
} {
  const model = table.getGlobalFacetedRowModel?.()
  const unique = table.getGlobalFacetedUniqueValues?.()
  const minmax = table.getGlobalFacetedMinMaxValues?.()

  const unique_values: Record<string, number> = {}
  for (const [k, v] of unique?.entries?.() ?? []) {
    unique_values[String(k)] = Number(v)
  }

  return {
    row_model: snapshotRowModel(model),
    unique_values,
    min_max:
      Array.isArray(minmax) && minmax.length === 2
        ? [Number(minmax[0]), Number(minmax[1])]
        : null,
  }
}

function snapshotGroupedRowModel(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["grouped_row_model"]> {
  if (typeof table.getGroupedRowModel !== "function") {
    throw new Error("Grouped row model APIs are not available on this table instance")
  }

  const grouped = table.getGroupedRowModel()
  const rootRows = grouped?.rows ?? []
  const flatRows = grouped?.flatRows ?? []

  type PathEntry = { column_id: string; value: unknown }
  type Node = NonNullable<FixtureSnapshot["expect"]["grouped_row_model"]>["root"][number]

  const nodesById = new Map<string, Node>()
  const root: Node[] = []

  const walk = (rows: any[], path: PathEntry[]) => {
    for (const row of rows) {
      const isGroup = !!row.groupingColumnId
      if (isGroup) {
        const nextPath: PathEntry[] = [
          ...path,
          { column_id: String(row.groupingColumnId), value: row.groupingValue },
        ]

        const node: Node = {
          kind: "group",
          depth: Number(row.depth ?? 0),
          path: nextPath,
          grouping_column_id: String(row.groupingColumnId),
          grouping_value: row.groupingValue,
          leaf_row_count: Number(row.leafRows?.length ?? 0),
          first_leaf_row_id: row.leafRows?.[0]?.id ?? null,
        }
        nodesById.set(String(row.id), node)
        root.push(node)
        walk(row.subRows ?? [], nextPath)
        continue
      }

      const node: Node = {
        kind: "leaf",
        depth: Number(row.depth ?? 0),
        path,
        row_id: String(row.id),
      }
      nodesById.set(String(row.id), node)
      root.push(node)
    }
  }

  walk(rootRows, [])

  const flat: Node[] = flatRows.map((row: any) => {
    const id = String(row.id)
    const cached = nodesById.get(id)
    if (cached) {
      return cached
    }

    const isGroup = !!row.groupingColumnId
    const path: PathEntry[] = isGroup
      ? [{ column_id: String(row.groupingColumnId), value: row.groupingValue }]
      : []

    return isGroup
      ? {
          kind: "group",
          depth: Number(row.depth ?? 0),
          path,
          grouping_column_id: String(row.groupingColumnId),
          grouping_value: row.groupingValue,
          leaf_row_count: Number(row.leafRows?.length ?? 0),
          first_leaf_row_id: row.leafRows?.[0]?.id ?? null,
        }
      : {
          kind: "leaf",
          depth: Number(row.depth ?? 0),
          path,
          row_id: id,
        }
  })

  return { root, flat }
}

function snapshotGroupedAggregationsU64(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["grouped_aggregations_u64"]> {
  if (typeof table.getGroupedRowModel !== "function") {
    throw new Error("Grouped row model APIs are not available on this table instance")
  }

  const grouped = table.getGroupedRowModel()
  const rootRows = grouped?.rows ?? []

  const grouping = (table.getState?.().grouping ?? []) as string[]
  const groupedColumnIds = new Set(grouping.map((v) => String(v)))

  const leaf = (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const aggCols = leaf.filter((id) => !groupedColumnIds.has(id))

  if (!aggCols.length) {
    return []
  }

  const out: NonNullable<FixtureSnapshot["expect"]["grouped_aggregations_u64"]> = []

  const walk = (rows: any[], path: { column_id: string; value: unknown }[]) => {
    for (const row of rows) {
      const isGroup = !!row.groupingColumnId
      if (!isGroup) {
        continue
      }

      const nextPath = [
        ...path,
        {
          column_id: String(row.groupingColumnId),
          value: String(row.groupingValue),
        },
      ]

      const values: Record<string, number | null> = {}
      for (const colId of aggCols) {
        const v = row.getValue?.(colId)
        values[colId] =
          typeof v === "number" ? v : v == null ? null : Number(v)
      }

      out.push({ path: nextPath, values })
      walk(row.subRows ?? [], nextPath)
    }
  }

  walk(rootRows, [])
  return out
}

function snapshotGroupedAggregationsAny(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["grouped_aggregations_any"]> {
  if (typeof table.getGroupedRowModel !== "function") {
    throw new Error("Grouped row model APIs are not available on this table instance")
  }

  const grouped = table.getGroupedRowModel()
  const rootRows = grouped?.rows ?? []

  const grouping = (table.getState?.().grouping ?? []) as string[]
  const groupedColumnIds = new Set(grouping.map((v) => String(v)))

  const leaf = (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const aggCols = leaf.filter((id) => !groupedColumnIds.has(id))

  if (!aggCols.length) {
    return []
  }

  const out: NonNullable<FixtureSnapshot["expect"]["grouped_aggregations_any"]> = []

  const walk = (rows: any[], path: { column_id: string; value: unknown }[]) => {
    for (const row of rows) {
      const isGroup = !!row.groupingColumnId
      if (!isGroup) {
        continue
      }

      const nextPath = [
        ...path,
        {
          column_id: String(row.groupingColumnId),
          value: String(row.groupingValue),
        },
      ]

      const values: Record<string, unknown> = {}
      for (const colId of aggCols) {
        values[colId] = jsonSafe(row.getValue?.(colId))
      }

      out.push({ path: nextPath, values })
      walk(row.subRows ?? [], nextPath)
    }
  }

  walk(rootRows, [])
  return out
}

function snapshotSortedGroupedRowModel(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["sorted_grouped_row_model"]> {
  if (typeof table.getSortedRowModel !== "function") {
    throw new Error("Sorted row model APIs are not available on this table instance")
  }

  const sorted = table.getSortedRowModel()
  const rootRows = sorted?.rows ?? []
  const flatRows = sorted?.flatRows ?? []

  type PathEntry = { column_id: string; value: unknown }
  type Node = NonNullable<
    FixtureSnapshot["expect"]["sorted_grouped_row_model"]
  >["root"][number]

  const nodesById = new Map<string, Node>()
  const root: Node[] = []

  const walk = (rows: any[], path: PathEntry[]) => {
    for (const row of rows) {
      const isGroup = !!row.groupingColumnId
      if (isGroup) {
        const nextPath: PathEntry[] = [
          ...path,
          { column_id: String(row.groupingColumnId), value: row.groupingValue },
        ]

        const node: Node = {
          kind: "group",
          depth: Number(row.depth ?? 0),
          path: nextPath,
          grouping_column_id: String(row.groupingColumnId),
          grouping_value: row.groupingValue,
          leaf_row_count: Number(row.leafRows?.length ?? 0),
          first_leaf_row_id: row.leafRows?.[0]?.id ?? null,
        }
        nodesById.set(String(row.id), node)
        root.push(node)
        walk(row.subRows ?? [], nextPath)
        continue
      }

      const node: Node = {
        kind: "leaf",
        depth: Number(row.depth ?? 0),
        path,
        row_id: String(row.id),
      }
      nodesById.set(String(row.id), node)
      root.push(node)
    }
  }

  walk(rootRows, [])

  const flat: Node[] = flatRows.map((row: any) => {
    const id = String(row.id)
    const cached = nodesById.get(id)
    if (cached) {
      return cached
    }

    const isGroup = !!row.groupingColumnId
    const path: PathEntry[] = isGroup
      ? [{ column_id: String(row.groupingColumnId), value: row.groupingValue }]
      : []

    return isGroup
      ? {
          kind: "group",
          depth: Number(row.depth ?? 0),
          path,
          grouping_column_id: String(row.groupingColumnId),
          grouping_value: row.groupingValue,
          leaf_row_count: Number(row.leafRows?.length ?? 0),
          first_leaf_row_id: row.leafRows?.[0]?.id ?? null,
        }
      : {
          kind: "leaf",
          depth: Number(row.depth ?? 0),
          path,
          row_id: id,
        }
  })

  return { root, flat }
}

function snapshotColumnPinning(
  table: any,
): NonNullable<FixtureSnapshot["expect"]["column_pinning"]> {
  if (typeof table.getIsSomeColumnsPinned !== "function") {
    throw new Error("Column pinning APIs are not available on this table instance")
  }

  const flattenColumns = (cols: any[]): any[] => {
    const flat: any[] = []
    const visit = (arr: any[]) => {
      for (const col of arr ?? []) {
        flat.push(col)
        const children = col?.columns
        if (children?.length) {
          visit(children)
        }
      }
    }
    visit(cols)
    return flat
  }

  const all_ids = flattenColumns(table.getAllColumns?.() ?? []).map((c: any) => String(c.id))
  const leaf = (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const ids = [...new Set([...leaf, ...all_ids])]

  const can_pin: Record<string, boolean> = {}
  const pin_position: Record<string, "left" | "right" | null> = {}
  const pinned_index: Record<string, number> = {}
  for (const id of ids) {
    const col = table.getColumn?.(id)
    if (!col) {
      continue
    }
    can_pin[id] = Boolean(col.getCanPin?.())
    const pos = col.getIsPinned?.()
    pin_position[id] = pos === "left" ? "left" : pos === "right" ? "right" : null
    pinned_index[id] = Number(col.getPinnedIndex?.() ?? 0)
  }

  const left = (table.getLeftLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const center = (table.getCenterLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const right = (table.getRightLeafColumns?.() ?? []).map((c: any) => String(c.id))

  return {
    left,
    center,
    right,
    can_pin,
    pin_position,
    pinned_index,
    is_some_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.()),
    is_some_left_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.("left")),
    is_some_right_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.("right")),
  }
}

  function snapshotForActions(
    options: TanStackOptions,
    state: TanStackState,
    actions: FixtureAction[],
    extras?: (table: any) => Partial<FixtureSnapshot["expect"]>,
  ): FixtureSnapshot["expect"] {
    const { table, currentState } = buildTable(options, state)
    const doc = new FakeDocument()
    let activeResize: { column_id: string } | null = null

    for (const action of actions) {
      if (action.type === "toggleSorting") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.toggleSorting(undefined, action.multi ?? false)
        continue
      }
      if (action.type === "clearSorting") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.clearSorting !== "function") {
          throw new Error("Column has no clearSorting")
        }
        col.clearSorting()
        continue
      }
      if (action.type === "toggleSortingHandler") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        const handler = col.getToggleSortingHandler()
        handler?.({
          multi: action.event_multi ?? false,
          persist: () => {},
        })
        continue
      }
      if (action.type === "setColumnFilterValue") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.setFilterValue(action.value)
        continue
      }
      if (action.type === "setGlobalFilterValue") {
        table.setGlobalFilter(action.value)
        continue
      }
      if (action.type === "toggleColumnVisibility") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.toggleVisibility(action.value)
        continue
      }
      if (action.type === "toggleAllColumnsVisible") {
        table.toggleAllColumnsVisible(action.value)
        continue
      }
      if (action.type === "setColumnOrder") {
        table.setColumnOrder(action.order)
        continue
      }
      if (action.type === "pinRow") {
        const row = table.getRow(action.row_id, true)
        if (!row) {
          throw new Error(`Unknown row in action: ${action.row_id}`)
        }
        row.pin(
          action.position === null ? false : action.position,
          action.include_leaf_rows ?? false,
          action.include_parent_rows ?? false,
        )
        continue
      }
      if (action.type === "pinColumn") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.pin(action.position === null ? false : action.position)
        continue
      }
      if (action.type === "resetRowPinning") {
        if (typeof table.resetRowPinning !== "function") {
          throw new Error("Table has no resetRowPinning")
        }
        table.resetRowPinning(action.default_state)
        continue
      }
      if (action.type === "resetColumnPinning") {
        if (typeof table.resetColumnPinning !== "function") {
          throw new Error("Table has no resetColumnPinning")
        }
        table.resetColumnPinning(action.default_state)
        continue
      }
      if (action.type === "toggleRowSelected") {
        const row = table.getRow(action.row_id)
        if (!row) {
          throw new Error(`Unknown row in action: ${action.row_id}`)
        }
        row.toggleSelected(action.value, { selectChildren: action.select_children ?? true })
        continue
      }
      if (action.type === "toggleAllRowsSelected") {
        table.toggleAllRowsSelected(action.value)
        continue
      }
      if (action.type === "toggleAllPageRowsSelected") {
        table.toggleAllPageRowsSelected(action.value)
        continue
      }
      if (action.type === "resetRowSelection") {
        if (typeof table.resetRowSelection !== "function") {
          throw new Error("Table has no resetRowSelection")
        }
        table.resetRowSelection(action.default_state)
        continue
      }
      if (action.type === "toggleRowExpanded") {
        const row = table.getRow(action.row_id)
        if (!row) {
          throw new Error(`Unknown row in action: ${action.row_id}`)
        }
        row.toggleExpanded(action.value)
        continue
      }
      if (action.type === "toggleAllRowsExpanded") {
        table.toggleAllRowsExpanded(action.value)
        continue
      }
      if (action.type === "setPageIndex") {
        if (typeof table.setPageIndex !== "function") {
          throw new Error("Table has no setPageIndex")
        }
        table.setPageIndex(action.page_index)
        continue
      }
      if (action.type === "setPageSize") {
        if (typeof table.setPageSize !== "function") {
          throw new Error("Table has no setPageSize")
        }
        table.setPageSize(action.page_size)
        continue
      }
      if (action.type === "nextPage") {
        if (typeof table.nextPage !== "function") {
          throw new Error("Table has no nextPage")
        }
        table.nextPage()
        continue
      }
      if (action.type === "previousPage") {
        if (typeof table.previousPage !== "function") {
          throw new Error("Table has no previousPage")
        }
        table.previousPage()
        continue
      }
      if (action.type === "firstPage") {
        if (typeof table.firstPage !== "function") {
          throw new Error("Table has no firstPage")
        }
        table.firstPage()
        continue
      }
      if (action.type === "lastPage") {
        if (typeof table.lastPage !== "function") {
          throw new Error("Table has no lastPage")
        }
        table.lastPage()
        continue
      }
      if (action.type === "resetPageIndex") {
        if (typeof table.resetPageIndex !== "function") {
          throw new Error("Table has no resetPageIndex")
        }
        table.resetPageIndex(action.default_state)
        continue
      }
      if (action.type === "resetPageSize") {
        if (typeof table.resetPageSize !== "function") {
          throw new Error("Table has no resetPageSize")
        }
        table.resetPageSize(action.default_state)
        continue
      }
      if (action.type === "resetPagination") {
        if (typeof table.resetPagination !== "function") {
          throw new Error("Table has no resetPagination")
        }
        table.resetPagination(action.default_state)
        continue
      }
      if (action.type === "resetSorting") {
        if (typeof table.resetSorting !== "function") {
          throw new Error("Table has no resetSorting")
        }
        table.resetSorting(action.default_state)
        continue
      }
      if (action.type === "resetColumnFilters") {
        if (typeof table.resetColumnFilters !== "function") {
          throw new Error("Table has no resetColumnFilters")
        }
        table.resetColumnFilters(action.default_state)
        continue
      }
      if (action.type === "resetGlobalFilter") {
        if (typeof table.resetGlobalFilter !== "function") {
          throw new Error("Table has no resetGlobalFilter")
        }
        table.resetGlobalFilter(action.default_state)
        continue
      }
      if (action.type === "resetGrouping") {
        if (typeof table.resetGrouping !== "function") {
          throw new Error("Table has no resetGrouping")
        }
        table.resetGrouping(action.default_state)
        continue
      }
      if (action.type === "resetColumnVisibility") {
        if (typeof table.resetColumnVisibility !== "function") {
          throw new Error("Table has no resetColumnVisibility")
        }
        table.resetColumnVisibility(action.default_state)
        continue
      }
      if (action.type === "resetColumnOrder") {
        if (typeof table.resetColumnOrder !== "function") {
          throw new Error("Table has no resetColumnOrder")
        }
        table.resetColumnOrder(action.default_state)
        continue
      }
      if (action.type === "toggleGrouping") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.toggleGrouping !== "function") {
          throw new Error(`Column has no toggleGrouping: ${action.column_id}`)
        }
        col.toggleGrouping(action.value)
        continue
      }
      if (action.type === "toggleGroupingHandler") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.getToggleGroupingHandler !== "function") {
          throw new Error(`Column has no getToggleGroupingHandler: ${action.column_id}`)
        }
        const handler = col.getToggleGroupingHandler()
        if (typeof handler !== "function") {
          throw new Error(`Column returned no toggle handler: ${action.column_id}`)
        }
        handler()
        continue
      }
      if (action.type === "setGrouping") {
        if (typeof table.setGrouping !== "function") {
          throw new Error("Table has no setGrouping")
        }
        table.setGrouping(action.grouping)
        continue
      }
      if (action.type === "columnResizeBegin") {
        const headerGroups = table.getHeaderGroups?.() ?? []
        const headers: any[] = headerGroups.flatMap((g: any) => g.headers ?? [])
        const header = headers.find((h: any) => String(h?.column?.id) === action.column_id)
        if (!header) {
          throw new Error(`Unknown header in action: ${action.column_id}`)
        }
        const handler = header.getResizeHandler?.(doc as any)
        if (typeof handler !== "function") {
          throw new Error(`Header has no resize handler: ${action.column_id}`)
        }
        handler({
          clientX: action.client_x,
          persist: () => {},
        })
        activeResize = { column_id: action.column_id }
        continue
      }
      if (action.type === "columnResizeMove") {
        if (!activeResize) {
          throw new Error("columnResizeMove without an active resize session")
        }
        doc.dispatch("mousemove", { clientX: action.client_x })
        continue
      }
      if (action.type === "columnResizeEnd") {
        if (!activeResize) {
          throw new Error("columnResizeEnd without an active resize session")
        }
        doc.dispatch("mouseup", { clientX: action.client_x })
        activeResize = null
        continue
      }
      if (action.type === "resetColumnSize") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.resetSize !== "function") {
          throw new Error(`Column has no resetSize: ${action.column_id}`)
        }
        col.resetSize()
        continue
      }
      if (action.type === "resetColumnSizing") {
        if (typeof table.resetColumnSizing !== "function") {
          throw new Error("Table has no resetColumnSizing")
        }
        table.resetColumnSizing(action.default_state)
        continue
      }
      if (action.type === "resetHeaderSizeInfo") {
        if (typeof table.resetHeaderSizeInfo !== "function") {
          throw new Error("Table has no resetHeaderSizeInfo")
        }
        table.resetHeaderSizeInfo(action.default_state)
        continue
      }
      // exhaustive
      const _exhaustive: never = action
      throw new Error(`Unhandled action: ${JSON.stringify(_exhaustive)}`)
    }

    const sizing = snapshotColumnSizing(table)

    return {
      core: snapshotRowModel(table.getCoreRowModel()),
      filtered: snapshotRowModel(table.getFilteredRowModel()),
      sorted: snapshotRowModel(table.getSortedRowModel()),
      expanded: snapshotRowModel(table.getExpandedRowModel()),
      paginated: snapshotRowModel(table.getPaginationRowModel()),
      row_model: snapshotRowModel(table.getRowModel()),
      page_count: Number(table.getPageCount?.() ?? 0),
      row_count: Number(table.getRowCount?.() ?? 0),
      can_previous_page: Boolean(table.getCanPreviousPage?.()),
      can_next_page: Boolean(table.getCanNextPage?.()),
      page_options: (table.getPageOptions?.() ?? []).map((v: any) => Number(v)),
      selected: snapshotRowModel(table.getSelectedRowModel?.() ?? emptyRowModelSnapshot()),
      filtered_selected: snapshotRowModel(
        table.getFilteredSelectedRowModel?.() ?? emptyRowModelSnapshot(),
      ),
      grouped_selected: snapshotRowModel(table.getGroupedSelectedRowModel?.() ?? emptyRowModelSnapshot()),
      is_all_rows_selected: Boolean(table.getIsAllRowsSelected?.()),
      is_some_rows_selected: Boolean(table.getIsSomeRowsSelected?.()),
      is_all_page_rows_selected: Boolean(table.getIsAllPageRowsSelected?.()),
      is_some_page_rows_selected: Boolean(table.getIsSomePageRowsSelected?.()),
      is_all_rows_expanded: Boolean(table.getIsAllRowsExpanded?.()),
      is_some_rows_expanded: Boolean(table.getIsSomeRowsExpanded?.()),
      can_some_rows_expand: Boolean(table.getCanSomeRowsExpand?.()),
      ...sizing,
      ...(extras ? extras(table) : {}),
      next_state: {
        sorting: currentState.sorting ?? [],
        columnFilters: currentState.columnFilters ?? [],
        globalFilter: currentState.globalFilter,
        pagination: currentState.pagination,
        grouping: currentState.grouping ?? [],
        expanded: currentState.expanded,
        rowPinning: currentState.rowPinning,
        rowSelection: currentState.rowSelection ?? {},
        columnVisibility: currentState.columnVisibility,
        columnSizing: currentState.columnSizing ?? {},
        columnSizingInfo: currentState.columnSizingInfo,
        columnPinning: currentState.columnPinning,
        columnOrder: currentState.columnOrder,
      },
    }
  }

  async function snapshotForGroupingActionsWithAutoResetFlush(
    options: TanStackOptions,
    state: TanStackState,
    actions: FixtureAction[],
  ): Promise<FixtureSnapshot["expect"]> {
    const { table, currentState } = buildTable(options, state)

    for (const action of actions) {
      if (action.type === "toggleGrouping") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.toggleGrouping !== "function") {
          throw new Error(`Column has no toggleGrouping: ${action.column_id}`)
        }
        col.toggleGrouping(action.value)
      } else if (action.type === "toggleGroupingHandler") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        if (typeof col.getToggleGroupingHandler !== "function") {
          throw new Error(`Column has no getToggleGroupingHandler: ${action.column_id}`)
        }
        const handler = col.getToggleGroupingHandler()
        if (typeof handler !== "function") {
          throw new Error(`Column returned no toggle handler: ${action.column_id}`)
        }
        handler()
      } else if (action.type === "setGrouping") {
        if (typeof table.setGrouping !== "function") {
          throw new Error("Table has no setGrouping")
        }
        table.setGrouping(action.grouping)
      } else {
        throw new Error(
          `snapshotForGroupingActionsWithAutoResetFlush only supports grouping actions; got: ${action.type}`,
        )
      }

      // Simulate a render pass where the grouped row model is recomputed. TanStack's
      // `getGroupedRowModel` memo debug callback queues `_autoResetExpanded()`.
      table.getGroupedRowModel?.()

      // Flush the microtask queue to run `table._queue` callbacks (registration + reset).
      await Promise.resolve()
      await Promise.resolve()
    }

    const expect = snapshotForState(options, currentState)

    return {
      ...expect,
      next_state: {
        sorting: currentState.sorting ?? [],
        columnFilters: currentState.columnFilters ?? [],
        globalFilter: currentState.globalFilter,
        pagination: currentState.pagination,
        grouping: currentState.grouping ?? [],
        expanded: currentState.expanded,
        rowPinning: currentState.rowPinning,
        rowSelection: currentState.rowSelection ?? {},
        columnVisibility: currentState.columnVisibility,
        columnSizing: currentState.columnSizing ?? {},
        columnSizingInfo: currentState.columnSizingInfo,
        columnPinning: currentState.columnPinning,
        columnOrder: currentState.columnOrder,
      },
    }
  }

  async function snapshotForAutoResetActionsWithFlush(
    options: TanStackOptions,
    state: TanStackState,
    actions: FixtureAction[],
  ): Promise<FixtureSnapshot["expect"]> {
    const { table, currentState } = buildTable(options, state)

    // Simulate an initial render pass that computes the full row model. TanStack's row-model
    // memo debug callbacks register auto-reset behavior without resetting on the first call.
    table.getRowModel?.()
    await Promise.resolve()
    await Promise.resolve()

    for (const action of actions) {
      if (action.type === "toggleSorting") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.toggleSorting(undefined, action.multi ?? false)
      } else if (action.type === "setColumnFilterValue") {
        const col = table.getColumn(action.column_id)
        if (!col) {
          throw new Error(`Unknown column in action: ${action.column_id}`)
        }
        col.setFilterValue(action.value)
      } else if (action.type === "setGlobalFilterValue") {
        table.setGlobalFilter(action.value)
      } else {
        throw new Error(
          `snapshotForAutoResetActionsWithFlush does not support action: ${action.type}`,
        )
      }

      // Simulate a render pass where derived models are recomputed. The core/filtered/sorted/grouped
      // row-model memo debug callbacks may queue `_autoResetPageIndex()` via `table._queue`.
      table.getRowModel?.()

      // Flush the microtask queue to run `table._queue` callbacks (including the reset itself).
      await Promise.resolve()
      await Promise.resolve()
    }

    const expect = snapshotForState(options, currentState)

    return {
      ...expect,
      next_state: {
        sorting: currentState.sorting ?? [],
        columnFilters: currentState.columnFilters ?? [],
        globalFilter: currentState.globalFilter,
        pagination: currentState.pagination,
        grouping: currentState.grouping ?? [],
        expanded: currentState.expanded,
        rowPinning: currentState.rowPinning,
        rowSelection: currentState.rowSelection ?? {},
        columnVisibility: currentState.columnVisibility,
        columnSizing: currentState.columnSizing ?? {},
        columnSizingInfo: currentState.columnSizingInfo,
        columnPinning: currentState.columnPinning,
        columnOrder: currentState.columnOrder,
      },
    }
  }

  const defaultOptions: TanStackOptions = {
    manualFiltering: false,
    manualSorting: false,
    manualPagination: false,
    manualExpanding: false,
    paginateExpandedRows: true,
    keepPinnedRows: true,
    enableSorting: true,
    enableMultiSort: true,
    enableSortingRemoval: true,
    enableMultiRemove: true,
  }

  const sortAscFirst: TanStackOptions = {
    ...defaultOptions,
    sortDescFirst: false,
  }

  let snapshots: FixtureSnapshot[]

  if (case_id === "demo_process") {
    const withSortingHelpers = (table: any) => ({
      sorting_helpers: snapshotSortingHelpers(table),
    })
    const mkExpectState = (options: TanStackOptions, state: TanStackState) =>
      snapshotForState(options, state, withSortingHelpers)
    const mkExpectActions = (
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => snapshotForActions(options, state, actions, withSortingHelpers)

    snapshots = [
      {
        id: "baseline",
        options: defaultOptions,
        state: {},
        expect: mkExpectState(defaultOptions, {}),
      },
      {
        id: "sorted_cpu_desc",
        options: defaultOptions,
        state: { sorting: [{ id: "cpu", desc: true }] },
        expect: mkExpectState(defaultOptions, { sorting: [{ id: "cpu", desc: true }] }),
      },
      {
        id: "sorted_cpu_desc_then_clear",
        options: defaultOptions,
        state: { sorting: [{ id: "cpu", desc: true }] },
        actions: [{ type: "clearSorting", column_id: "cpu" }],
        expect: mkExpectActions(
          defaultOptions,
          { sorting: [{ id: "cpu", desc: true }] },
          [{ type: "clearSorting", column_id: "cpu" }],
        ),
      },
      {
        id: "sorted_cpu_invert_asc",
        options: defaultOptions,
        state: { sorting: [{ id: "cpu_invert", desc: false }] },
        expect: mkExpectState(defaultOptions, {
          sorting: [{ id: "cpu_invert", desc: false }],
        }),
      },
      {
        id: "sorted_cpu_toggle_desc_first",
        options: defaultOptions,
        state: {},
        actions: [{ type: "toggleSorting", column_id: "cpu_desc_first" }],
        expect: mkExpectActions(defaultOptions, {}, [
          { type: "toggleSorting", column_id: "cpu_desc_first" },
        ]),
      },
      {
        id: "sorted_cpu_toggle_no_removal",
        options: {
          ...sortAscFirst,
          enableSortingRemoval: false,
        },
        state: {},
        actions: [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "cpu", multi: false },
        ],
        expect: mkExpectActions(
          {
            ...sortAscFirst,
            enableSortingRemoval: false,
          },
          {},
          [
            { type: "toggleSorting", column_id: "cpu", multi: false },
            { type: "toggleSorting", column_id: "cpu", multi: false },
            { type: "toggleSorting", column_id: "cpu", multi: false },
          ],
        ),
      },
      {
        id: "sorted_multi_max_1_keeps_latest",
        options: {
          ...sortAscFirst,
          maxMultiSortColCount: 1,
        },
        state: {},
        actions: [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "mem_mb", multi: true },
        ],
        expect: mkExpectActions(
          {
            ...sortAscFirst,
            maxMultiSortColCount: 1,
          },
          {},
          [
            { type: "toggleSorting", column_id: "cpu", multi: false },
            { type: "toggleSorting", column_id: "mem_mb", multi: true },
          ],
        ),
      },
      {
        id: "sorted_multi_max_2_drops_oldest",
        options: {
          ...sortAscFirst,
          maxMultiSortColCount: 2,
        },
        state: {},
        actions: [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "mem_mb", multi: true },
          { type: "toggleSorting", column_id: "status", multi: true },
        ],
        expect: mkExpectActions(
          {
            ...sortAscFirst,
            maxMultiSortColCount: 2,
          },
          {},
          [
            { type: "toggleSorting", column_id: "cpu", multi: false },
            { type: "toggleSorting", column_id: "mem_mb", multi: true },
            { type: "toggleSorting", column_id: "status", multi: true },
          ],
        ),
      },
      {
        id: "sorted_multi_disabled_replaces",
        options: {
          ...sortAscFirst,
          enableMultiSort: false,
        },
        state: {},
        actions: [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "mem_mb", multi: true },
        ],
        expect: mkExpectActions(
          {
            ...sortAscFirst,
            enableMultiSort: false,
          },
          {},
          [
            { type: "toggleSorting", column_id: "cpu", multi: false },
            { type: "toggleSorting", column_id: "mem_mb", multi: true },
          ],
        ),
      },
      {
        id: "sorted_multi_column_disabled_replaces",
        options: sortAscFirst,
        state: {},
        actions: [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "cpu_no_multi", multi: true },
        ],
        expect: mkExpectActions(sortAscFirst, {}, [
          { type: "toggleSorting", column_id: "cpu", multi: false },
          { type: "toggleSorting", column_id: "cpu_no_multi", multi: true },
        ]),
      },
      {
        id: "sorted_handler_table_sorting_disabled_noop",
        options: {
          ...sortAscFirst,
          enableSorting: false,
        },
        state: {},
        actions: [{ type: "toggleSortingHandler", column_id: "cpu" }],
        expect: mkExpectActions(
          {
            ...sortAscFirst,
            enableSorting: false,
          },
          {},
          [{ type: "toggleSortingHandler", column_id: "cpu" }],
        ),
      },
      {
        id: "sorted_handler_column_sorting_disabled_noop",
        options: sortAscFirst,
        state: {},
        actions: [{ type: "toggleSortingHandler", column_id: "cpu_no_sort" }],
        expect: mkExpectActions(sortAscFirst, {}, [
          { type: "toggleSortingHandler", column_id: "cpu_no_sort" },
        ]),
      },
      {
        id: "sorted_handler_multi_event_adds_when_allowed",
        options: sortAscFirst,
        state: { sorting: [{ id: "cpu", desc: false }] },
        actions: [
          { type: "toggleSortingHandler", column_id: "mem_mb", event_multi: true },
        ],
        expect: mkExpectActions(sortAscFirst, { sorting: [{ id: "cpu", desc: false }] }, [
          { type: "toggleSortingHandler", column_id: "mem_mb", event_multi: true },
        ]),
      },
      {
        id: "filter_status_run",
        options: defaultOptions,
        state: { columnFilters: [{ id: "status", value: "run" }] },
        expect: mkExpectState(defaultOptions, {
          columnFilters: [{ id: "status", value: "run" }],
        }),
      },
      {
        id: "page_0_size_2",
        options: defaultOptions,
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        expect: mkExpectState(defaultOptions, {
          pagination: { pageIndex: 0, pageSize: 2 },
        }),
      },
    ]
  } else if (case_id === "auto_reset") {
    const mkActionsAutoReset = async (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = await snapshotForAutoResetActionsWithFlush(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      return {
        id,
        options,
        state,
        actions,
        expect,
      }
    }

    snapshots = [
      await mkActionsAutoReset(
        "auto_reset_sorting_default_resets",
        {},
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "toggleSorting", column_id: "cpu_desc_first", multi: false }],
      ),
      await mkActionsAutoReset(
        "auto_reset_sorting_manual_pagination_true_no_reset",
        { manualPagination: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "toggleSorting", column_id: "cpu_desc_first", multi: false }],
      ),
      await mkActionsAutoReset(
        "auto_reset_sorting_manual_pagination_true_auto_reset_page_index_true_overrides_manual",
        { manualPagination: true, autoResetPageIndex: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "toggleSorting", column_id: "cpu_desc_first", multi: false }],
      ),
      await mkActionsAutoReset(
        "auto_reset_sorting_auto_reset_all_false_disables",
        { autoResetAll: false },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "toggleSorting", column_id: "cpu_desc_first", multi: false }],
      ),
      await mkActionsAutoReset(
        "auto_reset_global_filter_default_resets",
        {},
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "setGlobalFilterValue", value: "Renderer" }],
      ),
      await mkActionsAutoReset(
        "auto_reset_global_filter_manual_pagination_true_no_reset",
        { manualPagination: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "setGlobalFilterValue", value: "Renderer" }],
      ),
      await mkActionsAutoReset(
        "auto_reset_global_filter_manual_pagination_true_auto_reset_page_index_true_overrides_manual",
        { manualPagination: true, autoResetPageIndex: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "setGlobalFilterValue", value: "Renderer" }],
      ),
      await mkActionsAutoReset(
        "auto_reset_global_filter_auto_reset_all_false_disables",
        { autoResetAll: false },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [{ type: "setGlobalFilterValue", value: "Renderer" }],
      ),
    ]
  } else if (case_id === "resets") {
    const options: TanStackOptions = defaultOptions

    const baseInitialState: Partial<TanStackState> = {
      sorting: [{ id: "cpu", desc: true }],
      columnFilters: [{ id: "status", value: "Running" }],
      globalFilter: "Renderer",
      grouping: ["status"],
      rowSelection: { "1": true, "3": true },
      columnVisibility: { cpu: false },
      columnOrder: ["mem_mb", "name", "status", "cpu"],
    }

    const state: TanStackState = {
      sorting: [{ id: "mem_mb", desc: false }],
      columnFilters: [{ id: "status", value: "Idle" }],
      globalFilter: "Idle",
      grouping: ["name"],
      rowSelection: { "2": true },
      columnVisibility: { status: false },
      columnOrder: ["name", "cpu", "status", "mem_mb"],
    }

    const mk = (id: SnapshotId, actions: FixtureAction[]) => ({
      id,
      options: { ...options, initialState: baseInitialState },
      state,
      actions,
      expect: snapshotForActions(
        { ...options, initialState: baseInitialState },
        state,
        actions,
      ),
    })

    snapshots = [
      mk("resets_reset_sorting_restores_initial", [
        { type: "resetSorting", default_state: false },
      ]),
      mk("resets_reset_sorting_default_true_clears", [
        { type: "resetSorting", default_state: true },
      ]),
      mk("resets_reset_column_filters_restores_initial", [
        { type: "resetColumnFilters", default_state: false },
      ]),
      mk("resets_reset_column_filters_default_true_clears", [
        { type: "resetColumnFilters", default_state: true },
      ]),
      mk("resets_reset_global_filter_restores_initial", [
        { type: "resetGlobalFilter", default_state: false },
      ]),
      mk("resets_reset_global_filter_default_true_clears", [
        { type: "resetGlobalFilter", default_state: true },
      ]),
      mk("resets_reset_grouping_restores_initial", [
        { type: "resetGrouping", default_state: false },
      ]),
      mk("resets_reset_grouping_default_true_clears", [
        { type: "resetGrouping", default_state: true },
      ]),
      mk("resets_reset_column_visibility_restores_initial", [
        { type: "resetColumnVisibility", default_state: false },
      ]),
      mk("resets_reset_column_visibility_default_true_clears", [
        { type: "resetColumnVisibility", default_state: true },
      ]),
      mk("resets_reset_column_order_restores_initial", [
        { type: "resetColumnOrder", default_state: false },
      ]),
      mk("resets_reset_column_order_default_true_clears", [
        { type: "resetColumnOrder", default_state: true },
      ]),
      mk("resets_reset_row_selection_restores_initial", [
        { type: "resetRowSelection", default_state: false },
      ]),
      mk("resets_reset_row_selection_default_true_clears", [
        { type: "resetRowSelection", default_state: true },
      ]),
    ]
  } else if (case_id === "pagination") {
    const base = defaultOptions
    snapshots = [
      {
        id: "pagination_baseline",
        options: base,
        state: {},
        expect: snapshotForState(base, {}),
      },
      {
        id: "pagination_set_page_index_out_of_range_uncontrolled",
        options: base,
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        actions: [{ type: "setPageIndex", page_index: 10 }],
        expect: snapshotForActions(base, { pagination: { pageIndex: 0, pageSize: 2 } }, [
          { type: "setPageIndex", page_index: 10 },
        ]),
      },
      {
        id: "pagination_set_page_index_clamps_when_page_count_is_set",
        options: { ...base, pageCount: 2 },
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        actions: [{ type: "setPageIndex", page_index: 10 }],
        expect: snapshotForActions({ ...base, pageCount: 2 }, { pagination: { pageIndex: 0, pageSize: 2 } }, [
          { type: "setPageIndex", page_index: 10 },
        ]),
      },
      {
        id: "pagination_set_page_size_recomputes_page_index",
        options: base,
        state: { pagination: { pageIndex: 2, pageSize: 2 } },
        actions: [{ type: "setPageSize", page_size: 3 }],
        expect: snapshotForActions(base, { pagination: { pageIndex: 2, pageSize: 2 } }, [
          { type: "setPageSize", page_size: 3 },
        ]),
      },
      {
        id: "pagination_manual_pagination_true_returns_pre_pagination",
        options: { ...base, manualPagination: true },
        state: { pagination: { pageIndex: 1, pageSize: 2 } },
        expect: snapshotForState({ ...base, manualPagination: true }, { pagination: { pageIndex: 1, pageSize: 2 } }),
      },
      {
        id: "pagination_on_pagination_change_noop_ignores",
        options: { ...base, __onPaginationChange: "noop" },
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        actions: [{ type: "setPageIndex", page_index: 1 }],
        expect: snapshotForActions({ ...base, __onPaginationChange: "noop" }, { pagination: { pageIndex: 0, pageSize: 2 } }, [
          { type: "setPageIndex", page_index: 1 },
        ]),
      },
      {
        id: "pagination_page_count_minus_one_allows_next",
        options: { ...base, pageCount: -1 },
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        expect: snapshotForState({ ...base, pageCount: -1 }, { pagination: { pageIndex: 0, pageSize: 2 } }),
      },
      {
        id: "pagination_row_count_infers_page_count",
        options: { ...base, rowCount: 100 },
        state: { pagination: { pageIndex: 0, pageSize: 10 } },
        expect: snapshotForState({ ...base, rowCount: 100 }, { pagination: { pageIndex: 0, pageSize: 10 } }),
      },
      {
        id: "pagination_override_get_pagination_row_model_pre_pagination",
        options: { ...base, __getPaginationRowModel: "pre_pagination" },
        state: { pagination: { pageIndex: 1, pageSize: 2 } },
        expect: snapshotForState(
          { ...base, __getPaginationRowModel: "pre_pagination" },
          { pagination: { pageIndex: 1, pageSize: 2 } },
        ),
      },
    ]
  } else if (case_id === "state_shapes") {
    snapshots = [
      {
        id: "state_shapes_baseline",
        options: defaultOptions,
        state: {},
        expect: snapshotForState(defaultOptions, {}),
      },
      {
        id: "state_shapes_grouping_two_columns",
        options: defaultOptions,
        state: { grouping: ["status", "cpu"] },
        expect: snapshotForState(defaultOptions, { grouping: ["status", "cpu"] }),
      },
      {
        id: "state_shapes_expanded_all",
        options: defaultOptions,
        state: { expanded: true },
        expect: snapshotForState(defaultOptions, { expanded: true }),
      },
      {
        id: "state_shapes_expanded_map",
        options: defaultOptions,
        state: { expanded: { "1": true, "3": true } },
        expect: snapshotForState(defaultOptions, { expanded: { "1": true, "3": true } }),
      },
      {
        id: "state_shapes_row_pinning_top_bottom",
        options: defaultOptions,
        state: { rowPinning: { top: ["1", "2"], bottom: ["5"] } },
        expect: snapshotForState(defaultOptions, {
          rowPinning: { top: ["1", "2"], bottom: ["5"] },
        }),
      },
      {
        id: "state_shapes_global_filter_json",
        options: defaultOptions,
        state: { globalFilter: { kind: "x", n: 1 } },
        expect: snapshotForState(defaultOptions, { globalFilter: { kind: "x", n: 1 } }),
      },
    ]
  } else if (case_id === "selection") {
    const base = defaultOptions
    snapshots = [
      {
        id: "selection_baseline",
        options: base,
        state: {},
        expect: snapshotForState(base, {}),
      },
      {
        id: "selection_state_two_rows",
        options: base,
        state: { rowSelection: { "1": true, "3": true } },
        expect: snapshotForState(base, { rowSelection: { "1": true, "3": true } }),
      },
      {
        id: "selection_filtered_selected_intersects",
        options: base,
        state: {
          rowSelection: { "1": true, "2": true },
          columnFilters: [{ id: "status", value: "Running" }],
        },
        expect: snapshotForState(base, {
          rowSelection: { "1": true, "2": true },
          columnFilters: [{ id: "status", value: "Running" }],
        }),
      },
      {
        id: "selection_toggle_row_multi_disabled_keeps_latest",
        options: { ...base, enableMultiRowSelection: false },
        state: {},
        actions: [
          { type: "toggleRowSelected", row_id: "1" },
          { type: "toggleRowSelected", row_id: "3" },
        ],
        expect: snapshotForActions({ ...base, enableMultiRowSelection: false }, {}, [
          { type: "toggleRowSelected", row_id: "1" },
          { type: "toggleRowSelected", row_id: "3" },
        ]),
      },
      {
        id: "selection_toggle_all_rows_disabled_noop",
        options: { ...base, enableRowSelection: false },
        state: {},
        actions: [{ type: "toggleAllRowsSelected" }],
        expect: snapshotForActions({ ...base, enableRowSelection: false }, {}, [
          { type: "toggleAllRowsSelected" },
        ]),
      },
      {
        id: "selection_toggle_all_page_rows_respects_pagination",
        options: base,
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        actions: [{ type: "toggleAllPageRowsSelected" }],
        expect: snapshotForActions(base, { pagination: { pageIndex: 0, pageSize: 2 } }, [
          { type: "toggleAllPageRowsSelected" },
        ]),
      },
      {
        id: "selection_enable_row_selection_fn_odd_ids_toggle_all_rows_selects_selectable",
        options: { ...base, __enableRowSelection: "odd_ids" },
        state: {},
        actions: [{ type: "toggleAllRowsSelected" }],
        expect: snapshotForActions({ ...base, __enableRowSelection: "odd_ids" }, {}, [
          { type: "toggleAllRowsSelected" },
        ]),
      },
      {
        id: "selection_enable_row_selection_fn_odd_ids_toggle_row_unselectable_noop",
        options: { ...base, __enableRowSelection: "odd_ids" },
        state: {},
        actions: [{ type: "toggleRowSelected", row_id: "2" }],
        expect: snapshotForActions({ ...base, __enableRowSelection: "odd_ids" }, {}, [
          { type: "toggleRowSelected", row_id: "2" },
        ]),
      },
      {
        id: "selection_enable_row_selection_fn_odd_ids_toggle_all_page_rows_selects_selectable",
        options: { ...base, __enableRowSelection: "odd_ids" },
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        actions: [{ type: "toggleAllPageRowsSelected" }],
        expect: snapshotForActions(
          { ...base, __enableRowSelection: "odd_ids" },
          { pagination: { pageIndex: 0, pageSize: 2 } },
          [{ type: "toggleAllPageRowsSelected" }],
        ),
      },
    ]
  } else if (case_id === "selection_tree") {
    const base = defaultOptions
    const rowIds = ["1", "11", "12", "121", "2", "3", "31", "4"]

    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const baseSnap = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...baseSnap,
          row_selection_detail: snapshotRowSelectionDetail(table, rowIds),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          row_selection_detail: snapshotRowSelectionDetail(table, rowIds),
        },
      }
    }

    snapshots = [
      mk("selection_tree_baseline", base, {}),
      mk("selection_tree_state_child_selected_marks_parent_some_selected", base, {
        rowSelection: { "11": true },
      }),
      mk(
        "selection_tree_state_all_children_selected_marks_parent_all_sub_rows_selected",
        base,
        { rowSelection: { "11": true, "12": true, "121": true } },
      ),
      mkActions("selection_tree_action_toggle_root_selects_children_default", base, {}, [
        { type: "toggleRowSelected", row_id: "1" },
      ]),
      mkActions(
        "selection_tree_action_toggle_root_select_children_false_only_root",
        base,
        {},
        [{ type: "toggleRowSelected", row_id: "1", select_children: false }],
      ),
      mkActions(
        "selection_tree_action_toggle_root_enable_sub_row_selection_false_only_root",
        { ...base, enableSubRowSelection: false },
        {},
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
      mkActions(
        "selection_tree_action_toggle_root_enable_multi_row_selection_false_clears_previous",
        { ...base, enableMultiRowSelection: false },
        { rowSelection: { "2": true } },
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
      mkActions(
        "selection_tree_action_toggle_on_row_selection_change_noop_ignores",
        { ...base, __onRowSelectionChange: "noop" },
        {},
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
      mkActions(
        "selection_tree_enable_row_selection_fn_except_11_root_all_sub_rows_selected",
        { ...base, __enableRowSelection: "except_11" },
        {},
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
      mkActions(
        "selection_tree_enable_sub_row_selection_fn_disable_root_1_only_root",
        { ...base, __enableSubRowSelection: "disable_root_1" },
        {},
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
      mkActions(
        "selection_tree_enable_multi_row_selection_fn_always_false_clears_previous",
        { ...base, __enableMultiRowSelection: "always_false" },
        { rowSelection: { "2": true } },
        [{ type: "toggleRowSelected", row_id: "1" }],
      ),
    ]
  } else if (case_id === "expanding") {
    const base = defaultOptions
    snapshots = [
      {
        id: "expanding_baseline",
        options: base,
        state: {},
        expect: snapshotForState(base, {}),
      },
      {
        id: "expanding_enable_expanding_false_disables_can_expand",
        options: { ...base, enableExpanding: false },
        state: {},
        expect: snapshotForState({ ...base, enableExpanding: false }, {}),
      },
      {
        id: "expanding_hook_get_row_can_expand_overrides_enable_expanding_false",
        options: { ...base, enableExpanding: false, __getRowCanExpand: "only_root_1" },
        state: {},
        expect: snapshotForState(
          { ...base, enableExpanding: false, __getRowCanExpand: "only_root_1" },
          {},
        ),
      },
      {
        id: "expanding_state_row_1",
        options: base,
        state: { expanded: { "1": true } },
        expect: snapshotForState(base, { expanded: { "1": true } }),
      },
      {
        id: "expanding_override_get_expanded_row_model_pre_expanded",
        options: { ...base, __getExpandedRowModel: "pre_expanded" },
        state: { expanded: { "1": true } },
        expect: snapshotForState(
          { ...base, __getExpandedRowModel: "pre_expanded" },
          { expanded: { "1": true } },
        ),
      },
      {
        id: "expanding_hook_get_is_row_expanded_overrides_state",
        options: { ...base, __getIsRowExpanded: "always_false" },
        state: { expanded: { "1": true } },
        expect: snapshotForState(
          { ...base, __getIsRowExpanded: "always_false" },
          { expanded: { "1": true } },
        ),
      },
      {
        id: "expanding_state_all_true",
        options: base,
        state: { expanded: true },
        expect: snapshotForState(base, { expanded: true }),
      },
      {
        id: "expanding_paginate_expanded_rows_true_counts_children",
        options: base,
        state: { expanded: { "1": true }, pagination: { pageIndex: 0, pageSize: 2 } },
        expect: snapshotForState(base, {
          expanded: { "1": true },
          pagination: { pageIndex: 0, pageSize: 2 },
        }),
      },
      {
        id: "expanding_paginate_expanded_rows_false_expands_within_page",
        options: { ...base, paginateExpandedRows: false },
        state: { expanded: { "1": true }, pagination: { pageIndex: 0, pageSize: 2 } },
        expect: snapshotForState(
          { ...base, paginateExpandedRows: false },
          { expanded: { "1": true }, pagination: { pageIndex: 0, pageSize: 2 } },
        ),
      },
      {
        id: "expanding_action_toggle_row",
        options: base,
        state: {},
        actions: [
          { type: "toggleRowExpanded", row_id: "1" },
          { type: "toggleRowExpanded", row_id: "1" },
        ],
        expect: snapshotForActions(base, {}, [
          { type: "toggleRowExpanded", row_id: "1" },
          { type: "toggleRowExpanded", row_id: "1" },
        ]),
      },
      {
        id: "expanding_action_toggle_row_on_expanded_change_noop_ignores",
        options: { ...base, __onExpandedChange: "noop" },
        state: {},
        actions: [{ type: "toggleRowExpanded", row_id: "1" }],
        expect: snapshotForActions({ ...base, __onExpandedChange: "noop" }, {}, [
          { type: "toggleRowExpanded", row_id: "1" },
        ]),
      },
      {
        id: "expanding_action_toggle_row_enable_expanding_false_still_updates_state",
        options: { ...base, enableExpanding: false },
        state: {},
        actions: [{ type: "toggleRowExpanded", row_id: "1" }],
        expect: snapshotForActions({ ...base, enableExpanding: false }, {}, [
          { type: "toggleRowExpanded", row_id: "1" },
        ]),
      },
      {
        id: "expanding_action_toggle_all",
        options: base,
        state: {},
        actions: [{ type: "toggleAllRowsExpanded" }, { type: "toggleAllRowsExpanded" }],
        expect: snapshotForActions(base, {}, [
          { type: "toggleAllRowsExpanded" },
          { type: "toggleAllRowsExpanded" },
        ]),
      },
    ]
  } else if (case_id === "row_id_state_ops") {
    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      return {
        id,
        options,
        state,
        actions,
        expect,
      }
    }

    snapshots = [
      mkActions(
        "row_id_state_ops_leaf_selection_prefixed",
        { __getRowId: "prefixed" },
        {},
        [{ type: "toggleRowSelected", row_id: "row:1", value: true }],
      ),
      mkActions(
        "row_id_state_ops_group_selection",
        {},
        { grouping: ["status"] },
        [{ type: "toggleRowSelected", row_id: "status:Running", value: true }],
      ),
      mkActions(
        "row_id_state_ops_group_selection_select_children_false",
        {},
        { grouping: ["status"] },
        [
          {
            type: "toggleRowSelected",
            row_id: "status:Running",
            value: true,
            select_children: false,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_group_selection_toggle_off",
        {},
        { grouping: ["status"] },
        [
          { type: "toggleRowSelected", row_id: "status:Running", value: true },
          { type: "toggleRowSelected", row_id: "status:Running", value: false },
        ],
      ),
      mkActions(
        "row_id_state_ops_nested_group_selection",
        {},
        { grouping: ["status", "name"] },
        [
          {
            type: "toggleRowSelected",
            row_id: "status:Running>name:Renderer",
            value: true,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_group_selection_on_row_selection_change_noop",
        { __onRowSelectionChange: "noop" },
        { grouping: ["status"] },
        [{ type: "toggleRowSelected", row_id: "status:Running", value: true }],
      ),
      mkActions(
        "row_id_state_ops_group_expanding",
        {},
        { grouping: ["status"] },
        [{ type: "toggleRowExpanded", row_id: "status:Running", value: true }],
      ),
      mkActions(
        "row_id_state_ops_group_expanding_on_expanded_change_noop",
        { __onExpandedChange: "noop" },
        { grouping: ["status"] },
        [{ type: "toggleRowExpanded", row_id: "status:Running", value: true }],
      ),
      mkActions(
        "row_id_state_ops_group_pinning",
        {},
        { grouping: ["status"] },
        [
          {
            type: "pinRow",
            row_id: "status:Running",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: false,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_group_pinning_on_row_pinning_change_noop",
        { __onRowPinningChange: "noop" },
        { grouping: ["status"] },
        [
          {
            type: "pinRow",
            row_id: "status:Running",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: false,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_nested_group_pinning",
        {},
        { grouping: ["status", "name"] },
        [
          {
            type: "pinRow",
            row_id: "status:Running>name:Renderer",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: true,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_group_mixed_select_expand_pin",
        {},
        { grouping: ["status"] },
        [
          { type: "toggleRowSelected", row_id: "status:Running", value: true },
          { type: "toggleRowExpanded", row_id: "status:Running", value: true },
          {
            type: "pinRow",
            row_id: "status:Running",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: false,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_nested_group_mixed_select_expand_pin",
        {},
        { grouping: ["status", "name"] },
        [
          {
            type: "toggleRowSelected",
            row_id: "status:Running>name:Renderer",
            value: true,
          },
          { type: "toggleRowExpanded", row_id: "status:Running", value: true },
          {
            type: "pinRow",
            row_id: "status:Running>name:Renderer",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: true,
          },
        ],
      ),
      mkActions(
        "row_id_state_ops_group_mixed_selection_noop_expand_pin",
        { __onRowSelectionChange: "noop" },
        { grouping: ["status"] },
        [
          { type: "toggleRowSelected", row_id: "status:Running", value: true },
          { type: "toggleRowExpanded", row_id: "status:Running", value: true },
          {
            type: "pinRow",
            row_id: "status:Running",
            position: "top",
            include_leaf_rows: false,
            include_parent_rows: false,
          },
        ],
      ),
    ]
  } else if (case_id === "sorting_fns") {
    snapshots = [
      {
        id: "sorting_fns_builtin_basic",
        options: defaultOptions,
        state: { sorting: [{ id: "num_basic", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "num_basic", desc: false }],
        }),
      },
      {
        id: "sorting_fns_builtin_basic_desc",
        options: defaultOptions,
        state: { sorting: [{ id: "num_basic", desc: true }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "num_basic", desc: true }],
        }),
      },
      {
        id: "sorting_fns_builtin_datetime",
        options: defaultOptions,
        state: { sorting: [{ id: "dt_datetime", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "dt_datetime", desc: false }],
        }),
      },
      {
        id: "sorting_fns_builtin_datetime_desc",
        options: defaultOptions,
        state: { sorting: [{ id: "dt_datetime", desc: true }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "dt_datetime", desc: true }],
        }),
      },
      {
        id: "sorting_fns_builtin_text",
        options: defaultOptions,
        state: { sorting: [{ id: "text_text", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "text_text", desc: false }],
        }),
      },
      {
        id: "sorting_fns_builtin_text_case_sensitive",
        options: defaultOptions,
        state: { sorting: [{ id: "text_text_cs", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "text_text_cs", desc: false }],
        }),
      },
      {
        id: "sorting_fns_builtin_alphanumeric",
        options: defaultOptions,
        state: { sorting: [{ id: "alpha_alphanumeric", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "alpha_alphanumeric", desc: false }],
        }),
      },
      {
        id: "sorting_fns_builtin_alphanumeric_case_sensitive",
        options: defaultOptions,
        state: { sorting: [{ id: "alpha_alphanumeric_cs", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "alpha_alphanumeric_cs", desc: false }],
        }),
      },
      {
        id: "sorting_fns_auto_basic",
        options: defaultOptions,
        state: { sorting: [{ id: "num_auto", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "num_auto", desc: false }],
        }),
      },
      {
        id: "sorting_fns_auto_datetime",
        options: defaultOptions,
        state: { sorting: [{ id: "dt_auto", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "dt_auto", desc: false }],
        }),
      },
      {
        id: "sorting_fns_auto_text",
        options: defaultOptions,
        state: { sorting: [{ id: "text_auto", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "text_auto", desc: false }],
        }),
      },
      {
        id: "sorting_fns_auto_alphanumeric",
        options: defaultOptions,
        state: { sorting: [{ id: "alpha_auto", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "alpha_auto", desc: false }],
        }),
      },
      {
        id: "sorting_fns_registry_custom_text",
        options: { ...defaultOptions, sortingFnsMode: "custom_text" },
        state: { sorting: [{ id: "text_custom", desc: false }] },
        expect: snapshotForState(
          { ...defaultOptions, sortingFnsMode: "custom_text" },
          {
            sorting: [{ id: "text_custom", desc: false }],
          },
        ),
      },
      {
        id: "sorting_fns_toggle_num_auto_first",
        options: defaultOptions,
        state: {},
        actions: [{ type: "toggleSorting", column_id: "num_auto" }],
        expect: snapshotForActions(defaultOptions, {}, [
          { type: "toggleSorting", column_id: "num_auto" },
        ]),
      },
      {
        id: "sorting_fns_toggle_text_auto_first",
        options: defaultOptions,
        state: {},
        actions: [{ type: "toggleSorting", column_id: "text_auto" }],
        expect: snapshotForActions(defaultOptions, {}, [
          { type: "toggleSorting", column_id: "text_auto" },
        ]),
      },
      {
        id: "sorting_fns_toggle_dt_auto_first",
        options: defaultOptions,
        state: {},
        actions: [{ type: "toggleSorting", column_id: "dt_auto" }],
        expect: snapshotForActions(defaultOptions, {}, [
          { type: "toggleSorting", column_id: "dt_auto" },
        ]),
      },
    ]
  } else if (case_id === "filtering_fns") {
    const base = defaultOptions
    const withFilteringHelpers = (table: any) => ({
      filtering_helpers: snapshotFilteringHelpers(table),
    })
    const mkExpectState = (options: TanStackOptions, state: TanStackState) =>
      snapshotForState(options, state, withFilteringHelpers)
    const mkExpectActions = (
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => snapshotForActions(options, state, actions, withFilteringHelpers)

    snapshots = [
      {
        id: "filtering_fns_text_auto_includes",
        options: base,
        state: { columnFilters: [{ id: "text_auto", value: "ap" }] },
        expect: mkExpectState(base, { columnFilters: [{ id: "text_auto", value: "ap" }] }),
      },
      {
        id: "filtering_fns_text_equals_string",
        options: base,
        state: { columnFilters: [{ id: "text_equals_string", value: "banana" }] },
        expect: mkExpectState(base, {
          columnFilters: [{ id: "text_equals_string", value: "banana" }],
        }),
      },
      {
        id: "filtering_fns_num_in_number_range",
        options: base,
        state: { columnFilters: [{ id: "num_range", value: [4, 8] }] },
        expect: mkExpectState(base, {
          columnFilters: [{ id: "num_range", value: [4, 8] }],
        }),
      },
      {
        id: "filtering_fns_tags_arr_includes_all",
        options: base,
        state: { columnFilters: [{ id: "tags_all", value: ["a", "b"] }] },
        expect: mkExpectState(base, {
          columnFilters: [{ id: "tags_all", value: ["a", "b"] }],
        }),
      },
      {
        id: "filtering_fns_bool_equals",
        options: base,
        state: { columnFilters: [{ id: "flag_equals", value: true }] },
        expect: mkExpectState(base, {
          columnFilters: [{ id: "flag_equals", value: true }],
        }),
      },
      {
        id: "filtering_fns_weak_equals_string_number",
        options: base,
        state: { columnFilters: [{ id: "num_weak", value: "5" }] },
        expect: mkExpectState(base, {
          columnFilters: [{ id: "num_weak", value: "5" }],
        }),
      },
      {
        id: "filtering_fns_global_filter_includes",
        options: base,
        state: { globalFilter: "ap" },
        expect: mkExpectState(base, { globalFilter: "ap" }),
      },
      {
        id: "filtering_fns_global_filter_default_excludes_bool",
        options: base,
        state: { globalFilter: "true" },
        expect: mkExpectState(base, { globalFilter: "true" }),
      },
      {
        id: "filtering_fns_global_filter_disabled_when_enable_filters_false",
        options: { ...base, enableFilters: false },
        state: { globalFilter: "ap" },
        expect: mkExpectState({ ...base, enableFilters: false }, { globalFilter: "ap" }),
      },
      {
        id: "filtering_fns_column_filters_disabled_when_enable_column_filters_false",
        options: { ...base, enableColumnFilters: false },
        state: { columnFilters: [{ id: "text_auto", value: "ap" }] },
        expect: mkExpectState(
          { ...base, enableColumnFilters: false },
          { columnFilters: [{ id: "text_auto", value: "ap" }] },
        ),
      },
      {
        id: "filtering_fns_registry_custom_text_case_sensitive",
        options: { ...base, filterFnsMode: "custom_text_case_sensitive" },
        state: { columnFilters: [{ id: "text_custom", value: "A" }] },
        expect: mkExpectState(
          { ...base, filterFnsMode: "custom_text_case_sensitive" },
          { columnFilters: [{ id: "text_custom", value: "A" }] },
        ),
      },
      {
        id: "filtering_fns_action_set_empty_removes",
        options: base,
        state: { columnFilters: [{ id: "text_auto", value: "ap" }] },
        actions: [{ type: "setColumnFilterValue", column_id: "text_auto", value: "" }],
        expect: mkExpectActions(base, { columnFilters: [{ id: "text_auto", value: "ap" }] }, [
          { type: "setColumnFilterValue", column_id: "text_auto", value: "" },
        ]),
      },
      {
        id: "filtering_fns_action_set_column_filter_noop_hook_ignores",
        options: { ...base, __onColumnFiltersChange: "noop" },
        state: {},
        actions: [{ type: "setColumnFilterValue", column_id: "text_auto", value: "ap" }],
        expect: mkExpectActions({ ...base, __onColumnFiltersChange: "noop" }, {}, [
          { type: "setColumnFilterValue", column_id: "text_auto", value: "ap" },
        ]),
      },
      {
        id: "filtering_fns_action_set_global_filter_noop_hook_ignores",
        options: { ...base, __onGlobalFilterChange: "noop" },
        state: {},
        actions: [{ type: "setGlobalFilterValue", value: "ap" }],
        expect: mkExpectActions({ ...base, __onGlobalFilterChange: "noop" }, {}, [
          { type: "setGlobalFilterValue", value: "ap" },
        ]),
      },
    ]
  } else if (case_id === "headers_cells") {
    const base = defaultOptions
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const { table } = buildTable(options, state)

      const baseExpect: FixtureSnapshot["expect"] = {
        core: snapshotRowModel(table.getCoreRowModel()),
        filtered: snapshotRowModel(table.getFilteredRowModel()),
        sorted: snapshotRowModel(table.getSortedRowModel()),
        paginated: snapshotRowModel(table.getPaginationRowModel()),
        row_model: snapshotRowModel(table.getRowModel()),
      }

      const header_groups = snapshotHeaderGroups(table.getHeaderGroups())
      const left_header_groups = snapshotHeaderGroups(table.getLeftHeaderGroups())
      const center_header_groups = snapshotHeaderGroups(table.getCenterHeaderGroups())
      const right_header_groups = snapshotHeaderGroups(table.getRightHeaderGroups())
      const footer_groups = snapshotHeaderGroups(table.getFooterGroups())
      const left_footer_groups = snapshotHeaderGroups(table.getLeftFooterGroups())
      const center_footer_groups = snapshotHeaderGroups(table.getCenterFooterGroups())
      const right_footer_groups = snapshotHeaderGroups(table.getRightFooterGroups())
      const flat_headers = snapshotHeaders(table.getFlatHeaders())
      const left_flat_headers = snapshotHeaders(table.getLeftFlatHeaders())
      const center_flat_headers = snapshotHeaders(table.getCenterFlatHeaders())
      const right_flat_headers = snapshotHeaders(table.getRightFlatHeaders())
      const leaf_headers = snapshotHeaders(table.getLeafHeaders())
      const left_leaf_headers = snapshotHeaders(table.getLeftLeafHeaders())
      const center_leaf_headers = snapshotHeaders(table.getCenterLeafHeaders())
      const right_leaf_headers = snapshotHeaders(table.getRightLeafHeaders())
      const cells = snapshotCells(table)

      return {
        id,
        options,
        state,
        expect: {
          ...baseExpect,
          headers_cells: {
            header_groups,
            footer_groups,
            left_header_groups,
            left_footer_groups,
            center_header_groups,
            center_footer_groups,
            right_header_groups,
            right_footer_groups,
            flat_headers,
            left_flat_headers,
            center_flat_headers,
            right_flat_headers,
            leaf_headers,
            left_leaf_headers,
            center_leaf_headers,
            right_leaf_headers,
            cells,
          },
          flat_columns: {
            all: (table.getAllFlatColumns?.() ?? []).map((c: any) => String(c.id)),
            visible: (table.getVisibleFlatColumns?.() ?? []).map((c: any) => String(c.id)),
          },
          core_model: {
            column_tree: snapshotColumnTree(table.getAllColumns()),
            leaf_columns: {
              all: (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id)),
              visible: (table.getVisibleLeafColumns?.() ?? []).map((c: any) => String(c.id)),
              left_visible: (table.getLeftVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
              center_visible: (table.getCenterVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
              right_visible: (table.getRightVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
            },
            header_groups,
            left_header_groups,
            center_header_groups,
            right_header_groups,
            rows: {
              core: snapshotRowModel(table.getCoreRowModel()),
              row_model: snapshotRowModel(table.getRowModel()),
            },
            cells,
          },
        },
      }
    }

    snapshots = [
      mk("baseline", base, { columnPinning: { left: ["name"], right: ["mem_mb"] } }),
      mk("headers_cells_order_and_pinning", base, {
        columnOrder: ["mem_mb", "cpu", "name"],
        columnPinning: { left: ["cpu"], right: [] },
      }),
      mk("headers_cells_hide_right_leaf", base, {
        columnPinning: { left: ["name"], right: ["mem_mb"] },
        columnVisibility: { mem_mb: false },
      }),
      mk("headers_cells_hide_left_leaf", base, {
        columnPinning: { left: ["cpu"], right: ["mem_mb"] },
        columnVisibility: { cpu: false },
      }),
      mk("headers_cells_column_order_reorders", base, {
        columnOrder: ["cpu", "name", "mem_mb"],
      }),
      mk(
        "headers_cells_grouped_column_mode_reorder_moves_grouped_first",
        { ...base, groupedColumnMode: "reorder" },
        {
        grouping: ["mem_mb"],
        },
      ),
      mk("headers_cells_grouped_column_mode_remove_hides_grouped_column", { ...base, groupedColumnMode: "remove" }, {
        grouping: ["cpu"],
      }),
      mk("headers_cells_grouped_column_mode_remove_drops_pinned_grouped_column", { ...base, groupedColumnMode: "remove" }, {
        grouping: ["cpu"],
        columnPinning: { left: ["cpu"], right: [] },
      }),
      mk(
        "headers_cells_grouped_column_mode_reorder_respects_column_order_after_grouping",
        { ...base, groupedColumnMode: "reorder" },
        {
        columnOrder: ["mem_mb", "cpu", "name"],
        grouping: ["cpu"],
        },
      ),
    ]
  } else if (case_id === "headers_inventory_deep") {
    const base = defaultOptions
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const { table } = buildTable(options, state)

      const baseExpect: FixtureSnapshot["expect"] = {
        core: snapshotRowModel(table.getCoreRowModel()),
        filtered: snapshotRowModel(table.getFilteredRowModel()),
        sorted: snapshotRowModel(table.getSortedRowModel()),
        paginated: snapshotRowModel(table.getPaginationRowModel()),
        row_model: snapshotRowModel(table.getRowModel()),
      }

      const header_groups = snapshotHeaderGroups(table.getHeaderGroups())
      const left_header_groups = snapshotHeaderGroups(table.getLeftHeaderGroups())
      const center_header_groups = snapshotHeaderGroups(table.getCenterHeaderGroups())
      const right_header_groups = snapshotHeaderGroups(table.getRightHeaderGroups())
      const footer_groups = snapshotHeaderGroups(table.getFooterGroups())
      const left_footer_groups = snapshotHeaderGroups(table.getLeftFooterGroups())
      const center_footer_groups = snapshotHeaderGroups(table.getCenterFooterGroups())
      const right_footer_groups = snapshotHeaderGroups(table.getRightFooterGroups())
      const flat_headers = snapshotHeaders(table.getFlatHeaders())
      const left_flat_headers = snapshotHeaders(table.getLeftFlatHeaders())
      const center_flat_headers = snapshotHeaders(table.getCenterFlatHeaders())
      const right_flat_headers = snapshotHeaders(table.getRightFlatHeaders())
      const leaf_headers = snapshotHeaders(table.getLeafHeaders())
      const left_leaf_headers = snapshotHeaders(table.getLeftLeafHeaders())
      const center_leaf_headers = snapshotHeaders(table.getCenterLeafHeaders())
      const right_leaf_headers = snapshotHeaders(table.getRightLeafHeaders())
      const cells = snapshotCells(table)

      return {
        id,
        options,
        state,
        expect: {
          ...baseExpect,
          headers_cells: {
            header_groups,
            footer_groups,
            left_header_groups,
            left_footer_groups,
            center_header_groups,
            center_footer_groups,
            right_header_groups,
            right_footer_groups,
            flat_headers,
            left_flat_headers,
            center_flat_headers,
            right_flat_headers,
            leaf_headers,
            left_leaf_headers,
            center_leaf_headers,
            right_leaf_headers,
            cells,
          },
          core_model: {
            column_tree: snapshotColumnTree(table.getAllColumns()),
            leaf_columns: {
              all: (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id)),
              visible: (table.getVisibleLeafColumns?.() ?? []).map((c: any) => String(c.id)),
              left_visible: (table.getLeftVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
              center_visible: (table.getCenterVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
              right_visible: (table.getRightVisibleLeafColumns?.() ?? []).map((c: any) =>
                String(c.id),
              ),
            },
            header_groups,
            left_header_groups,
            center_header_groups,
            right_header_groups,
            rows: {
              core: snapshotRowModel(table.getCoreRowModel()),
              row_model: snapshotRowModel(table.getRowModel()),
            },
            cells,
          },
        },
      }
    }

    snapshots = [
      mk("headers_inventory_deep_baseline", base, {
        columnPinning: { left: ["name"], right: ["status"] },
      }),
      mk("headers_inventory_deep_pin_one_leaf_left", base, {
        columnPinning: { left: ["cpu"], right: [] },
      }),
      mk("headers_inventory_deep_hide_deep_leaf", base, {
        columnVisibility: { cpu2: false },
      }),
      mk("headers_inventory_deep_hide_whole_branch", base, {
        columnVisibility: { perf: false },
      }),
      mk("headers_inventory_deep_order_reorders_across_depths", base, {
        columnOrder: ["status", "mem_mb", "cpu", "cpu2", "name"],
      }),
    ]
  } else if (case_id === "visibility_ordering") {
    const coreModelForState = (options: TanStackOptions, state: TanStackState) => {
      const { table } = buildTable(options, state)
      const header_groups = snapshotHeaderGroups(table.getHeaderGroups())
      const left_header_groups = snapshotHeaderGroups(table.getLeftHeaderGroups())
      const center_header_groups = snapshotHeaderGroups(table.getCenterHeaderGroups())
      const right_header_groups = snapshotHeaderGroups(table.getRightHeaderGroups())
      const cells = snapshotCells(table)

      return {
        column_tree: snapshotColumnTree(table.getAllColumns()),
        flat_columns: {
          all: (table.getAllFlatColumns?.() ?? []).map((c: any) => String(c.id)),
          visible: (table.getVisibleFlatColumns?.() ?? []).map((c: any) => String(c.id)),
        },
        leaf_columns: {
          all: (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id)),
          visible: (table.getVisibleLeafColumns?.() ?? []).map((c: any) => String(c.id)),
          left_visible: (table.getLeftVisibleLeafColumns?.() ?? []).map((c: any) => String(c.id)),
          center_visible: (table.getCenterVisibleLeafColumns?.() ?? []).map((c: any) =>
            String(c.id),
          ),
          right_visible: (table.getRightVisibleLeafColumns?.() ?? []).map((c: any) =>
            String(c.id),
          ),
        },
        header_groups,
        left_header_groups,
        center_header_groups,
        right_header_groups,
        rows: {
          core: snapshotRowModel(table.getCoreRowModel()),
          row_model: snapshotRowModel(table.getRowModel()),
        },
        cells,
      }
    }

    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => ({
      id,
      options,
      state,
      expect: {
        ...snapshotForState(options, state),
        ...snapshotColumnSizing(buildTable(options, state).table),
        core_model: coreModelForState(options, state),
        is_all_columns_visible: Boolean(
          buildTable(options, state).table.getIsAllColumnsVisible?.(),
        ),
        is_some_columns_visible: Boolean(
          buildTable(options, state).table.getIsSomeColumnsVisible?.(),
        ),
      },
    })

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          core_model: coreModelForState(options, expect.next_state),
          is_all_columns_visible: Boolean(
            buildTable(options, expect.next_state).table.getIsAllColumnsVisible?.(),
          ),
          is_some_columns_visible: Boolean(
            buildTable(options, expect.next_state).table.getIsSomeColumnsVisible?.(),
          ),
        },
      }
    }

    const base: TanStackOptions = {
      enableHiding: true,
    }

    snapshots = [
      mk("visord_baseline", base, {}),
      mk("visord_pinning_left_a_right_c", base, {
        columnPinning: { left: ["a"], right: ["c"] },
      }),
      mk("visord_pinning_left_a_right_c_hide_left", base, {
        columnPinning: { left: ["a"], right: ["c"] },
        columnVisibility: { a: false },
      }),
      mk("visord_pinning_left_a_right_c_hide_center_and_resize_left", base, {
        columnPinning: { left: ["a"], right: ["c"] },
        columnVisibility: { b: false },
        columnSizing: { a: 120 },
      }),
      mk("visord_pinning_left_a_right_c_order_and_hide_right", base, {
        columnOrder: ["c", "a", "b"],
        columnPinning: { left: ["a"], right: ["c"] },
        columnVisibility: { c: false },
      }),
      mkActions("visord_toggle_column_a_off", base, {}, [
        { type: "toggleColumnVisibility", column_id: "a", value: false },
      ]),
      mkActions("visord_toggle_column_a_off_enable_hiding_false_noops", { enableHiding: false }, {}, [
        { type: "toggleColumnVisibility", column_id: "a", value: false },
      ]),
      mkActions(
        "visord_toggle_column_a_off_on_column_visibility_change_noop_ignores",
        { ...base, __onColumnVisibilityChange: "noop" },
        {},
        [{ type: "toggleColumnVisibility", column_id: "a", value: false }],
      ),
      mkActions("visord_toggle_all_off_keeps_non_hideable", base, {}, [
        { type: "toggleAllColumnsVisible", value: false },
      ]),
      mkActions(
        "visord_toggle_all_on_clears_state",
        base,
        { columnVisibility: { a: false, c: false } },
        [{ type: "toggleAllColumnsVisible", value: true }],
      ),
      mkActions("visord_set_column_order_reorders", base, {}, [
        { type: "setColumnOrder", order: ["c", "a", "b"] },
      ]),
      mkActions(
        "visord_set_column_order_on_column_order_change_noop_ignores",
        { ...base, __onColumnOrderChange: "noop" },
        {},
        [{ type: "setColumnOrder", order: ["c", "a", "b"] }],
      ),
      mkActions("visord_set_column_order_with_duplicates", base, {}, [
        { type: "setColumnOrder", order: ["c", "a", "c", "b"] },
      ]),
      mkActions("visord_set_order_then_hide", base, {}, [
        { type: "setColumnOrder", order: ["c", "a", "b"] },
        { type: "toggleColumnVisibility", column_id: "a", value: false },
      ]),
      mkActions(
        "visord_toggle_noop_when_enable_hiding_false",
        { enableHiding: false },
        {},
        [{ type: "toggleColumnVisibility", column_id: "a", value: false }],
      ),
    ]
  } else if (case_id === "grouping") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      const isGroupingApplied =
        !options.manualGrouping &&
        options.__getGroupedRowModel !== "pre_grouped" &&
        (state.grouping?.length ?? 0) > 0
      const sorted_grouped_row_model =
        isGroupingApplied ? snapshotSortedGroupedRowModel(table) : undefined
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          grouped_row_model: snapshotGroupedRowModel(table),
          grouped_aggregations_u64: snapshotGroupedAggregationsU64(table),
          sorted_grouped_row_model,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      const isGroupingApplied =
        !options.manualGrouping &&
        options.__getGroupedRowModel !== "pre_grouped" &&
        (expect.next_state.grouping?.length ?? 0) > 0
      const sorted_grouped_row_model =
        isGroupingApplied ? snapshotSortedGroupedRowModel(table) : undefined
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          grouped_row_model: snapshotGroupedRowModel(table),
          grouped_aggregations_u64: snapshotGroupedAggregationsU64(table),
          sorted_grouped_row_model,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const mkActionsAutoReset = async (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = await snapshotForGroupingActionsWithAutoResetFlush(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      const isGroupingApplied =
        !options.manualGrouping &&
        options.__getGroupedRowModel !== "pre_grouped" &&
        (expect.next_state.grouping?.length ?? 0) > 0
      const sorted_grouped_row_model =
        isGroupingApplied ? snapshotSortedGroupedRowModel(table) : undefined
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          grouped_row_model: snapshotGroupedRowModel(table),
          grouped_aggregations_u64: snapshotGroupedAggregationsU64(table),
          sorted_grouped_row_model,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    snapshots = [
      mk("grouping_baseline", {}, {}),
      mk("grouping_state_one_column", {}, { grouping: ["role"] }),
      mk("grouping_state_two_columns", {}, { grouping: ["role", "team"] }),
      mk(
        "grouping_state_one_column_row_pinning_keep_true_page_1",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          grouping: ["role"],
          pagination: { pageIndex: 1, pageSize: 1 },
          rowPinning: { top: ["4"], bottom: ["5"] },
        },
      ),
      mk(
        "grouping_state_one_column_row_pinning_keep_false_page_1",
        { enableRowPinning: true, keepPinnedRows: false },
        {
          grouping: ["role"],
          pagination: { pageIndex: 1, pageSize: 1 },
          rowPinning: { top: ["4"], bottom: ["5"] },
        },
      ),
      mk("grouping_manual_grouping_true_noops", { manualGrouping: true }, { grouping: ["role"] }),
      mk(
        "grouping_enable_grouping_false_state_noops",
        { enableGrouping: false },
        { grouping: ["role"] },
      ),
      mk(
        "grouping_override_get_grouped_row_model_pre_grouped",
        { __getGroupedRowModel: "pre_grouped" },
        { grouping: ["role"] },
      ),
      mkActions("grouping_action_toggle_role_on", {}, {}, [
        { type: "toggleGrouping", column_id: "role" },
      ]),
      mkActions("grouping_action_toggle_role_off", {}, { grouping: ["role"] }, [
        { type: "toggleGrouping", column_id: "role" },
      ]),
      await mkActionsAutoReset(
        "grouping_autoreset_expanded_default_resets",
        {},
        { expanded: { "1": true } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_expanded_manual_expanding_true_no_reset",
        { manualExpanding: true },
        { expanded: { "1": true } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_expanded_auto_reset_expanded_true_overrides_manual",
        { manualExpanding: true, autoResetExpanded: true },
        { expanded: { "1": true } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_page_index_default_resets",
        {},
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_page_index_manual_pagination_true_no_reset",
        { manualPagination: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_page_index_auto_reset_page_index_true_overrides_manual",
        { manualPagination: true, autoResetPageIndex: true },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      await mkActionsAutoReset(
        "grouping_autoreset_page_index_auto_reset_all_false_disables",
        { autoResetAll: false },
        { pagination: { pageIndex: 1, pageSize: 2 } },
        [
          { type: "toggleGrouping", column_id: "role" },
          { type: "toggleGrouping", column_id: "team" },
        ],
      ),
      mkActions(
        "grouping_action_toggle_noop_when_enable_grouping_false",
        { enableGrouping: false },
        {},
        [{ type: "toggleGroupingHandler", column_id: "role" }],
      ),
      mkActions(
        "grouping_action_toggle_ignores_enable_grouping_false",
        { enableGrouping: false },
        {},
        [{ type: "toggleGrouping", column_id: "role" }],
      ),
      mk("grouping_state_one_column_sort_role_desc", {}, {
        grouping: ["role"],
        sorting: [{ id: "role", desc: true }],
      }),
      mk("grouping_state_one_column_sort_score_desc", {}, {
        grouping: ["role"],
        sorting: [{ id: "score", desc: true }],
      }),
      mk("grouping_state_two_columns_sort_score_desc", {}, {
        grouping: ["role", "team"],
        sorting: [{ id: "score", desc: true }],
      }),
    ]
  } else if (case_id === "grouping_aggregation_fns") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          grouped_row_model: snapshotGroupedRowModel(table),
          grouped_aggregations_any: snapshotGroupedAggregationsAny(table),
        },
      }
    }

    snapshots = [
      mk("grouping_aggregation_fns_builtin_mix", {}, { grouping: ["role"] }),
      mk(
        "grouping_aggregation_fns_custom_registry",
        { aggregationFnsMode: "custom_plus_one" },
        { grouping: ["role"] },
      ),
    ]
  } else if (case_id === "render_fallback") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          render_fallback: snapshotRenderFallback(table),
        },
      }
    }

    snapshots = [mk("render_fallback_baseline", { renderFallbackValue: "N/A" }, {})]
  } else if (case_id === "pinning") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const baseState: TanStackState = {
      pagination: { pageIndex: 0, pageSize: 2 },
      rowPinning: { top: ["4"], bottom: ["5"] },
    }

    snapshots = [
      mk(
        "pinning_keep_true_page_0",
        { enableRowPinning: true, keepPinnedRows: true },
        baseState,
      ),
      mk(
        "pinning_keep_false_page_0",
        { enableRowPinning: true, keepPinnedRows: false },
        baseState,
      ),
      mk(
        "pinning_keep_true_multi_pinned_index_page_0",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          pagination: { pageIndex: 0, pageSize: 1 },
          rowPinning: { top: ["2", "3"], bottom: ["4", "5"] },
        },
      ),
      mk(
        "pinning_keep_false_multi_pinned_index_page_0",
        { enableRowPinning: true, keepPinnedRows: false },
        {
          pagination: { pageIndex: 0, pageSize: 1 },
          rowPinning: { top: ["2", "3"], bottom: ["4", "5"] },
        },
      ),
      mk(
        "pinning_keep_true_sorted_page_0",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          sorting: [{ id: "cpu", desc: true }],
          pagination: { pageIndex: 0, pageSize: 2 },
          rowPinning: { top: ["2"], bottom: [] },
        },
      ),
      mk(
        "pinning_keep_false_sorted_page_0",
        { enableRowPinning: true, keepPinnedRows: false },
        {
          sorting: [{ id: "cpu", desc: true }],
          pagination: { pageIndex: 0, pageSize: 2 },
          rowPinning: { top: ["2"], bottom: [] },
        },
      ),
      mk(
        "pinning_keep_true_filter_excludes_pinned",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          ...baseState,
          globalFilter: "Renderer",
        },
      ),
      mk(
        "pinning_keep_false_filter_excludes_pinned",
        { enableRowPinning: true, keepPinnedRows: false },
        {
          ...baseState,
          globalFilter: "Renderer",
        },
      ),
      mk(
        "pinning_enable_row_pinning_false_disables_can_pin",
        { enableRowPinning: false, keepPinnedRows: true },
        baseState,
      ),
      mk(
        "pinning_enable_pinning_false_disables_can_pin",
        { enablePinning: false, keepPinnedRows: true },
        baseState,
      ),
      mk(
        "pinning_enable_pinning_false_enable_row_pinning_true_overrides",
        { enablePinning: false, enableRowPinning: true, keepPinnedRows: true },
        baseState,
      ),
      mk(
        "pinning_enable_pinning_false_enable_row_pinning_fn_overrides",
        { enablePinning: false, __enableRowPinning: "odd_ids", keepPinnedRows: true },
        baseState,
      ),
      mkActions(
        "pinning_action_pin_top_bottom",
        { enableRowPinning: true, keepPinnedRows: true },
        { pagination: { pageIndex: 0, pageSize: 2 } },
        [
          {
            type: "pinRow",
            row_id: "4",
            position: "top",
          },
          {
            type: "pinRow",
            row_id: "5",
            position: "bottom",
          },
        ],
      ),
      mkActions(
        "pinning_action_unpin_top",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          pagination: { pageIndex: 0, pageSize: 2 },
          rowPinning: { top: ["4"], bottom: [] },
        },
        [
          {
            type: "pinRow",
            row_id: "4",
            position: null,
          },
        ],
      ),
      mkActions(
        "pinning_action_on_row_pinning_change_noop_ignores",
        { enableRowPinning: true, keepPinnedRows: true, __onRowPinningChange: "noop" },
        {},
        [
          {
            type: "pinRow",
            row_id: "4",
            position: "top",
          },
        ],
      ),
      mkActions(
        "pinning_action_reset_row_pinning_restores_initial",
        {
          enableRowPinning: true,
          keepPinnedRows: true,
          initialState: { rowPinning: { top: ["2"], bottom: [] } },
        },
        {
          rowPinning: { top: ["4"], bottom: ["5"] },
        },
        [
          {
            type: "resetRowPinning",
            default_state: false,
          },
        ],
      ),
      mkActions(
        "pinning_action_reset_row_pinning_default_true_clears",
        {
          enableRowPinning: true,
          keepPinnedRows: true,
          initialState: { rowPinning: { top: ["2"], bottom: [] } },
        },
        {
          rowPinning: { top: ["4"], bottom: ["5"] },
        },
        [
          {
            type: "resetRowPinning",
            default_state: true,
          },
        ],
      ),
    ]
  } else if (case_id === "pinning_tree") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    snapshots = [
      mk(
        "pinning_tree_keep_true_child_hidden_when_parent_collapsed",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          rowPinning: { top: ["11"], bottom: [] },
        },
      ),
      mk(
        "pinning_tree_keep_true_child_visible_when_parent_expanded",
        { enableRowPinning: true, keepPinnedRows: true },
        {
          expanded: { "1": true },
          rowPinning: { top: ["11"], bottom: [] },
        },
      ),
      mk(
        "pinning_tree_keep_false_never_surfaces_child_row",
        { enableRowPinning: true, keepPinnedRows: false },
        {
          expanded: { "1": true },
          rowPinning: { top: ["11"], bottom: [] },
        },
      ),
      mkActions(
        "pinning_tree_action_pin_root_includes_leaf_rows",
        { enableRowPinning: true, keepPinnedRows: true },
        {},
        [
          {
            type: "pinRow",
            row_id: "1",
            position: "top",
            include_leaf_rows: true,
          },
        ],
      ),
      mkActions(
        "pinning_tree_action_pin_grandchild_includes_parent_rows",
        { enableRowPinning: true, keepPinnedRows: true },
        {},
        [
          {
            type: "pinRow",
            row_id: "121",
            position: "bottom",
            include_parent_rows: true,
          },
        ],
      ),
    ]
  } else if (case_id === "pinning_grouped_rows") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          row_pinning: snapshotRowPinning(table),
        },
      }
    }

    const baseOptions: TanStackOptions = { enableRowPinning: true, keepPinnedRows: true }

    snapshots = [
      mk("pinning_grouped_rows_baseline_page_0", baseOptions, {
        grouping: ["role"],
        pagination: { pageIndex: 0, pageSize: 1 },
      }),
      mkActions(
        "pinning_grouped_rows_action_pin_group_role_1_top",
        baseOptions,
        {
          grouping: ["role"],
          pagination: { pageIndex: 0, pageSize: 1 },
        },
        [
          {
            type: "pinRow",
            row_id: "role:1",
            position: "top",
          },
        ],
      ),
      mkActions(
        "pinning_grouped_rows_action_pin_group_role_1_top_include_leaf_rows",
        baseOptions,
        {
          grouping: ["role"],
          pagination: { pageIndex: 0, pageSize: 1 },
        },
        [
          {
            type: "pinRow",
            row_id: "role:1",
            position: "top",
            include_leaf_rows: true,
          },
        ],
      ),
      mkActions(
        "pinning_grouped_rows_action_pin_leaf_1_top_include_parent_rows",
        baseOptions,
        {
          grouping: ["role"],
          pagination: { pageIndex: 0, pageSize: 1 },
        },
        [
          {
            type: "pinRow",
            row_id: "1",
            position: "top",
            include_parent_rows: true,
          },
        ],
      ),
      mk("pinning_grouped_rows_state_page_1_pinned_role_1", baseOptions, {
        grouping: ["role"],
        pagination: { pageIndex: 1, pageSize: 1 },
        rowPinning: { top: ["role:1"], bottom: [] },
      }),
    ]
  } else if (case_id === "column_pinning") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          column_pinning: snapshotColumnPinning(table),
          cells: snapshotCells(table),
        },
      }
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          column_pinning: snapshotColumnPinning(table),
          cells: snapshotCells(table),
        },
      }
    }

    snapshots = [
      mk("column_pinning_default_can_pin", {}, {}),
      mk(
        "column_pinning_enable_column_pinning_false_disables_can_pin",
        { enableColumnPinning: false },
        {},
      ),
      mk(
        "column_pinning_enable_pinning_false_disables_can_pin",
        { enablePinning: false },
        {},
      ),
      mkActions(
        "column_pinning_action_pin_left_right_unpin",
        {},
        {},
        [
          { type: "pinColumn", column_id: "a", position: "left" },
          { type: "pinColumn", column_id: "c", position: "right" },
          { type: "pinColumn", column_id: "a", position: null },
        ],
      ),
      mkActions(
        "column_pinning_action_pin_group_pins_leaf_columns",
        {},
        {},
        [{ type: "pinColumn", column_id: "ab", position: "left" }],
      ),
      mkActions(
        "column_pinning_action_pins_when_enable_column_pinning_false",
        { enableColumnPinning: false },
        {},
        [{ type: "pinColumn", column_id: "a", position: "left" }],
      ),
      mkActions(
        "column_pinning_action_pins_when_enable_pinning_false",
        { enablePinning: false },
        {},
        [{ type: "pinColumn", column_id: "a", position: "left" }],
      ),
      mkActions(
        "column_pinning_action_on_column_pinning_change_noop_ignores",
        { __onColumnPinningChange: "noop" },
        {},
        [{ type: "pinColumn", column_id: "a", position: "left" }],
      ),
      mkActions(
        "column_pinning_action_reset_column_pinning_restores_initial",
        { initialState: { columnPinning: { left: ["a"], right: [] } } },
        { columnPinning: { left: [], right: ["c"] } },
        [{ type: "resetColumnPinning", default_state: false }],
      ),
      mkActions(
        "column_pinning_action_reset_column_pinning_default_true_clears",
        { initialState: { columnPinning: { left: ["a"], right: [] } } },
        { columnPinning: { left: [], right: ["c"] } },
        [{ type: "resetColumnPinning", default_state: true }],
      ),
    ]
  } else if (case_id === "faceting") {
    const mk = (id: SnapshotId, options: TanStackOptions, state: TanStackState) => {
      const base = snapshotForState(options, state)
      const { table } = buildTable(options, state)
      return {
        id,
        options,
        state,
        expect: {
          ...base,
          faceting: {
            cpu: snapshotFaceting(table, "cpu"),
            global: snapshotGlobalFaceting(table),
          },
        },
      }
    }

    snapshots = [
      mk("faceting_baseline", {}, {}),
      mk(
        "faceting_cpu_own_filter_ignored",
        {},
        { columnFilters: [{ id: "cpu", value: [12, 12] }] },
      ),
      mk(
        "faceting_cpu_other_filter_applied",
        {},
        {
          columnFilters: [
            { id: "status", value: "Running" },
            { id: "cpu", value: [12, 12] },
          ],
        },
      ),
      mk(
        "faceting_manual_filtering_bypasses",
        { manualFiltering: true },
        { columnFilters: [{ id: "status", value: "Running" }] },
      ),
    ]
  } else if (case_id === "sort_undefined") {
    snapshots = [
      {
        id: "sort_undefined_first_asc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_first", desc: false }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_first", desc: false }],
        }),
      },
      {
        id: "sort_undefined_last_asc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_last", desc: false }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_last", desc: false }],
        }),
      },
      {
        id: "sort_undefined_1_asc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_1", desc: false }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_1", desc: false }],
        }),
      },
      {
        id: "sort_undefined_1_desc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_1", desc: true }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_1", desc: true }],
        }),
      },
      {
        id: "sort_undefined_neg1_asc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_neg1", desc: false }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_neg1", desc: false }],
        }),
      },
      {
        id: "sort_undefined_neg1_desc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_neg1", desc: true }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_neg1", desc: true }],
        }),
      },
      {
        id: "sort_undefined_false_text_asc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_false_text", desc: false }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_false_text", desc: false }],
        }),
      },
      {
        id: "sort_undefined_false_text_desc",
        options: sortAscFirst,
        state: { sorting: [{ id: "rank_false_text", desc: true }] },
        expect: snapshotForState(sortAscFirst, {
          sorting: [{ id: "rank_false_text", desc: true }],
        }),
      },
    ]
  } else if (case_id === "column_sizing") {
    const baseOptions: TanStackOptions = {
      enableColumnResizing: true,
    }
    const pinnedOrderedState: TanStackState = {
      columnOrder: ["b", "c", "a"],
      columnPinning: { left: ["b"], right: ["a"] },
    }
    const expectPinnedDefaults = (() => {
      const base = snapshotForState(baseOptions, pinnedOrderedState)
      const { table } = buildTable(baseOptions, pinnedOrderedState)
      const sizing = snapshotColumnSizing(table)
      return { ...base, ...sizing }
    })()

    const clampedState: TanStackState = {
      ...pinnedOrderedState,
      columnSizing: { a: 5000, b: 1, c: 75 },
    }
    const expectClamped = (() => {
      const base = snapshotForState(baseOptions, clampedState)
      const { table } = buildTable(baseOptions, clampedState)
      const sizing = snapshotColumnSizing(table)
      return { ...base, ...sizing }
    })()

    snapshots = [
      {
        id: "colsize_pinned_order_defaults",
        options: baseOptions,
        state: pinnedOrderedState,
        expect: expectPinnedDefaults,
      },
      {
        id: "colsize_override_and_clamp",
        options: baseOptions,
        state: clampedState,
        expect: expectClamped,
      },
      {
        id: "colsize_enable_column_resizing_false_noops",
        options: {
          ...baseOptions,
          enableColumnResizing: false,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            enableColumnResizing: false,
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
            { type: "columnResizeEnd", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_reset_column_size_removes_override",
        options: baseOptions,
        state: clampedState,
        actions: [{ type: "resetColumnSize", column_id: "a" }],
        expect: snapshotForActions(baseOptions, clampedState, [
          { type: "resetColumnSize", column_id: "a" },
        ]),
      },
      {
        id: "colsize_reset_column_sizing_default_true_clears",
        options: baseOptions,
        state: clampedState,
        actions: [{ type: "resetColumnSizing", default_state: true }],
        expect: snapshotForActions(baseOptions, clampedState, [
          { type: "resetColumnSizing", default_state: true },
        ]),
      },
      {
        id: "colsize_reset_column_sizing_restores_initial",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
          initialState: {
            columnSizing: { c: 70 },
          },
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
          { type: "resetColumnSizing" },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
            initialState: {
              columnSizing: { c: 70 },
            },
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
            { type: "columnResizeEnd", client_x: 35 },
            { type: "resetColumnSizing" },
          ],
        ),
      },
      {
        id: "colsize_reset_header_size_info_default_true_clears",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "resetHeaderSizeInfo", default_state: true },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
            { type: "resetHeaderSizeInfo", default_state: true },
          ],
        ),
      },
      {
        id: "colsize_hook_noop_sizing_move_keeps_sizing",
        options: {
          ...baseOptions,
          __onColumnSizingChange: "noop",
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            __onColumnSizingChange: "noop",
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_hook_noop_sizing_reset_column_sizing_keeps_state",
        options: {
          ...baseOptions,
          __onColumnSizingChange: "noop",
        },
        state: clampedState,
        actions: [{ type: "resetColumnSizing", default_state: true }],
        expect: snapshotForActions(
          {
            ...baseOptions,
            __onColumnSizingChange: "noop",
          },
          clampedState,
          [{ type: "resetColumnSizing", default_state: true }],
        ),
      },
      {
        id: "colsize_hook_noop_info_move_keeps_info_and_sizing",
        options: {
          ...baseOptions,
          __onColumnSizingInfoChange: "noop",
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            __onColumnSizingInfoChange: "noop",
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_hook_noop_info_reset_header_size_info_keeps_state",
        options: {
          ...baseOptions,
          __onColumnSizingInfoChange: "noop",
        },
        state: pinnedOrderedState,
        actions: [{ type: "resetHeaderSizeInfo", default_state: true }],
        expect: snapshotForActions(
          {
            ...baseOptions,
            __onColumnSizingInfoChange: "noop",
          },
          pinnedOrderedState,
          [{ type: "resetHeaderSizeInfo", default_state: true }],
        ),
      },
      {
        id: "colsize_resize_on_change_move_updates",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_resize_on_change_end_resets",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onChange",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
            { type: "columnResizeEnd", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_resize_on_end_move_no_sizing",
        options: {
          ...baseOptions,
          columnResizeMode: "onEnd",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onEnd",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_resize_on_end_end_writes",
        options: {
          ...baseOptions,
          columnResizeMode: "onEnd",
          columnResizeDirection: "ltr",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onEnd",
            columnResizeDirection: "ltr",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
            { type: "columnResizeEnd", client_x: 35 },
          ],
        ),
      },
      {
        id: "colsize_resize_rtl_move_flips",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "rtl",
        },
        state: pinnedOrderedState,
        actions: [
          { type: "columnResizeBegin", column_id: "c", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
          {
            ...baseOptions,
            columnResizeMode: "onChange",
            columnResizeDirection: "rtl",
          },
          pinnedOrderedState,
          [
            { type: "columnResizeBegin", column_id: "c", client_x: 10 },
            { type: "columnResizeMove", client_x: 35 },
          ],
        ),
      },
    ]
  } else if (case_id === "column_resizing_group_headers") {
    const baseOptions: TanStackOptions = {
      enableColumnResizing: true,
    }

    const pinnedState: TanStackState = {
      columnPinning: { left: ["a", "b"], right: ["c"] },
    }

    const mkActions = (
      id: SnapshotId,
      options: TanStackOptions,
      state: TanStackState,
      actions: FixtureAction[],
    ) => {
      const expect = snapshotForActions(options, state, actions)
      if (!expect.next_state) {
        throw new Error(`Missing next_state for snapshot ${id}`)
      }
      const { table } = buildTable(options, expect.next_state)
      return {
        id,
        options,
        state,
        actions,
        expect: {
          ...expect,
          header_sizing: snapshotHeaderSizing(table),
        },
      }
    }

    snapshots = [
      mkActions(
        "group_resize_on_change_move_updates",
        {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        {},
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
      ),
      mkActions(
        "group_resize_on_change_end_resets",
        {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        {},
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
      ),
      mkActions(
        "group_resize_on_end_end_writes",
        {
          ...baseOptions,
          columnResizeMode: "onEnd",
          columnResizeDirection: "ltr",
        },
        {},
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
      ),
      mkActions(
        "group_resize_pinned_on_change_move_updates",
        {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        pinnedState,
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
      ),
      mkActions(
        "group_resize_pinned_on_change_end_resets",
        {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        pinnedState,
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
      ),
      mkActions(
        "group_resize_pinned_on_end_end_writes",
        {
          ...baseOptions,
          columnResizeMode: "onEnd",
          columnResizeDirection: "ltr",
        },
        pinnedState,
        [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
      ),
    ]
  } else {
    throw new Error(`Unhandled fixture case_id: ${case_id}`)
  }

  const fixture: Fixture = {
    upstream: {
      package: tableCorePkgJson.name,
      version: tableCorePkgJson.version,
      ...tableRepoCommit,
      source: "repo-ref/table/packages/table-core",
    },
    case_id,
    data,
    columns_meta,
    snapshots,
  }

  fs.mkdirSync(path.dirname(out), { recursive: true })
  fs.writeFileSync(out, JSON.stringify(fixture, null, 2) + "\n", "utf8")
}

main().catch((err) => {
  console.error(err)
  process.exitCode = 1
})
