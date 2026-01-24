'use client'

import React from "react"

import { useRef, useCallback, useEffect, useState } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { formatFileSize } from '@/lib/download'
import { exportTriageJson, generateMarkdownSummary, downloadMarkdown } from '@/lib/download'
import { CommandPalette } from '@/components/command-palette'
import { ThemeToggle } from '@/components/theme-toggle'
import { LanguageSwitcher } from '@/components/language-switcher'
import { useTranslation } from '@/hooks/use-i18n'
import { extractBundleJsonFromZipFile } from '@/lib/zip'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import {
  FolderOpen,
  ClipboardPaste,
  Download,
  FileJson,
  AlertTriangle,
  ChevronDown,
  Command,
} from 'lucide-react'

export function HeaderBar() {
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [commandOpen, setCommandOpen] = useState(false)
  const [pasteOpen, setPasteOpen] = useState(false)
  const [pasteText, setPasteText] = useState('')
  const { t } = useTranslation()

  const {
    bundle,
    loadBundle,
    loadSampleBundle,
    compareMode,
    setCompareMode,
    redactText,
    setRedactText,
    selectedWindowIndex,
    selectedSnapshotAIndex,
    selectedSnapshotBIndex,
    selectedNodeId,
  } = useBundleStore()

  const handleFileOpen = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0]
      if (!file) return

      const isZip = /\.zip$/i.test(file.name)
      void (async () => {
        try {
          if (isZip) {
            const { text, bundlePath } = await extractBundleJsonFromZipFile(file)
            const derivedName = `${file.name.replace(/\.zip$/i, '')}-${bundlePath.split('/').pop() ?? 'bundle.json'}`
            loadBundle(text, { fileName: derivedName, fileSize: text.length, recordRecent: true })
          } else {
            const text = await file.text()
            loadBundle(text, { fileName: file.name, fileSize: file.size, recordRecent: true })
          }
        } finally {
          // Reset input so same file can be reloaded
          event.target.value = ''
        }
      })()
    },
    [loadBundle]
  )

  const handleExportSummary = useCallback(() => {
    if (!bundle) return
    const markdown = generateMarkdownSummary(
      bundle,
      selectedWindowIndex,
      selectedSnapshotAIndex,
      selectedSnapshotBIndex,
      selectedNodeId
    )
    const fileName = bundle.meta.fileName
      ? bundle.meta.fileName.replace('.json', '-summary.md')
      : 'bundle-summary.md'
    downloadMarkdown(markdown, fileName)
  }, [bundle, selectedWindowIndex, selectedSnapshotAIndex, selectedSnapshotBIndex, selectedNodeId])

  const handleExportTriage = useCallback(() => {
    if (!bundle) return
    exportTriageJson(
      bundle,
      selectedWindowIndex,
      selectedSnapshotAIndex,
      selectedSnapshotBIndex,
      selectedNodeId
    )
  }, [bundle, selectedWindowIndex, selectedSnapshotAIndex, selectedSnapshotBIndex, selectedNodeId])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd + O: Open file
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault()
        handleFileOpen()
      }
      // Ctrl/Cmd + K: Open command palette
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault()
        setCommandOpen(true)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleFileOpen])

  return (
    <TooltipProvider>
      <header className="flex flex-wrap items-center gap-2 border-b border-border bg-card px-3 py-2 lg:flex-nowrap lg:gap-3">
        {/* Title - compact */}
        <Tooltip>
          <TooltipTrigger asChild>
            <h1 className="shrink-0 text-sm font-semibold text-foreground">
              {t('app.title')}
            </h1>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t('app.description')}</p>
          </TooltipContent>
        </Tooltip>

        {/* Divider - hidden on mobile */}
        <div className="hidden h-6 w-px bg-border lg:block" />

        {/* File Actions - icon only on mobile */}
        <div className="flex items-center gap-1.5">
          <input
            ref={fileInputRef}
            type="file"
            accept=".json,.zip"
            className="hidden"
            onChange={handleFileChange}
          />

          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="outline" size="sm" className="h-8 bg-transparent px-2 lg:px-3" onClick={handleFileOpen}>
                <FolderOpen className="h-4 w-4" />
                <span className="ml-1.5 hidden lg:inline">{t('header.open')}</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t('header.openTooltip')}</p>
            </TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="h-8 bg-transparent px-2 lg:px-3"
                onClick={() => setPasteOpen(true)}
              >
                <ClipboardPaste className="h-4 w-4" />
                <span className="ml-1.5 hidden lg:inline">{t('header.paste')}</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t('header.pasteTooltip')}</p>
            </TooltipContent>
          </Tooltip>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="h-8 bg-transparent px-2 lg:px-3">
                <FileJson className="h-4 w-4" />
                <span className="ml-1.5 hidden lg:inline">{t('header.sample')}</span>
                <ChevronDown className="ml-1 h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={() => loadSampleBundle('simple')}>
                {t('header.sampleSimple')}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => loadSampleBundle('multi-window')}>
                {t('header.sampleMultiWindow')}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="h-8 bg-transparent px-2 lg:px-3"
                onClick={handleExportSummary}
                disabled={!bundle}
              >
                <Download className="h-4 w-4" />
                <span className="ml-1.5 hidden lg:inline">{t('header.export')}</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t('header.exportTooltip')}</p>
            </TooltipContent>
          </Tooltip>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="h-8 bg-transparent px-2 lg:px-3"
                disabled={!bundle}
              >
                <ChevronDown className="h-4 w-4" />
                <span className="ml-1.5 hidden lg:inline">{t('header.exportMore')}</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={handleExportSummary} disabled={!bundle}>
                {t('header.exportMarkdown')}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleExportTriage} disabled={!bundle}>
                {t('header.exportTriage')}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* Divider */}
        <div className="hidden h-6 w-px bg-border lg:block" />

        {/* Toggles - compact labels */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <Switch
              id="redact-text"
              checked={redactText}
              onCheckedChange={setRedactText}
              className="scale-90"
            />
            <Label htmlFor="redact-text" className="cursor-pointer text-xs">
              {t('header.redact')}
            </Label>
          </div>

          <div className="flex items-center gap-1.5">
            <Switch
              id="compare-mode"
              checked={compareMode}
              onCheckedChange={setCompareMode}
              className="scale-90"
            />
            <Label htmlFor="compare-mode" className="cursor-pointer text-xs">
              {t('header.compare')}
            </Label>
          </div>
        </div>

        {/* Divider */}
        <div className="hidden h-6 w-px bg-border lg:block" />

        {/* Command Palette Trigger */}
        <Button
          variant="outline"
          size="sm"
          className="h-8 gap-2 bg-transparent text-muted-foreground lg:min-w-48"
          onClick={() => setCommandOpen(true)}
        >
          <Command className="h-3.5 w-3.5" />
          <span className="hidden text-xs lg:inline">{t('header.searchCommands')}</span>
          <kbd className="ml-auto hidden rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] lg:inline">
            Ctrl+K
          </kbd>
        </Button>

        {/* Spacer - only on large screens */}
        <div className="hidden flex-1 lg:block" />

        {/* Status Area - compact */}
        {bundle && (
          <div className="flex items-center gap-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <FileJson className="h-3.5 w-3.5" />
                  <span className="max-w-24 truncate font-medium lg:max-w-40">
                    {bundle.meta.fileName ?? 'Unknown'}
                  </span>
                </div>
              </TooltipTrigger>
              <TooltipContent>
                <p>{bundle.meta.fileName} ({formatFileSize(bundle.meta.fileSize ?? 0)})</p>
              </TooltipContent>
            </Tooltip>

            {bundle.warnings.length > 0 && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Badge variant="outline" className="h-6 gap-1 border-amber-500/30 bg-amber-500/10 px-1.5 text-amber-600 dark:text-amber-400">
                    <AlertTriangle className="h-3 w-3" />
                    <span className="text-xs">{bundle.warnings.length}</span>
                  </Badge>
                </TooltipTrigger>
                <TooltipContent className="max-w-sm">
                  <ul className="space-y-1 text-xs">
                    {bundle.warnings.slice(0, 5).map((w, i) => (
                      <li key={i}>{w}</li>
                    ))}
                    {bundle.warnings.length > 5 && (
                      <li>...and {bundle.warnings.length - 5} more</li>
                    )}
                  </ul>
                </TooltipContent>
              </Tooltip>
            )}
          </div>
        )}

        {/* Language Switcher */}
        <LanguageSwitcher />

        {/* Theme Toggle */}
        <ThemeToggle />
      </header>

      {/* Command Palette */}
      <CommandPalette
        open={commandOpen}
        onOpenChange={setCommandOpen}
        onOpenFile={handleFileOpen}
      />

      <Dialog
        open={pasteOpen}
        onOpenChange={(open) => {
          setPasteOpen(open)
          if (!open) setPasteText('')
        }}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>{t('paste.title')}</DialogTitle>
            <DialogDescription>{t('paste.description')}</DialogDescription>
          </DialogHeader>
          <Textarea
            value={pasteText}
            onChange={(e) => setPasteText(e.target.value)}
            placeholder={t('paste.placeholder')}
            className="min-h-56 font-mono text-xs"
          />
          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              onClick={() => setPasteOpen(false)}
            >
              {t('paste.cancel')}
            </Button>
            <Button
              onClick={() => {
                const text = pasteText.trim()
                if (!text) return
                loadBundle(text, { fileName: 'pasted.json', fileSize: text.length, recordRecent: false })
                setPasteOpen(false)
              }}
              disabled={!pasteText.trim()}
            >
              {t('paste.load')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </TooltipProvider>
  )
}
