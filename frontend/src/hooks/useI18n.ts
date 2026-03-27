'use client'

import { useState, useEffect, useCallback } from 'react'

interface I18nState {
  language: string
  translations: Record<string, string>
  isLoading: boolean
}

const SUPPORTED_LANGUAGES = ['en', 'es', 'fr', 'de', 'zh', 'ar']

export function useI18n() {
  const [state, setState] = useState<I18nState>({
    language: 'en',
    translations: {},
    isLoading: true,
  })

  useEffect(() => {
    // Load saved language preference
    const savedLang = localStorage.getItem('language') || 'en'
    setLanguage(savedLang)
  }, [])

  const setLanguage = useCallback(async (lang: string) => {
    if (!SUPPORTED_LANGUAGES.includes(lang)) {
      console.warn(`Language ${lang} not supported`)
      return
    }

    setState(prev => ({ ...prev, isLoading: true }))

    try {
      const response = await fetch(`http://localhost:3000/api/v1/i18n/translations?lang=${lang}`)
      const data = await response.json()

      if (data.success) {
        setState({
          language: lang,
          translations: data.data.strings,
          isLoading: false,
        })
        localStorage.setItem('language', lang)
        document.documentElement.lang = lang
        document.documentElement.dir = lang === 'ar' ? 'rtl' : 'ltr'
      }
    } catch (error) {
      console.error('Failed to load translations:', error)
      setState(prev => ({ ...prev, isLoading: false }))
    }
  }, [])

  const t = useCallback((key: string) => {
    return state.translations[key] || key
  }, [state.translations])

  const formatNumber = useCallback((num: number) => {
    return new Intl.NumberFormat(state.language, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(num)
  }, [state.language])

  const formatCurrency = useCallback((amount: number, currency = 'USD') => {
    return new Intl.NumberFormat(state.language, {
      style: 'currency',
      currency,
    }).format(amount)
  }, [state.language])

  return {
    ...state,
    supportedLanguages: SUPPORTED_LANGUAGES,
    setLanguage,
    t,
    formatNumber,
    formatCurrency,
  }
}
