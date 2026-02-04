import fs from "fs"
import path from "path"
import { createRequire } from "module"
import { execSync } from "child_process"

type CaseId =
  | "demo_process"
  | "sort_undefined"
  | "sorting_fns"
  | "filtering_fns"
  | "headers_cells"
  | "visibility_ordering"
  | "pinning"
  | "pinning_tree"
  | "column_pinning"
  | "column_sizing"
  | "column_resizing_group_headers"
  | "state_shapes"
  | "selection"
  | "expanding"
  | "grouping"

type SnapshotId =
  | "baseline"
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
  | "group_resize_on_change_move_updates"
  | "group_resize_on_change_end_resets"
  | "group_resize_on_end_end_writes"
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
  | "expanding_baseline"
  | "expanding_state_row_1"
  | "expanding_state_all_true"
  | "expanding_paginate_expanded_rows_true_counts_children"
  | "expanding_paginate_expanded_rows_false_expands_within_page"
  | "expanding_action_toggle_row"
  | "expanding_action_toggle_all"
  | "pinning_keep_true_page_0"
  | "pinning_keep_false_page_0"
  | "pinning_keep_true_sorted_page_0"
  | "pinning_keep_false_sorted_page_0"
  | "pinning_keep_true_filter_excludes_pinned"
  | "pinning_keep_false_filter_excludes_pinned"
  | "pinning_enable_row_pinning_false_disables_can_pin"
  | "pinning_enable_pinning_false_disables_can_pin"
  | "pinning_enable_pinning_false_enable_row_pinning_true_overrides"
  | "pinning_action_pin_top_bottom"
  | "pinning_action_unpin_top"
  | "pinning_tree_keep_true_child_hidden_when_parent_collapsed"
  | "pinning_tree_keep_true_child_visible_when_parent_expanded"
  | "pinning_tree_keep_false_never_surfaces_child_row"
  | "pinning_tree_action_pin_root_includes_leaf_rows"
  | "pinning_tree_action_pin_grandchild_includes_parent_rows"
  | "column_pinning_default_can_pin"
  | "column_pinning_enable_column_pinning_false_disables_can_pin"
  | "column_pinning_enable_pinning_false_disables_can_pin"
  | "column_pinning_action_pin_left_right_unpin"
  | "column_pinning_action_pins_when_enable_column_pinning_false"
  | "column_pinning_action_pins_when_enable_pinning_false"
  | "grouping_baseline"
  | "grouping_state_one_column"
  | "grouping_state_two_columns"
  | "grouping_manual_grouping_true_noops"
  | "grouping_enable_grouping_false_state_noops"
  | "grouping_override_get_grouped_row_model_pre_grouped"
  | "grouping_action_toggle_role_on"
  | "grouping_action_toggle_role_off"
  | "grouping_action_toggle_noop_when_enable_grouping_false"
  | "grouping_action_toggle_ignores_enable_grouping_false"
  | "grouping_state_one_column_sort_role_desc"
  | "grouping_state_one_column_sort_score_desc"
  | "grouping_state_two_columns_sort_score_desc"

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
  manualFiltering?: boolean
  manualSorting?: boolean
  manualPagination?: boolean
  manualExpanding?: boolean
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
  columnResizeMode?: "onChange" | "onEnd"
  columnResizeDirection?: "ltr" | "rtl"
  // Fixture-only: override `getGroupedRowModel` with a deterministic implementation.
  __getGroupedRowModel?: "pre_grouped"
  // Fixture-only: when set, the generator injects a deterministic `options.sortingFns` map.
  sortingFnsMode?: "custom_text"
  // Fixture-only: when set, the generator injects a deterministic `options.filterFns` map.
  filterFnsMode?: "custom_text_case_sensitive"
  globalFilterFn?: "auto" | string
}

type RowModelSnapshot = { root: string[]; flat: string[] }

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
    selected?: RowModelSnapshot
    filtered_selected?: RowModelSnapshot
    grouped_selected?: RowModelSnapshot
    is_all_rows_selected?: boolean
    is_some_rows_selected?: boolean
    is_all_page_rows_selected?: boolean
    is_some_page_rows_selected?: boolean
    is_all_rows_expanded?: boolean
    is_some_rows_expanded?: boolean
    can_some_rows_expand?: boolean
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
    row_pinning?: {
      top: string[]
      center: string[]
      bottom: string[]
      can_pin: Record<string, boolean>
      pin_position: Record<string, "top" | "bottom" | null>
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
    column_pinning?: {
      left: string[]
      center: string[]
      right: string[]
      can_pin: Record<string, boolean>
      pin_position: Record<string, "left" | "right" | null>
      is_some_columns_pinned: boolean
      is_some_left_columns_pinned: boolean
      is_some_right_columns_pinned: boolean
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
      type: "toggleRowExpanded"
      row_id: string
      value?: boolean
    }
  | {
      type: "toggleAllRowsExpanded"
      value?: boolean
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
        v !== "sort_undefined" &&
        v !== "sorting_fns" &&
        v !== "filtering_fns" &&
        v !== "headers_cells" &&
        v !== "visibility_ordering" &&
        v !== "pinning" &&
        v !== "pinning_tree" &&
        v !== "column_pinning" &&
        v !== "column_sizing" &&
        v !== "column_resizing_group_headers" &&
        v !== "state_shapes" &&
        v !== "selection" &&
        v !== "expanding" &&
        v !== "grouping"
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
      "usage: node extract-fixtures.mts --out <path> [--case demo_process|sort_undefined|sorting_fns|filtering_fns|headers_cells|visibility_ordering|pinning|pinning_tree|column_pinning|column_sizing|column_resizing_group_headers|state_shapes|selection|expanding|grouping]",
    )
  }
  return { out, case_id }
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
    case_id === "state_shapes" ||
    case_id === "selection" ||
    case_id === "pinning"
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
  } else if (case_id === "expanding" || case_id === "pinning_tree") {
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

    const table = tableCore.createTable<DemoProcessRow>({
      data,
      columns,
      getRowId: (row: DemoProcessRow) => String(row.id),
      getSubRows: (row: DemoProcessRow) => (row as any).subRows,
      manualFiltering: options.manualFiltering ?? false,
      manualSorting: options.manualSorting ?? false,
      manualPagination: options.manualPagination ?? false,
      manualExpanding: options.manualExpanding ?? false,
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
      enableRowSelection: options.enableRowSelection ?? true,
      enableMultiRowSelection: options.enableMultiRowSelection ?? true,
      enableSubRowSelection: options.enableSubRowSelection ?? true,
      enableColumnResizing: options.enableColumnResizing ?? true,
      enableHiding: options.enableHiding ?? true,
      enableRowPinning: options.enableRowPinning,
      enableGrouping: options.enableGrouping,
      enableColumnPinning: options.enableColumnPinning,
      enablePinning: options.enablePinning,
      columnResizeMode: options.columnResizeMode,
      columnResizeDirection: options.columnResizeDirection,
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
      isMultiSortEvent: (e: unknown) => {
        return !!(e as any)?.multi
      },
      state: currentState,
      getCoreRowModel: tableCore.getCoreRowModel(),
      getFilteredRowModel: tableCore.getFilteredRowModel(),
      ...(case_id === "grouping"
        ? {
            getGroupedRowModel:
              options.__getGroupedRowModel === "pre_grouped"
                ? (t: any) => () => t.getPreGroupedRowModel?.()
                : tableCore.getGroupedRowModel(),
          }
        : {}),
      getSortedRowModel: tableCore.getSortedRowModel(),
      getPaginationRowModel: tableCore.getPaginationRowModel(),
      getExpandedRowModel: tableCore.getExpandedRowModel(),
      onSortingChange: (updater: any) => {
        const next = typeof updater === "function" ? updater(currentState.sorting) : updater
        currentState.sorting = next ?? []
      },
      onColumnFiltersChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.columnFilters) : updater
        currentState.columnFilters = next ?? []
      },
      onGlobalFilterChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.globalFilter) : updater
        currentState.globalFilter = next
      },
      onColumnVisibilityChange: (updater: any) => {
        const next =
          typeof updater === "function"
            ? updater(currentState.columnVisibility)
            : updater
        currentState.columnVisibility = next ?? {}
      },
      onColumnSizingChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.columnSizing) : updater
        currentState.columnSizing = next ?? {}
      },
      onColumnSizingInfoChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.columnSizingInfo) : updater
        currentState.columnSizingInfo = next ?? currentState.columnSizingInfo
      },
      onColumnPinningChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.columnPinning) : updater
        currentState.columnPinning = next ?? { left: [], right: [] }
      },
      onColumnOrderChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.columnOrder) : updater
        currentState.columnOrder = next ?? []
      },
      onRowPinningChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.rowPinning) : updater
        currentState.rowPinning = next ?? { top: [], bottom: [] }
      },
      onRowSelectionChange: (updater: any) => {
        const next =
          typeof updater === "function" ? updater(currentState.rowSelection) : updater
        currentState.rowSelection = next ?? {}
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
): FixtureSnapshot["expect"] {
  const { table } = buildTable(options, state)
  return {
    core: snapshotRowModel(table.getCoreRowModel()),
    filtered: snapshotRowModel(table.getFilteredRowModel()),
    sorted: snapshotRowModel(table.getSortedRowModel()),
    expanded: snapshotRowModel(table.getExpandedRowModel()),
    paginated: snapshotRowModel(table.getPaginationRowModel()),
    row_model: snapshotRowModel(table.getRowModel()),
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
}

function emptyRowModelSnapshot(): any {
  return { rows: [], flatRows: [] }
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
  for (const row of coreRows) {
    const id = String(row.id)
    const r = table.getRow?.(id, true)
    if (!r) {
      continue
    }
    can_pin[id] = Boolean(r.getCanPin?.())
    const pos = r.getIsPinned?.()
    pin_position[id] = pos === "top" ? "top" : pos === "bottom" ? "bottom" : null
  }

  return {
    top,
    center,
    bottom,
    can_pin,
    pin_position,
    is_some_rows_pinned: Boolean(table.getIsSomeRowsPinned?.()),
    is_some_top_rows_pinned: Boolean(table.getIsSomeRowsPinned?.("top")),
    is_some_bottom_rows_pinned: Boolean(table.getIsSomeRowsPinned?.("bottom")),
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

  const leaf = (table.getAllLeafColumns?.() ?? []).map((c: any) => String(c.id))
  const can_pin: Record<string, boolean> = {}
  const pin_position: Record<string, "left" | "right" | null> = {}
  for (const id of leaf) {
    const col = table.getColumn?.(id)
    if (!col) {
      continue
    }
    can_pin[id] = Boolean(col.getCanPin?.())
    const pos = col.getIsPinned?.()
    pin_position[id] = pos === "left" ? "left" : pos === "right" ? "right" : null
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
    is_some_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.()),
    is_some_left_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.("left")),
    is_some_right_columns_pinned: Boolean(table.getIsSomeColumnsPinned?.("right")),
  }
}

  function snapshotForActions(
    options: TanStackOptions,
    state: TanStackState,
    actions: FixtureAction[],
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
    snapshots = [
      {
        id: "baseline",
        options: defaultOptions,
        state: {},
        expect: snapshotForState(defaultOptions, {}),
      },
      {
        id: "sorted_cpu_desc",
        options: defaultOptions,
        state: { sorting: [{ id: "cpu", desc: true }] },
        expect: snapshotForState(defaultOptions, { sorting: [{ id: "cpu", desc: true }] }),
      },
      {
        id: "sorted_cpu_invert_asc",
        options: defaultOptions,
        state: { sorting: [{ id: "cpu_invert", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "cpu_invert", desc: false }],
        }),
      },
      {
        id: "sorted_cpu_toggle_desc_first",
        options: defaultOptions,
        state: {},
        actions: [{ type: "toggleSorting", column_id: "cpu_desc_first" }],
        expect: snapshotForActions(defaultOptions, {}, [
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
        expect: snapshotForActions(
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
        expect: snapshotForActions(
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
        expect: snapshotForActions(
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
        expect: snapshotForActions(
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
        expect: snapshotForActions(sortAscFirst, {}, [
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
        expect: snapshotForActions(
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
        expect: snapshotForActions(sortAscFirst, {}, [
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
        expect: snapshotForActions(sortAscFirst, { sorting: [{ id: "cpu", desc: false }] }, [
          { type: "toggleSortingHandler", column_id: "mem_mb", event_multi: true },
        ]),
      },
      {
        id: "filter_status_run",
        options: defaultOptions,
        state: { columnFilters: [{ id: "status", value: "run" }] },
        expect: snapshotForState(defaultOptions, {
          columnFilters: [{ id: "status", value: "run" }],
        }),
      },
      {
        id: "page_0_size_2",
        options: defaultOptions,
        state: { pagination: { pageIndex: 0, pageSize: 2 } },
        expect: snapshotForState(defaultOptions, {
          pagination: { pageIndex: 0, pageSize: 2 },
        }),
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
        id: "expanding_state_row_1",
        options: base,
        state: { expanded: { "1": true } },
        expect: snapshotForState(base, { expanded: { "1": true } }),
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
        id: "sorting_fns_builtin_datetime",
        options: defaultOptions,
        state: { sorting: [{ id: "dt_datetime", desc: false }] },
        expect: snapshotForState(defaultOptions, {
          sorting: [{ id: "dt_datetime", desc: false }],
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
    ]
  } else if (case_id === "filtering_fns") {
    const base = defaultOptions

    snapshots = [
      {
        id: "filtering_fns_text_auto_includes",
        options: base,
        state: { columnFilters: [{ id: "text_auto", value: "ap" }] },
        expect: snapshotForState(base, { columnFilters: [{ id: "text_auto", value: "ap" }] }),
      },
      {
        id: "filtering_fns_text_equals_string",
        options: base,
        state: { columnFilters: [{ id: "text_equals_string", value: "banana" }] },
        expect: snapshotForState(base, {
          columnFilters: [{ id: "text_equals_string", value: "banana" }],
        }),
      },
      {
        id: "filtering_fns_num_in_number_range",
        options: base,
        state: { columnFilters: [{ id: "num_range", value: [4, 8] }] },
        expect: snapshotForState(base, {
          columnFilters: [{ id: "num_range", value: [4, 8] }],
        }),
      },
      {
        id: "filtering_fns_tags_arr_includes_all",
        options: base,
        state: { columnFilters: [{ id: "tags_all", value: ["a", "b"] }] },
        expect: snapshotForState(base, {
          columnFilters: [{ id: "tags_all", value: ["a", "b"] }],
        }),
      },
      {
        id: "filtering_fns_bool_equals",
        options: base,
        state: { columnFilters: [{ id: "flag_equals", value: true }] },
        expect: snapshotForState(base, {
          columnFilters: [{ id: "flag_equals", value: true }],
        }),
      },
      {
        id: "filtering_fns_weak_equals_string_number",
        options: base,
        state: { columnFilters: [{ id: "num_weak", value: "5" }] },
        expect: snapshotForState(base, {
          columnFilters: [{ id: "num_weak", value: "5" }],
        }),
      },
      {
        id: "filtering_fns_global_filter_includes",
        options: base,
        state: { globalFilter: "ap" },
        expect: snapshotForState(base, { globalFilter: "ap" }),
      },
      {
        id: "filtering_fns_global_filter_default_excludes_bool",
        options: base,
        state: { globalFilter: "true" },
        expect: snapshotForState(base, { globalFilter: "true" }),
      },
      {
        id: "filtering_fns_global_filter_disabled_when_enable_filters_false",
        options: { ...base, enableFilters: false },
        state: { globalFilter: "ap" },
        expect: snapshotForState({ ...base, enableFilters: false }, { globalFilter: "ap" }),
      },
      {
        id: "filtering_fns_registry_custom_text_case_sensitive",
        options: { ...base, filterFnsMode: "custom_text_case_sensitive" },
        state: { columnFilters: [{ id: "text_custom", value: "A" }] },
        expect: snapshotForState(
          { ...base, filterFnsMode: "custom_text_case_sensitive" },
          { columnFilters: [{ id: "text_custom", value: "A" }] },
        ),
      },
      {
        id: "filtering_fns_action_set_empty_removes",
        options: base,
        state: { columnFilters: [{ id: "text_auto", value: "ap" }] },
        actions: [{ type: "setColumnFilterValue", column_id: "text_auto", value: "" }],
        expect: snapshotForActions(base, { columnFilters: [{ id: "text_auto", value: "ap" }] }, [
          { type: "setColumnFilterValue", column_id: "text_auto", value: "" },
        ]),
      },
    ]
  } else if (case_id === "headers_cells") {
    const base = defaultOptions
    const mk = (id: SnapshotId, state: TanStackState) => {
      const { table } = buildTable(base, state)

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
      const cells = snapshotCells(table)

      return {
        id,
        options: base,
        state,
        expect: {
          ...baseExpect,
          headers_cells: {
            header_groups,
            left_header_groups,
            center_header_groups,
            right_header_groups,
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
      mk("baseline", { columnPinning: { left: ["name"], right: ["mem_mb"] } }),
      mk("headers_cells_order_and_pinning", {
        columnOrder: ["mem_mb", "cpu", "name"],
        columnPinning: { left: ["cpu"], right: [] },
      }),
      mk("headers_cells_hide_right_leaf", {
        columnPinning: { left: ["name"], right: ["mem_mb"] },
        columnVisibility: { mem_mb: false },
      }),
      mk("headers_cells_hide_left_leaf", {
        columnPinning: { left: ["cpu"], right: ["mem_mb"] },
        columnVisibility: { cpu: false },
      }),
      mk("headers_cells_column_order_reorders", {
        columnOrder: ["cpu", "name", "mem_mb"],
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
        core_model: coreModelForState(options, state),
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
        },
      }
    }

    const base: TanStackOptions = {
      enableHiding: true,
    }

    snapshots = [
      mk("visord_baseline", base, {}),
      mkActions("visord_toggle_column_a_off", base, {}, [
        { type: "toggleColumnVisibility", column_id: "a", value: false },
      ]),
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
        },
      }
    }

    snapshots = [
      mk("grouping_baseline", {}, {}),
      mk("grouping_state_one_column", {}, { grouping: ["role"] }),
      mk("grouping_state_two_columns", {}, { grouping: ["role", "team"] }),
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

    snapshots = [
      {
        id: "group_resize_on_change_move_updates",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: {},
        actions: [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
        ],
        expect: snapshotForActions(
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
      },
      {
        id: "group_resize_on_change_end_resets",
        options: {
          ...baseOptions,
          columnResizeMode: "onChange",
          columnResizeDirection: "ltr",
        },
        state: {},
        actions: [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
        expect: snapshotForActions(
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
      },
      {
        id: "group_resize_on_end_end_writes",
        options: {
          ...baseOptions,
          columnResizeMode: "onEnd",
          columnResizeDirection: "ltr",
        },
        state: {},
        actions: [
          { type: "columnResizeBegin", column_id: "ab", client_x: 10 },
          { type: "columnResizeMove", client_x: 35 },
          { type: "columnResizeEnd", client_x: 35 },
        ],
        expect: snapshotForActions(
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
      },
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
