'use client'

import { strFromU8, unzipSync } from 'fflate'
import type { ZipArtifact } from './types'
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

export async function extractBundleAndArtifactsFromZipFile(file: File): Promise<{
  bundleText: string
  bundlePathInZip: string
  artifacts: ZipArtifact[]
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
  return { bundleText: strFromU8(payload), bundlePathInZip, artifacts }
}
