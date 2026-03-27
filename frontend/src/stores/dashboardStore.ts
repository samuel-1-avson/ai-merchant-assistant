/**
 * Dashboard Store - Zustand
 * Manages dashboard data: analytics, transactions, insights
 */

import { create } from 'zustand'
import { Transaction, Product, AnalyticsSummary } from '@/types'
import { transactionsApi, productsApi, analyticsApi } from '@/lib/api/client'

interface DashboardState {
  // Analytics State
  analytics: AnalyticsSummary | null
  analyticsLoading: boolean
  insights: {
    summary: string
    revenue_change_percent: number
    recommendations: string[]
    alerts: unknown[]
  } | null
  insightsLoading: boolean

  // Transactions State
  transactions: Transaction[]
  transactionsLoading: boolean
  transactionsMeta: { limit: number; offset: number; count: number } | null

  // Products State
  products: Product[]
  productsLoading: boolean

  // Voice Transaction State
  lastVoiceTransaction: Transaction | null
  lastTranscription: string | null

  // Actions
  fetchAnalytics: (days?: number) => Promise<void>
  fetchInsights: () => Promise<void>
  fetchTransactions: (params?: { limit?: number; offset?: number }) => Promise<void>
  fetchProducts: (search?: string) => Promise<void>
  createTransaction: (data: {
    product_id?: string
    quantity: number
    unit?: string
    price: number
    notes?: string
  }) => Promise<boolean>
  createVoiceTransaction: (audioBase64: string) => Promise<{ success: boolean; transcription?: string }>
  clearLastVoiceTransaction: () => void
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
  // Initial state
  analytics: null,
  analyticsLoading: false,
  insights: null,
  insightsLoading: false,
  transactions: [],
  transactionsLoading: false,
  transactionsMeta: null,
  products: [],
  productsLoading: false,
  lastVoiceTransaction: null,
  lastTranscription: null,

  // Fetch analytics
  fetchAnalytics: async (days = 7) => {
    set({ analyticsLoading: true })
    try {
      const response = await analyticsApi.getSummary(days)
      if (response.success && response.data) {
        set({ analytics: response.data, analyticsLoading: false })
      } else {
        set({ analyticsLoading: false })
      }
    } catch (error) {
      console.error('Failed to fetch analytics:', error)
      set({ analyticsLoading: false })
    }
  },

  // Fetch insights
  fetchInsights: async () => {
    set({ insightsLoading: true })
    try {
      const response = await analyticsApi.getInsights()
      if (response.success && response.data) {
        set({
          insights: {
            summary: response.data.summary,
            revenue_change_percent: response.data.revenue_change_percent,
            recommendations: response.data.recommendations,
            alerts: response.data.alerts,
          },
          insightsLoading: false,
        })
      } else {
        set({ insightsLoading: false })
      }
    } catch (error) {
      console.error('Failed to fetch insights:', error)
      set({ insightsLoading: false })
    }
  },

  // Fetch transactions
  fetchTransactions: async (params = { limit: 10, offset: 0 }) => {
    set({ transactionsLoading: true })
    try {
      const response = await transactionsApi.list(params)
      if (response.success && response.data) {
        set({
          transactions: response.data.transactions,
          transactionsMeta: response.data.meta,
          transactionsLoading: false,
        })
      } else {
        set({ transactionsLoading: false })
      }
    } catch (error) {
      console.error('Failed to fetch transactions:', error)
      set({ transactionsLoading: false })
    }
  },

  // Fetch products
  fetchProducts: async (search?: string) => {
    set({ productsLoading: true })
    try {
      const response = await productsApi.list(search)
      if (response.success && response.data) {
        set({ products: response.data.products, productsLoading: false })
      } else {
        set({ productsLoading: false })
      }
    } catch (error) {
      console.error('Failed to fetch products:', error)
      set({ productsLoading: false })
    }
  },

  // Create transaction
  createTransaction: async (data) => {
    try {
      const response = await transactionsApi.create(data)
      if (response.success) {
        // Refresh transactions list
        get().fetchTransactions()
        // Refresh analytics
        get().fetchAnalytics()
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to create transaction:', error)
      return false
    }
  },

  // Create voice transaction
  createVoiceTransaction: async (audioBase64: string) => {
    try {
      const response = await transactionsApi.createVoice(audioBase64)
      if (response.success && response.data) {
        set({
          lastVoiceTransaction: response.data.transaction,
          lastTranscription: response.data.transcription,
        })
        // Refresh transactions and analytics
        get().fetchTransactions()
        get().fetchAnalytics()
        return { success: true, transcription: response.data.transcription }
      }
      return { success: false }
    } catch (error) {
      console.error('Failed to create voice transaction:', error)
      return { success: false }
    }
  },

  // Clear last voice transaction
  clearLastVoiceTransaction: () => {
    set({ lastVoiceTransaction: null, lastTranscription: null })
  },
}))
