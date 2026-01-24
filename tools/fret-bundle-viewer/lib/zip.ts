'use client'

import { strFromU8, unzipSync } from 'fflate'

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
  const buf = await file.arrayBuffer()
  const entries = unzipSync(new Uint8Array(buf))
  const bundlePath = pickBundleJsonPath(entries)
  if (!bundlePath) {
    throw new Error('No bundle.json found in zip')
  }

  const payload = entries[bundlePath]
  if (!payload) {
    throw new Error(`Missing zip entry: ${bundlePath}`)
  }

  return { text: strFromU8(payload), bundlePath }
}

