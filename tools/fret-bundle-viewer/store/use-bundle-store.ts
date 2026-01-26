import { create } from 'zustand'
import type { BundleModel, SnapshotModel, SemanticsNodeModel, UiMessage, ZipImportMeta } from '@/lib/types'
import { parseBundle } from '@/lib/parser'
import { simpleSampleBundle, multiWindowSampleBundle } from '@/lib/sample-bundles'
import { isLocalizedErrorLike } from '@/lib/localized-error'

// Callback for when a file is loaded (to track recent files)
type OnFileLoadedCallback = (fileName: string, fileSize: number, content: string) => void
let onFileLoadedCallback: OnFileLoadedCallback | null = null

export function setOnFileLoadedCallback(callback: OnFileLoadedCallback | null) {
  onFileLoadedCallback = callback
}

function revokeZipObjectUrls(zip?: ZipImportMeta) {
  if (!zip?.screenshots?.length) return
  if (typeof URL === 'undefined' || typeof URL.revokeObjectURL !== 'function') return
  for (const s of zip.screenshots) {
    if (!s?.objectUrl) continue
    try {
      URL.revokeObjectURL(s.objectUrl)
    } catch {
      // ignore
    }
  }
}

interface BundleState {
  // Bundle data
  bundle: BundleModel | null
  rawText: string | null
  parseError: UiMessage | null

  // Selection state
  selectedWindowIndex: number
  selectedSnapshotAIndex: number
  selectedSnapshotBIndex: number | null
  selectedNodeId: string | null

  // UI state
  compareMode: boolean
  redactText: boolean
  searchQuery: string
  eventsSearchQuery: string
  expandedNodes: Set<string>

  // Computed getters
  getSelectedWindow: () => BundleState['bundle'] extends null ? null : NonNullable<BundleState['bundle']>['windows'][number] | null
  getSelectedSnapshotA: () => SnapshotModel | null
  getSelectedSnapshotB: () => SnapshotModel | null
  getSelectedNode: () => SemanticsNodeModel | null

  // Actions
  loadBundle: (
    text: string,
    fileNameOrOptions?: string | { fileName?: string; fileSize?: number; recordRecent?: boolean; zip?: ZipImportMeta }
  ) => void
  setParseError: (error: UiMessage | null, rawText?: string | null) => void
  loadSampleBundle: (type: 'simple' | 'multi-window') => void
  clearBundle: () => void

  setSelectedWindowIndex: (index: number) => void
  setSelectedSnapshotAIndex: (index: number) => void
  setSelectedSnapshotBIndex: (index: number | null) => void
  setSelectedNodeId: (id: string | null) => void

  setCompareMode: (enabled: boolean) => void
  setRedactText: (enabled: boolean) => void
  setSearchQuery: (query: string) => void
  setEventsSearchQuery: (query: string) => void

  toggleNodeExpanded: (nodeId: string) => void
  expandNode: (nodeId: string) => void
  collapseNode: (nodeId: string) => void
  expandAllNodes: () => void
  collapseAllNodes: () => void
}

export const useBundleStore = create<BundleState>((set, get) => ({
  // Initial state
  bundle: null,
  rawText: null,
  parseError: null,

  selectedWindowIndex: 0,
  selectedSnapshotAIndex: 0,
  selectedSnapshotBIndex: null,
  selectedNodeId: null,

  compareMode: false,
  redactText: false,
  searchQuery: '',
  eventsSearchQuery: '',
  expandedNodes: new Set<string>(),

  // Computed getters
  getSelectedWindow: () => {
    const { bundle, selectedWindowIndex } = get()
    if (!bundle || selectedWindowIndex >= bundle.windows.length) return null
    return bundle.windows[selectedWindowIndex]
  },

  getSelectedSnapshotA: () => {
    const window = get().getSelectedWindow()
    const { selectedSnapshotAIndex } = get()
    if (!window || selectedSnapshotAIndex >= window.snapshots.length) return null
    return window.snapshots[selectedSnapshotAIndex]
  },

  getSelectedSnapshotB: () => {
    const { compareMode, selectedSnapshotBIndex } = get()
    if (!compareMode || selectedSnapshotBIndex === null) return null
    const window = get().getSelectedWindow()
    if (!window || selectedSnapshotBIndex >= window.snapshots.length) return null
    return window.snapshots[selectedSnapshotBIndex]
  },

  getSelectedNode: () => {
    const snapshot = get().getSelectedSnapshotA()
    const { selectedNodeId } = get()
    if (!snapshot?.semantics || !selectedNodeId) return null
    return snapshot.semantics.nodesById[selectedNodeId] ?? null
  },

  // Actions
  loadBundle: (text, fileNameOrOptions) => {
    revokeZipObjectUrls(get().bundle?.meta.zip)

    const options =
      typeof fileNameOrOptions === 'string'
        ? { fileName: fileNameOrOptions }
        : fileNameOrOptions ?? {}
    const fileName = options.fileName
    const recordRecent = options.recordRecent ?? true
    try {
      const bundle = parseBundle(text, fileName)

      // Treat "can't parse JSON" as a fatal error (show the error UI), but keep other warnings non-fatal.
      const fatalWarning =
        bundle.windows.length === 0
          ? bundle.warnings.find(
              (w) => w.key === 'error.jsonParse' || w.key === 'error.bundleRootNotObject'
            )
          : undefined
      if (fatalWarning) {
        set({
          bundle: null,
          rawText: text,
          parseError: fatalWarning,
        })
        return
      }

      if (typeof options.fileSize === 'number') {
        bundle.meta.fileSize = options.fileSize
      }
      if (options.zip) {
        bundle.meta.zip = options.zip
      }
      
      // Auto-expand root nodes
      const expandedNodes = new Set<string>()
      for (const window of bundle.windows) {
        for (const snapshot of window.snapshots) {
          if (snapshot.semantics) {
            for (const rootId of snapshot.semantics.roots) {
              expandedNodes.add(rootId)
            }
          }
        }
      }

      set({
        bundle,
        rawText: text,
        parseError: null,
        selectedWindowIndex: 0,
        selectedSnapshotAIndex: 0,
        selectedSnapshotBIndex: null,
        selectedNodeId: null,
        expandedNodes,
      })

      // Notify callback for recent files tracking (only for user-loaded files, not samples)
      if (onFileLoadedCallback && recordRecent && fileName && !fileName.startsWith('sample-')) {
        onFileLoadedCallback(fileName, typeof options.fileSize === 'number' ? options.fileSize : text.length, text)
      }
    } catch (error) {
      if (isLocalizedErrorLike(error)) {
        set({
          bundle: null,
          rawText: text,
          parseError: {
            key: error.key,
            params: error.params,
            detail: error.detail,
          },
        })
        return
      }
      set({
        bundle: null,
        rawText: text,
        parseError: {
          key: 'error.unknownParse',
          detail: error instanceof Error ? error.message : String(error),
        },
      })
    }
  },

  setParseError: (error, rawText = null) => {
    revokeZipObjectUrls(get().bundle?.meta.zip)
    set({
      bundle: null,
      rawText,
      parseError: error,
      selectedWindowIndex: 0,
      selectedSnapshotAIndex: 0,
      selectedSnapshotBIndex: null,
      selectedNodeId: null,
      expandedNodes: new Set<string>(),
    })
  },

  loadSampleBundle: (type) => {
    const sampleData = type === 'simple' ? simpleSampleBundle : multiWindowSampleBundle
    const text = JSON.stringify(sampleData, null, 2)
    get().loadBundle(text, `sample-${type}.json`)
  },

  clearBundle: () => {
    revokeZipObjectUrls(get().bundle?.meta.zip)
    set({
      bundle: null,
      rawText: null,
      parseError: null,
      selectedWindowIndex: 0,
      selectedSnapshotAIndex: 0,
      selectedSnapshotBIndex: null,
      selectedNodeId: null,
      expandedNodes: new Set<string>(),
    })
  },

  setSelectedWindowIndex: (index) => {
    set({
      selectedWindowIndex: index,
      selectedSnapshotAIndex: 0,
      selectedSnapshotBIndex: null,
      selectedNodeId: null,
    })
  },

  setSelectedSnapshotAIndex: (index) => {
    set({ selectedSnapshotAIndex: index, selectedNodeId: null })
  },

  setSelectedSnapshotBIndex: (index) => {
    set({ selectedSnapshotBIndex: index })
  },

  setSelectedNodeId: (id) => {
    set({ selectedNodeId: id })
  },

  setCompareMode: (enabled) => {
    set({ 
      compareMode: enabled,
      selectedSnapshotBIndex: enabled ? null : null 
    })
  },

  setRedactText: (enabled) => {
    set({ redactText: enabled })
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query })
  },

  setEventsSearchQuery: (query) => {
    set({ eventsSearchQuery: query })
  },

  toggleNodeExpanded: (nodeId) => {
    const { expandedNodes } = get()
    const newSet = new Set(expandedNodes)
    if (newSet.has(nodeId)) {
      newSet.delete(nodeId)
    } else {
      newSet.add(nodeId)
    }
    set({ expandedNodes: newSet })
  },

  expandNode: (nodeId) => {
    const { expandedNodes } = get()
    const newSet = new Set(expandedNodes)
    newSet.add(nodeId)
    set({ expandedNodes: newSet })
  },

  collapseNode: (nodeId) => {
    const { expandedNodes } = get()
    const newSet = new Set(expandedNodes)
    newSet.delete(nodeId)
    set({ expandedNodes: newSet })
  },

  expandAllNodes: () => {
    const snapshot = get().getSelectedSnapshotA()
    if (!snapshot?.semantics) return
    const allIds = Object.keys(snapshot.semantics.nodesById)
    set({ expandedNodes: new Set(allIds) })
  },

  collapseAllNodes: () => {
    set({ expandedNodes: new Set<string>() })
  },
}))
