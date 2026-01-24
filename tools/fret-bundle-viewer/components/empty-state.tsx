'use client'

import React from "react"

import { useBundleStore, setOnFileLoadedCallback } from '@/store/use-bundle-store'
import { useRecentFiles, formatRelativeTime, type RecentFile } from '@/hooks/use-recent-files'
import { formatFileSize } from '@/lib/download'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { FileJson, FolderOpen, ChevronDown, AlertCircle, Clock, X, Trash2 } from 'lucide-react'
import { useRef, useCallback, useEffect } from 'react'
import { useTranslation } from '@/hooks/use-i18n'
import { extractBundleJsonFromZipFile } from '@/lib/zip'

export function EmptyState() {
  const fileInputRef = useRef<HTMLInputElement>(null)
  const loadBundle = useBundleStore((s) => s.loadBundle)
  const loadSampleBundle = useBundleStore((s) => s.loadSampleBundle)
  const parseError = useBundleStore((s) => s.parseError)
  const rawText = useBundleStore((s) => s.rawText)
  const { t } = useTranslation()
  
  const { recentFiles, addRecentFile, removeRecentFile, clearRecentFiles } = useRecentFiles()

  // Register callback to track recent files
  useEffect(() => {
    setOnFileLoadedCallback(addRecentFile)
    return () => setOnFileLoadedCallback(null)
  }, [addRecentFile])

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
          event.target.value = ''
        }
      })()
    },
    [loadBundle]
  )

  if (parseError) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center">
        <div className="w-16 h-16 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
          <AlertCircle className="w-8 h-8 text-destructive" />
        </div>
        <h2 className="text-lg font-semibold text-foreground mb-2">{t('error.title')}</h2>
        <p className="text-sm text-muted-foreground mb-4 max-w-md">{parseError}</p>
        
        {rawText && (
          <div className="w-full max-w-lg mb-4">
            <p className="text-xs text-muted-foreground mb-2">{t('error.rawPreview')}</p>
            <pre className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-32 text-left">
              {rawText.slice(0, 500)}
              {rawText.length > 500 && '...'}
            </pre>
          </div>
        )}

        <div className="flex gap-2">
          <input
            ref={fileInputRef}
            type="file"
            accept=".json,.zip"
            className="hidden"
            onChange={handleFileChange}
          />
          <Button variant="outline" onClick={handleFileOpen}>
            <FolderOpen className="mr-2 h-4 w-4" />
            {t('error.tryAnother')}
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline">
                <FileJson className="mr-2 h-4 w-4" />
                {t('empty.loadSample')}
                <ChevronDown className="ml-2 h-3 w-3" />
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
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center justify-center h-full p-8 text-center">
      <div className="w-20 h-20 rounded-full bg-muted flex items-center justify-center mb-6">
        <FileJson className="w-10 h-10 text-muted-foreground" />
      </div>
      <h2 className="text-xl font-semibold text-foreground mb-2">{t('empty.title')}</h2>
      <p className="text-sm text-muted-foreground mb-6 max-w-md">
        {t('empty.description')}
      </p>
      
      <div className="flex gap-3">
        <input
          ref={fileInputRef}
          type="file"
          accept=".json,.zip"
          className="hidden"
          onChange={handleFileChange}
        />
        <Button onClick={handleFileOpen}>
          <FolderOpen className="mr-2 h-4 w-4" />
          {t('empty.openButton')}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline">
              <FileJson className="mr-2 h-4 w-4" />
              {t('empty.loadSample')}
              <ChevronDown className="ml-2 h-3 w-3" />
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
      </div>

      {/* Recent Files */}
      {recentFiles.length > 0 && (
        <div className="mt-8 w-full max-w-md">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-foreground flex items-center gap-2">
              <Clock className="h-4 w-4 text-muted-foreground" />
              {t('empty.recentFiles')}
            </h3>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs text-muted-foreground hover:text-foreground"
              onClick={clearRecentFiles}
            >
              <Trash2 className="h-3 w-3 mr-1" />
              {t('empty.clear')}
            </Button>
          </div>
          <div className="space-y-1">
            {recentFiles.map((file, index) => (
              <RecentFileItem
                key={`${file.fileName}-${file.openedAt}`}
                file={file}
                onOpen={handleFileOpen}
                onRemove={() => removeRecentFile(index)}
              />
            ))}
          </div>
          <p className="text-[10px] text-muted-foreground mt-3 text-center">
            {t('empty.recentFilesNote')}
          </p>
        </div>
      )}

      <p className="text-xs text-muted-foreground mt-8">
        {t('app.offline')}
      </p>
    </div>
  )
}

function RecentFileItem({
  file,
  onOpen,
  onRemove,
}: {
  file: RecentFile
  onOpen: () => void
  onRemove: () => void
}) {
  const { locale } = useTranslation()
  return (
    <div className="group flex items-center gap-2 rounded-md border border-border bg-card px-3 py-2 transition-colors hover:bg-accent">
      <FileJson className="h-4 w-4 shrink-0 text-muted-foreground" />
      <button
        type="button"
        className="flex flex-1 min-w-0 flex-col items-start text-left"
        onClick={onOpen}
      >
        <span className="text-sm font-medium text-foreground truncate w-full">
          {file.fileName}
        </span>
        <span className="text-xs text-muted-foreground">
          {formatFileSize(file.fileSize)} · {formatRelativeTime(file.openedAt, locale)}
        </span>
      </button>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
        onClick={(e) => {
          e.stopPropagation()
          onRemove()
        }}
      >
        <X className="h-3 w-3" />
      </Button>
    </div>
  )
}
