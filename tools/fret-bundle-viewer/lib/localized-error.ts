import type { TranslationKey } from './i18n'

export interface LocalizedErrorLike {
  key: TranslationKey
  params?: Record<string, string | number>
  detail?: string
}

export class LocalizedError extends Error implements LocalizedErrorLike {
  key: TranslationKey
  params?: Record<string, string | number>
  detail?: string

  constructor(key: TranslationKey, opts?: { params?: Record<string, string | number>; detail?: string; cause?: unknown }) {
    super(key, { cause: opts?.cause })
    this.name = 'LocalizedError'
    this.key = key
    this.params = opts?.params
    this.detail = opts?.detail
  }
}

export function isLocalizedErrorLike(err: unknown): err is LocalizedErrorLike {
  if (!err || typeof err !== 'object') return false
  return typeof (err as { key?: unknown }).key === 'string'
}

