import type { SnapshotModel, SemanticsNodeModel, DiffResult } from './types'

export function diffSnapshots(a: SnapshotModel, b: SnapshotModel): DiffResult {
  const result: DiffResult = {
    added: [],
    removed: [],
    changed: [],
  }

  const nodesA = a.semantics?.nodesById ?? {}
  const nodesB = b.semantics?.nodesById ?? {}

  const idsA = new Set(Object.keys(nodesA))
  const idsB = new Set(Object.keys(nodesB))

  // Find removed nodes (in A but not in B)
  for (const id of idsA) {
    if (!idsB.has(id)) {
      result.removed.push(id)
    }
  }

  // Find added nodes (in B but not in A)
  for (const id of idsB) {
    if (!idsA.has(id)) {
      result.added.push(id)
    }
  }

  // Find changed nodes (in both but different)
  for (const id of idsA) {
    if (idsB.has(id)) {
      if (hasNodeChanged(nodesA[id], nodesB[id])) {
        result.changed.push(id)
      }
    }
  }

  return result
}

function hasNodeChanged(a: SemanticsNodeModel, b: SemanticsNodeModel): boolean {
  // Compare key properties
  if (a.role !== b.role) return true
  if (a.label !== b.label) return true
  if (a.name !== b.name) return true
  if (a.testId !== b.testId) return true

  // Compare bounds
  if (a.bounds || b.bounds) {
    if (!a.bounds || !b.bounds) return true
    if (
      a.bounds.x !== b.bounds.x ||
      a.bounds.y !== b.bounds.y ||
      a.bounds.w !== b.bounds.w ||
      a.bounds.h !== b.bounds.h
    ) {
      return true
    }
  }

  // Compare children
  if (a.children.length !== b.children.length) return true
  for (let i = 0; i < a.children.length; i++) {
    if (a.children[i] !== b.children[i]) return true
  }

  // Compare flags
  if (JSON.stringify(a.flags) !== JSON.stringify(b.flags)) return true

  // Compare actions
  if (JSON.stringify(a.actions) !== JSON.stringify(b.actions)) return true

  return false
}

export function getDiffSummary(diff: DiffResult): string {
  const parts: string[] = []
  if (diff.added.length > 0) {
    parts.push(`${diff.added.length} added`)
  }
  if (diff.removed.length > 0) {
    parts.push(`${diff.removed.length} removed`)
  }
  if (diff.changed.length > 0) {
    parts.push(`${diff.changed.length} changed`)
  }
  return parts.length > 0 ? parts.join(', ') : 'No changes'
}
