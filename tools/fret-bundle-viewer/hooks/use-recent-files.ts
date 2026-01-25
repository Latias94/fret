'use client'

import { useState, useEffect, useCallback } from 'react'

export interface RecentFile {
  fileName: string
  fileSize: number
  openedAt: number // timestamp
  // We store a hash/preview to help identify the file, not the full content
  contentPreview: string
}

const STORAGE_KEY = 'fret-viewer-recent-files'
const MAX_RECENT_FILES = 10

function getStoredFiles(): RecentFile[] {
  if (typeof window === 'undefined') return []
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return []
    return JSON.parse(stored) as RecentFile[]
  } catch {
    return []
  }
}

function storeFiles(files: RecentFile[]) {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(files))
  } catch {
    // localStorage might be full or disabled
  }
}

export function useRecentFiles() {
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([])

  // Load from localStorage on mount
  useEffect(() => {
    setRecentFiles(getStoredFiles())
  }, [])

  const addRecentFile = useCallback((fileName: string, fileSize: number, content: string) => {
    // Create a preview from the content (first 200 chars, useful for identification)
    const contentPreview = content.slice(0, 200)
    
    setRecentFiles((prev) => {
      // Remove if already exists (to move to top)
      const filtered = prev.filter(
        (f) => !(f.fileName === fileName && f.contentPreview === contentPreview)
      )
      
      const newFile: RecentFile = {
        fileName,
        fileSize,
        openedAt: Date.now(),
        contentPreview,
      }
      
      // Add to beginning, limit to MAX_RECENT_FILES
      const updated = [newFile, ...filtered].slice(0, MAX_RECENT_FILES)
      storeFiles(updated)
      return updated
    })
  }, [])

  const removeRecentFile = useCallback((index: number) => {
    setRecentFiles((prev) => {
      const updated = prev.filter((_, i) => i !== index)
      storeFiles(updated)
      return updated
    })
  }, [])

  const clearRecentFiles = useCallback(() => {
    setRecentFiles([])
    storeFiles([])
  }, [])

  return {
    recentFiles,
    addRecentFile,
    removeRecentFile,
    clearRecentFiles,
  }
}

// Format relative time
export function formatRelativeTime(timestamp: number, locale: 'en' | 'zh' = 'en'): string {
  const now = Date.now()
  const diff = now - timestamp
  
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) {
    if (locale === 'zh') {
      return days === 1 ? '昨天' : `${days} 天前`
    }
    return days === 1 ? 'Yesterday' : `${days} days ago`
  }
  if (hours > 0) {
    if (locale === 'zh') {
      return hours === 1 ? '1 小时前' : `${hours} 小时前`
    }
    return hours === 1 ? '1 hour ago' : `${hours} hours ago`
  }
  if (minutes > 0) {
    if (locale === 'zh') {
      return minutes === 1 ? '1 分钟前' : `${minutes} 分钟前`
    }
    return minutes === 1 ? '1 minute ago' : `${minutes} minutes ago`
  }
  return locale === 'zh' ? '刚刚' : 'Just now'
}
