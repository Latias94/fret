'use client'

import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { translations, type Locale, type TranslationKey } from '@/lib/i18n'

interface I18nState {
  locale: Locale
  setLocale: (locale: Locale) => void
}

export const useI18nStore = create<I18nState>()(
  persist(
    (set) => ({
      locale: 'en',
      setLocale: (locale) => set({ locale }),
    }),
    {
      name: 'fret-viewer-locale',
    }
  )
)

export function useTranslation() {
  const { locale, setLocale } = useI18nStore()

  const t = (key: TranslationKey, params?: Record<string, string | number>): string => {
    let text = translations[locale][key] || translations.en[key] || key
    
    if (params) {
      Object.entries(params).forEach(([k, v]) => {
        text = text.replaceAll(`{${k}}`, String(v))
      })
    }
    
    return text
  }

  return { t, locale, setLocale }
}
