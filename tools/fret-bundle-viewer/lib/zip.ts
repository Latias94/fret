'use client'

import { strFromU8, unzipSync } from 'fflate'
import type { ZipArtifact, ZipScreenshot } from './types'
import { LocalizedError } from './localized-error'

function pickBundleJsonPath(entries: Record<string, Uint8Array>): string | null {
  const names = Object.keys(entries).filter((n) => !n.endsWith('/'))
  const candidates = names.filter((n) => n.toLowerCase().endsWith('bundle.json'))
  if (candidates.length === 0) return null

  const exact = candidates.find((n) => n.toLowerCase() === 'bundle.json')
  if (exact) return exact

  return candidates.sort((a, b) => a.length - b.length || a.localeCompare(b))[0] ?? null
}

export async function extractBundleJsonFromZipFile(file: File): Promise<{
  text: string
  bundlePath: string
}> {
  const { bundleText, bundlePathInZip } = await extractBundleAndArtifactsFromZipFile(file)
  return { text: bundleText, bundlePath: bundlePathInZip }
}

const MAX_ARTIFACT_BYTES = 1024 * 1024
const MAX_SCREENSHOT_BYTES = 64 * 1024 * 1024

function pickArtifacts(
  entries: Record<string, Uint8Array>,
  bundlePathInZip: string
): ZipArtifact[] {
  const names = Object.keys(entries).filter((n) => !n.endsWith('/'))

  const bundleDir = bundlePathInZip.includes('/')
    ? bundlePathInZip.split('/').slice(0, -1).join('/')
    : ''

  const preferredPrefix = bundleDir ? `${bundleDir}/_root/` : `_root/`
  const preferred = names.filter(
    (n) => n.startsWith(preferredPrefix) && n.toLowerCase().endsWith('.json')
  )

  const fallback = preferred.length > 0
    ? []
    : names.filter((n) => n.includes('/_root/') && n.toLowerCase().endsWith('.json'))

  const candidates = (preferred.length > 0 ? preferred : fallback)
    .slice()
    .sort((a, b) => a.length - b.length || a.localeCompare(b))

  const artifacts: ZipArtifact[] = []
  for (const path of candidates) {
    const payload = entries[path]
    if (!payload) continue
    const sizeBytes = payload.length
    const truncated = sizeBytes > MAX_ARTIFACT_BYTES
    const bytes = truncated ? payload.subarray(0, MAX_ARTIFACT_BYTES) : payload
    artifacts.push({
      path,
      fileName: path.split('/').pop() ?? path,
      sizeBytes,
      truncated: truncated || undefined,
      text: strFromU8(bytes),
    })
  }

  return artifacts
}

function pickScreenshots(
  entries: Record<string, Uint8Array>,
  bundlePathInZip: string
): ZipScreenshot[] {
  const names = Object.keys(entries).filter((n) => !n.endsWith('/'))

  const bundleDir = bundlePathInZip.includes('/')
    ? bundlePathInZip.split('/').slice(0, -1).join('/')
    : ''

  const manifestPath = pickScreenshotManifestPath(entries, bundlePathInZip)
  const metaByFile = manifestPath ? parseScreenshotManifest(entries[manifestPath]) : {}

  const preferredPrefixes = [
    bundleDir ? `${bundleDir}/_root/screenshots/` : `_root/screenshots/`,
    bundleDir ? `${bundleDir}/screenshots/` : `screenshots/`,
  ]

  const preferred = names.filter((n) => {
    const lower = n.toLowerCase()
    if (!lower.endsWith('.png')) return false
    return preferredPrefixes.some((p) => n.startsWith(p))
  })

  const fallback = preferred.length > 0
    ? []
    : names.filter((n) => n.toLowerCase().endsWith('.png') && n.includes('/screenshots/'))

  const candidates = (preferred.length > 0 ? preferred : fallback)
    .slice()
    .sort((a, b) => a.length - b.length || a.localeCompare(b))

  const screenshots: ZipScreenshot[] = []
  for (const path of candidates) {
    const payload = entries[path]
    if (!payload) continue
    const sizeBytes = payload.length
    if (sizeBytes > MAX_SCREENSHOT_BYTES) continue

    const blob = new Blob([payload], { type: 'image/png' })
    const objectUrl = URL.createObjectURL(blob)
    const fileName = path.split('/').pop() ?? path
    screenshots.push({
      path,
      fileName,
      sizeBytes,
      objectUrl,
      meta: metaByFile[fileName],
    })
  }

  return screenshots
}

function pickScreenshotManifestPath(
  entries: Record<string, Uint8Array>,
  bundlePathInZip: string
): string | null {
  const names = Object.keys(entries).filter((n) => !n.endsWith('/'))

  const bundleDir = bundlePathInZip.includes('/')
    ? bundlePathInZip.split('/').slice(0, -1).join('/')
    : ''

  const preferredCandidates = [
    bundleDir ? `${bundleDir}/_root/screenshots/manifest.json` : `_root/screenshots/manifest.json`,
    bundleDir ? `${bundleDir}/screenshots/manifest.json` : `screenshots/manifest.json`,
  ]
  for (const c of preferredCandidates) {
    if (names.includes(c)) return c
  }

  const fallback = names
    .filter((n) => n.toLowerCase().endsWith('/screenshots/manifest.json'))
    .sort((a, b) => a.length - b.length || a.localeCompare(b))
  return fallback[0] ?? null
}

function parseScreenshotManifest(payload?: Uint8Array): Record<string, ZipScreenshot['meta']> {
  if (!payload) return {}
  try {
    const text = strFromU8(payload)
    const data = JSON.parse(text) as unknown
    if (!data || typeof data !== 'object') return {}

    const schemaVersion = (data as any).schema_version
    if (schemaVersion !== 1) return {}

    const images = (data as any).images
    if (!Array.isArray(images)) return {}

    const out: Record<string, ZipScreenshot['meta']> = {}
    for (const img of images) {
      const file = img?.file
      if (typeof file !== 'string' || !file) continue
      const meta: ZipScreenshot['meta'] = {}
      if (img.window != null) meta.windowId = String(img.window)
      if (img.tick_id != null) meta.tickId = String(img.tick_id)
      if (img.frame_id != null) meta.frameId = String(img.frame_id)
      if (typeof img.scale_factor === 'number') meta.scaleFactor = img.scale_factor
      if (typeof img.width_px === 'number') meta.widthPx = img.width_px
      if (typeof img.height_px === 'number') meta.heightPx = img.height_px
      out[file] = meta
    }
    return out
  } catch {
    return {}
  }
}

export async function extractBundleAndArtifactsFromZipFile(file: File): Promise<{
  bundleText: string
  bundlePathInZip: string
  artifacts: ZipArtifact[]
  screenshots: ZipScreenshot[]
}> {
  const buf = await file.arrayBuffer()
  const entries = unzipSync(new Uint8Array(buf))
  const bundlePathInZip = pickBundleJsonPath(entries)
  if (!bundlePathInZip) {
    throw new LocalizedError('error.zipNoBundleJson')
  }

  const payload = entries[bundlePathInZip]
  if (!payload) {
    throw new LocalizedError('error.zipMissingEntry', { params: { path: bundlePathInZip } })
  }

  const artifacts = pickArtifacts(entries, bundlePathInZip)
  const screenshots = pickScreenshots(entries, bundlePathInZip)
  return { bundleText: strFromU8(payload), bundlePathInZip, artifacts, screenshots }
}
