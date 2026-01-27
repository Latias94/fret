'use client'

import { useCallback, useMemo, useEffect, useState, useRef } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import type { SemanticsNodeModel, SemanticsModel } from '@/lib/types'
import { bestSelector, nodePath, selectorToJson, generateScriptStep, scriptStepToJson, copyToClipboard } from '@/lib/selector'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { cn } from '@/lib/utils'
import { useTranslation } from '@/hooks/use-i18n'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { ChevronRight, ChevronDown, ChevronsUpDown, ChevronsDownUp, Search, ChevronUp, X } from 'lucide-react'

const MAX_INDENT_DEPTH = 8 // Cap indentation at depth 8 to preserve horizontal space

const ROLE_COLORS: Record<string, string> = {
  window: 'bg-blue-100 text-blue-800 border-blue-200',
  button: 'bg-green-100 text-green-800 border-green-200',
  dialog: 'bg-purple-100 text-purple-800 border-purple-200',
  textfield: 'bg-amber-100 text-amber-800 border-amber-200',
  viewport: 'bg-cyan-100 text-cyan-800 border-cyan-200',
  panel: 'bg-slate-100 text-slate-800 border-slate-200',
  toolbar: 'bg-orange-100 text-orange-800 border-orange-200',
  canvas: 'bg-pink-100 text-pink-800 border-pink-200',
  list: 'bg-indigo-100 text-indigo-800 border-indigo-200',
  listitem: 'bg-violet-100 text-violet-800 border-violet-200',
  navigation: 'bg-teal-100 text-teal-800 border-teal-200',
  link: 'bg-sky-100 text-sky-800 border-sky-200',
  header: 'bg-rose-100 text-rose-800 border-rose-200',
  menubar: 'bg-lime-100 text-lime-800 border-lime-200',
  menuitem: 'bg-emerald-100 text-emerald-800 border-emerald-200',
  group: 'bg-fuchsia-100 text-fuchsia-800 border-fuchsia-200',
  colorpicker: 'bg-red-100 text-red-800 border-red-200',
}

function getRoleColor(role?: string): string {
  if (!role) return 'bg-muted text-muted-foreground border-border'
  return ROLE_COLORS[role.toLowerCase()] ?? 'bg-muted text-muted-foreground border-border'
}

interface TreeNodeProps {
  node: SemanticsNodeModel
  semantics: SemanticsModel
  depth: number
  isExpanded: boolean
  isSelected: boolean
  redactText: boolean
  searchQuery: string
  onToggle: (nodeId: string) => void
  onSelect: (nodeId: string) => void
}

function TreeNode({
  node,
  semantics,
  depth,
  isExpanded,
  isSelected,
  redactText,
  searchQuery,
  onToggle,
  onSelect,
}: TreeNodeProps) {
  const { t } = useTranslation()
  const hasChildren = node.children.length > 0
  const displayLabel = redactText ? (node.label || node.name ? '•••' : undefined) : (node.label ?? node.name)

  const isMatch = useMemo(() => {
    if (!searchQuery) return false
    const query = searchQuery.toLowerCase()
    return (
      node.id.toLowerCase().includes(query) ||
      node.testId?.toLowerCase().includes(query) ||
      node.role?.toLowerCase().includes(query) ||
      node.label?.toLowerCase().includes(query) ||
      node.name?.toLowerCase().includes(query)
    )
  }, [node, searchQuery])

  const handleCopySelector = useCallback(() => {
    const selector = bestSelector(node)
    copyToClipboard(selectorToJson(selector))
  }, [node])

  const handleCopyNodePath = useCallback(() => {
    copyToClipboard(nodePath(node, semantics))
  }, [node, semantics])

  const handleCopyNodeId = useCallback(() => {
    copyToClipboard(node.id)
  }, [node.id])

  const handleCopyScriptStep = useCallback(() => {
    const step = generateScriptStep(node)
    copyToClipboard(scriptStepToJson(step))
  }, [node])

  // Cap indentation to preserve horizontal space for deep trees
  const effectiveDepth = Math.min(depth, MAX_INDENT_DEPTH)
  const indentPx = effectiveDepth * 14 + 8

  return (
    <div>
      <ContextMenu>
        <ContextMenuTrigger>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  data-node-id={node.id}
                  className={cn(
                    'group flex min-h-7 cursor-pointer items-start gap-1 rounded-sm px-2 py-1 transition-colors hover:bg-accent',
                    isSelected && 'bg-accent',
                    isMatch && 'ring-2 ring-chart-4 ring-inset'
                  )}
                  style={{ paddingLeft: `${indentPx}px` }}
                  onClick={() => onSelect(node.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') onSelect(node.id)
                    if (e.key === 'ArrowRight' && hasChildren && !isExpanded) onToggle(node.id)
                    if (e.key === 'ArrowLeft' && hasChildren && isExpanded) onToggle(node.id)
                  }}
                  tabIndex={0}
                  role="treeitem"
                  aria-expanded={hasChildren ? isExpanded : undefined}
                  aria-selected={isSelected}
                >
                  {/* Expand/Collapse button - aligned to top */}
                  <button
                    type="button"
                    className={cn(
                      'mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center',
                      hasChildren ? 'cursor-pointer text-muted-foreground hover:text-foreground' : 'invisible'
                    )}
                    onClick={(e) => {
                      e.stopPropagation()
                      if (hasChildren) onToggle(node.id)
                    }}
                    tabIndex={-1}
                  >
                    {hasChildren && (isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />)}
                  </button>

                  {/* Content area - wraps when needed */}
                  <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1">
                    {/* Role badge */}
                    <Badge variant="outline" className={cn('h-4 shrink-0 px-1.5 py-0 text-[10px]', getRoleColor(node.role))}>
                      {node.role ?? 'Generic'}
                    </Badge>

                    {/* Test ID chip */}
                    {node.testId && (
                      <Badge variant="secondary" className="h-4 shrink-0 border-chart-1/20 bg-chart-1/10 px-1.5 py-0 text-[10px] text-chart-1">
                        {node.testId}
                      </Badge>
                    )}

                    {/* Label/Name - can wrap to new line */}
                    {displayLabel && (
                      <span className="break-words text-sm text-foreground">
                        {displayLabel.length > 40 ? `${displayLabel.slice(0, 40)}...` : displayLabel}
                      </span>
                    )}

                    {/* Depth indicator for deeply nested nodes */}
                    {depth > MAX_INDENT_DEPTH && (
                      <Badge variant="outline" className="h-4 shrink-0 px-1 py-0 text-[9px] text-muted-foreground">
                        {t('tree.levelPrefix')}{depth}
                      </Badge>
                    )}
                  </div>

                  {/* Node ID - hidden by default, shown on hover */}
                  <span className="shrink-0 self-center font-mono text-[10px] text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100">
                    {node.id.length > 8 ? `${node.id.slice(0, 8)}...` : node.id}
                  </span>
                </div>
              </TooltipTrigger>
              <TooltipContent side="right" className="max-w-xs">
                <div className="space-y-1 text-xs">
                  <div><span className="text-muted-foreground">{t('node.id')}:</span> <span className="font-mono">{node.id}</span></div>
                  {node.role && <div><span className="text-muted-foreground">{t('node.role')}:</span> {node.role}</div>}
                  {node.testId && <div><span className="text-muted-foreground">{t('node.testId')}:</span> {node.testId}</div>}
                  {displayLabel && <div><span className="text-muted-foreground">{t('node.label')}:</span> {displayLabel}</div>}
                  <div><span className="text-muted-foreground">{t('tree.depth')}:</span> {depth}</div>
                  {node.children.length > 0 && <div><span className="text-muted-foreground">{t('node.children')}:</span> {node.children.length}</div>}
                </div>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={handleCopySelector}>
            {t('context.copySelector')}
          </ContextMenuItem>
          <ContextMenuItem onClick={handleCopyNodePath}>
            {t('context.copyPath')}
          </ContextMenuItem>
          <ContextMenuItem onClick={handleCopyNodeId}>
            {t('context.copyId')}
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem onClick={handleCopyScriptStep}>
            {t('context.generateScript')}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>

      {/* Render children */}
      {hasChildren && isExpanded && (
        <div role="group">
          {node.children.map((childId) => {
            const childNode = semantics.nodesById[childId]
            if (!childNode) return null
            return (
              <MemoizedTreeNode
                key={childId}
                node={childNode}
                semantics={semantics}
                depth={depth + 1}
                redactText={redactText}
                searchQuery={searchQuery}
                onToggle={onToggle}
                onSelect={onSelect}
              />
            )
          })}
        </div>
      )}
    </div>
  )
}

// Wrapper component that gets state from store
function TreeNodeWithStore(props: Omit<TreeNodeProps, 'isExpanded' | 'isSelected'>) {
  const expandedNodes = useBundleStore((s) => s.expandedNodes)
  const selectedNodeId = useBundleStore((s) => s.selectedNodeId)

  return (
    <TreeNode
      {...props}
      isExpanded={expandedNodes.has(props.node.id)}
      isSelected={selectedNodeId === props.node.id}
    />
  )
}

const MemoizedTreeNode = TreeNodeWithStore

// Helper to find all matching node IDs
function findMatchingNodes(semantics: SemanticsModel, query: string): string[] {
  if (!query) return []
  const q = query.toLowerCase()
  const matches: string[] = []
  for (const node of Object.values(semantics.nodesById)) {
    if (
      node.id.toLowerCase().includes(q) ||
      node.testId?.toLowerCase().includes(q) ||
      node.role?.toLowerCase().includes(q) ||
      node.label?.toLowerCase().includes(q) ||
      node.name?.toLowerCase().includes(q)
    ) {
      matches.push(node.id)
    }
  }
  return matches
}

// Helper to get all ancestors of a node
function getAncestors(nodeId: string, semantics: SemanticsModel): string[] {
  const ancestors: string[] = []
  let current = semantics.nodesById[nodeId]
  while (current?.parentId) {
    ancestors.push(current.parentId)
    current = semantics.nodesById[current.parentId]
  }
  return ancestors
}

export function SemanticsTreePanel() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const redactText = useBundleStore((s) => s.redactText)
  const searchQuery = useBundleStore((s) => s.searchQuery)
  const setSearchQuery = useBundleStore((s) => s.setSearchQuery)
  const toggleNodeExpanded = useBundleStore((s) => s.toggleNodeExpanded)
  const setSelectedNodeId = useBundleStore((s) => s.setSelectedNodeId)
  const expandAllNodes = useBundleStore((s) => s.expandAllNodes)
  const collapseAllNodes = useBundleStore((s) => s.collapseAllNodes)
  const expandNode = useBundleStore((s) => s.expandNode)
  const { t } = useTranslation()

  const [localSearch, setLocalSearch] = useState('')
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0)
  const [showSearch, setShowSearch] = useState(false)
  const searchInputRef = useRef<HTMLInputElement>(null)

  const semantics = snapshot?.semantics

  // Find all matching nodes
  const matchingNodes = useMemo(() => {
    if (!semantics || !searchQuery) return []
    return findMatchingNodes(semantics, searchQuery)
  }, [semantics, searchQuery])

  // Auto-expand ancestors of matching nodes when search changes
  useEffect(() => {
    if (!semantics || matchingNodes.length === 0) return
    
    // Expand all ancestors of matching nodes
    const ancestorsToExpand = new Set<string>()
    for (const nodeId of matchingNodes) {
      const ancestors = getAncestors(nodeId, semantics)
      for (const ancestor of ancestors) {
        ancestorsToExpand.add(ancestor)
      }
    }
    
    for (const nodeId of ancestorsToExpand) {
      expandNode(nodeId)
    }
    
    // Reset to first match
    setCurrentMatchIndex(0)
  }, [searchQuery, matchingNodes, semantics, expandNode])

  // Jump to current match
  const jumpToMatch = useCallback((index: number) => {
    if (matchingNodes.length === 0) return
    const safeIndex = ((index % matchingNodes.length) + matchingNodes.length) % matchingNodes.length
    setCurrentMatchIndex(safeIndex)
    const nodeId = matchingNodes[safeIndex]
    setSelectedNodeId(nodeId)
    
    // Scroll the node into view
    setTimeout(() => {
      const element = document.querySelector(`[data-node-id="${nodeId}"]`)
      element?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }, 50)
  }, [matchingNodes, setSelectedNodeId])

  const handleNextMatch = useCallback(() => {
    jumpToMatch(currentMatchIndex + 1)
  }, [currentMatchIndex, jumpToMatch])

  const handlePrevMatch = useCallback(() => {
    jumpToMatch(currentMatchIndex - 1)
  }, [currentMatchIndex, jumpToMatch])

  // Keyboard shortcut to open search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        // Let header handle this
      }
      // Enter/Shift+Enter to navigate matches when search is focused
      if (showSearch && document.activeElement === searchInputRef.current) {
        if (e.key === 'Enter') {
          e.preventDefault()
          if (e.shiftKey) {
            handlePrevMatch()
          } else {
            handleNextMatch()
          }
        }
        if (e.key === 'Escape') {
          setShowSearch(false)
          setLocalSearch('')
          setSearchQuery('')
        }
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [showSearch, handleNextMatch, handlePrevMatch, setSearchQuery])

  // Sync local search with global when typing
  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchQuery(localSearch)
    }, 150)
    return () => clearTimeout(timer)
  }, [localSearch, setSearchQuery])

  // Sync global search to local when changed externally
  useEffect(() => {
    if (searchQuery && searchQuery !== localSearch) {
      setLocalSearch(searchQuery)
      setShowSearch(true)
    }
  }, [searchQuery])

  if (!semantics) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-muted/30">
          <h2 className="text-sm font-medium text-foreground">{t('tree.title')}</h2>
        </div>
        <div className="flex-1 flex items-center justify-center p-4">
          <p className="text-sm text-muted-foreground text-center">
            {t('tree.noData')}
          </p>
        </div>
      </div>
    )
  }

  const nodeCount = Object.keys(semantics.nodesById).length

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-muted/30">
        <div className="flex items-center gap-2 min-w-0">
          <h2 className="text-sm font-medium text-foreground shrink-0">{t('tree.title')}</h2>
          <Badge variant="secondary" className="text-[10px] shrink-0">
            {nodeCount}
          </Badge>
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant={showSearch ? 'secondary' : 'ghost'}
            size="icon"
            className="h-6 w-6"
            data-search-tree-button
            onClick={() => {
              setShowSearch(!showSearch)
              if (!showSearch) {
                setTimeout(() => searchInputRef.current?.focus(), 50)
              }
            }}
            title={t('tree.searchTree')}
          >
            <Search className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={expandAllNodes}
            title={t('tree.expandAll')}
          >
            <ChevronsUpDown className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={collapseAllNodes}
            title={t('tree.collapseAll')}
          >
            <ChevronsDownUp className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* Search bar */}
      {showSearch && (
        <div className="flex items-center gap-1.5 px-2 py-1.5 border-b border-border bg-muted/20">
          <div className="relative flex-1">
            <Search className="absolute left-2 top-1/2 h-3 w-3 -translate-y-1/2 text-muted-foreground" />
            <Input
              ref={searchInputRef}
              placeholder={t('tree.searchPlaceholder')}
              value={localSearch}
              onChange={(e) => setLocalSearch(e.target.value)}
              className="h-7 pl-7 pr-2 text-xs"
            />
          </div>
          {matchingNodes.length > 0 && (
            <>
              <span className="text-[10px] text-muted-foreground whitespace-nowrap">
                {currentMatchIndex + 1}/{matchingNodes.length}
              </span>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={handlePrevMatch}
                title={t('tree.previousMatch')}
              >
                <ChevronUp className="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={handleNextMatch}
                title={t('tree.nextMatch')}
              >
                <ChevronDown className="h-3.5 w-3.5" />
              </Button>
            </>
          )}
          {localSearch && matchingNodes.length === 0 && (
            <span className="text-[10px] text-amber-600">{t('tree.noMatches')}</span>
          )}
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={() => {
              setShowSearch(false)
              setLocalSearch('')
              setSearchQuery('')
            }}
            title={t('tree.closeSearch')}
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        </div>
      )}

      <ScrollArea className="flex-1" type="always">
        <div className="min-w-max">
          <div className="p-2" role="tree">
            {semantics.roots.map((rootId) => {
              const rootNode = semantics.nodesById[rootId]
              if (!rootNode) return null
              return (
                <MemoizedTreeNode
                  key={rootId}
                  node={rootNode}
                  semantics={semantics}
                  depth={0}
                  redactText={redactText}
                  searchQuery={searchQuery}
                  onToggle={toggleNodeExpanded}
                  onSelect={setSelectedNodeId}
                />
              )
            })}
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
