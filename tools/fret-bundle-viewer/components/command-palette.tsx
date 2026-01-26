'use client'

import { useRef } from "react"

import { useCallback } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import { useRecentFiles, formatRelativeTime } from '@/hooks/use-recent-files'
import { useTranslation } from '@/hooks/use-i18n'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from '@/components/ui/command'
import {
  FolderOpen,
  Download,
  Search,
  FileJson,
  Eye,
  EyeOff,
  GitCompare,
  ChevronsUpDown,
  ChevronsDownUp,
  Layers,
  Monitor,
  Clock,
  History,
} from 'lucide-react'
import { generateMarkdownSummary, downloadMarkdown } from '@/lib/download'

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onOpenFile: () => void
}

export function CommandPalette({ open, onOpenChange, onOpenFile }: CommandPaletteProps) {
  const inputRef = useRef(null)
  const { recentFiles } = useRecentFiles()
  const { t } = useTranslation()
  
  const {
    bundle,
    loadSampleBundle,
    compareMode,
    setCompareMode,
    redactText,
    setRedactText,
    searchQuery,
    setSearchQuery,
    expandAllNodes,
    collapseAllNodes,
    selectedWindowIndex,
    setSelectedWindowIndex,
    selectedSnapshotAIndex,
    setSelectedSnapshotAIndex,
    selectedSnapshotBIndex,
  } = useBundleStore()

  const windows = bundle?.windows ?? []
  const snapshots = windows[selectedWindowIndex]?.snapshots ?? []

  const handleExportSummary = useCallback(() => {
    if (!bundle) return
    const markdown = generateMarkdownSummary(
      bundle,
      selectedWindowIndex,
      selectedSnapshotAIndex,
      selectedSnapshotBIndex,
      null
    )
    const fileName = bundle.meta.fileName
      ? bundle.meta.fileName.replace('.json', '-summary.md')
      : 'bundle-summary.md'
    downloadMarkdown(markdown, fileName)
    onOpenChange(false)
  }, [bundle, selectedWindowIndex, selectedSnapshotAIndex, selectedSnapshotBIndex, onOpenChange])

  const runCommand = useCallback((command: () => void) => {
    onOpenChange(false)
    command()
  }, [onOpenChange])

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder={t('cmd.placeholder')} />
      <CommandList>
        <CommandEmpty>{t('cmd.noResults')}</CommandEmpty>

        {/* File Actions */}
        <CommandGroup heading={t('cmd.file')}>
          <CommandItem onSelect={() => runCommand(onOpenFile)}>
            <FolderOpen className="h-4 w-4" />
            <span>{t('cmd.openBundle')}</span>
            <CommandShortcut>Ctrl+O</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => loadSampleBundle('simple'))}>
            <FileJson className="h-4 w-4" />
            <span>{t('cmd.loadSampleSimple')}</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => loadSampleBundle('multi-window'))}>
            <FileJson className="h-4 w-4" />
            <span>{t('cmd.loadSampleMulti')}</span>
          </CommandItem>
          {bundle && (
            <CommandItem onSelect={handleExportSummary}>
              <Download className="h-4 w-4" />
              <span>{t('cmd.exportSummary')}</span>
            </CommandItem>
          )}
        </CommandGroup>

        {/* Recent Files */}
        {recentFiles.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={t('cmd.recentFiles')}>
              {recentFiles.slice(0, 5).map((file, index) => (
                <CommandItem
                  key={`${file.fileName}-${file.openedAt}`}
                  onSelect={() => runCommand(onOpenFile)}
                >
                  <History className="h-4 w-4" />
                  <span className="truncate">{file.fileName}</span>
                  <span className="ml-auto text-xs text-muted-foreground">
                    {formatRelativeTime(file.openedAt)}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}

        <CommandSeparator />

        {/* View Options */}
        <CommandGroup heading={t('cmd.view')}>
          <CommandItem onSelect={() => runCommand(() => setRedactText(!redactText))}>
            {redactText ? <Eye className="h-4 w-4" /> : <EyeOff className="h-4 w-4" />}
            <span>{redactText ? t('cmd.showText') : t('cmd.redactText')}</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(() => setCompareMode(!compareMode))}>
            <GitCompare className="h-4 w-4" />
            <span>{compareMode ? t('cmd.disableCompare') : t('cmd.enableCompare')}</span>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        {/* Tree Actions */}
        <CommandGroup heading={t('cmd.tree')}>
          <CommandItem onSelect={() => {
            onOpenChange(false)
            // Trigger search in semantics tree
            setSearchQuery('')
            setTimeout(() => {
              const searchBtn = document.querySelector('[data-search-tree-button]') as HTMLButtonElement
              searchBtn?.click()
            }, 50)
          }}>
            <Search className="h-4 w-4" />
            <span>{t('cmd.searchNodes')}</span>
            <CommandShortcut>Ctrl+F</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(expandAllNodes)}>
            <ChevronsUpDown className="h-4 w-4" />
            <span>{t('cmd.expandAll')}</span>
          </CommandItem>
          <CommandItem onSelect={() => runCommand(collapseAllNodes)}>
            <ChevronsDownUp className="h-4 w-4" />
            <span>{t('cmd.collapseAll')}</span>
          </CommandItem>
        </CommandGroup>

        {/* Window Navigation */}
        {windows.length > 1 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={t('cmd.windows')}>
              {windows.map((w, i) => (
                <CommandItem
                  key={w.windowId}
                  onSelect={() => runCommand(() => setSelectedWindowIndex(i))}
                >
                  <Monitor className="h-4 w-4" />
                  <span>
                    {w.windowId}
                    {i === selectedWindowIndex && ` ${t('cmd.current')}`}
                  </span>
                  <span className="ml-auto text-xs text-muted-foreground">
                    {w.snapshots.length} {t('cmd.snapshots')}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}

        {/* Quick Snapshot Navigation */}
        {snapshots.length > 0 && (
          <>
            <CommandSeparator />
            <CommandGroup heading={t('cmd.quickJump')}>
              <CommandItem onSelect={() => runCommand(() => setSelectedSnapshotAIndex(0))}>
                <Layers className="h-4 w-4" />
                <span>{t('cmd.firstSnapshot')}</span>
              </CommandItem>
              <CommandItem onSelect={() => runCommand(() => setSelectedSnapshotAIndex(snapshots.length - 1))}>
                <Layers className="h-4 w-4" />
                <span>{t('cmd.lastSnapshot')}</span>
              </CommandItem>
              {/* Find slowest snapshot */}
              {(() => {
                let maxTime = 0
                let maxIdx = 0
                snapshots.forEach((s, i) => {
                  const time = s.perf?.totalUs ?? 0
                  if (time > maxTime) {
                    maxTime = time
                    maxIdx = i
                  }
                })
                if (maxTime > 0) {
                  return (
                    <CommandItem onSelect={() => runCommand(() => setSelectedSnapshotAIndex(maxIdx))}>
                      <Clock className="h-4 w-4" />
                      <span>{t('cmd.slowestSnapshot', { index: maxIdx + 1 })}</span>
                      <span className="ml-auto text-xs text-muted-foreground">
                        {formatMicroseconds(maxTime)}
                      </span>
                    </CommandItem>
                  )
                }
                return null
              })()}
            </CommandGroup>
          </>
        )}
      </CommandList>
    </CommandDialog>
  )
}

function formatMicroseconds(us: number): string {
  if (us < 1000) return `${us}μs`
  if (us < 1_000_000) return `${(us / 1000).toFixed(1)}ms`
  return `${(us / 1_000_000).toFixed(2)}s`
}
