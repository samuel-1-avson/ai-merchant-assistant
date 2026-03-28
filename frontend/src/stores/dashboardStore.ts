/**
 * Dashboard Store — Zustand
 *
 * Manages dashboard data: analytics, transactions, products, and the AI voice
 * transaction flow (including the pending-confirmation workflow).
 */

import { create } from 'zustand'
import { Transaction, Product, AnalyticsSummary, PendingConfirmation, VoiceTransactionResult } from '@/types'
import { transactionsApi, productsApi, analyticsApi, confirmationApi } from '@/lib/api/client'

interface DashboardState {
  // Analytics
  analytics: AnalyticsSummary | null
  analyticsLoading: boolean
  insights: {
    summary: string
    health_score: number
    health_label: string
    revenue: number
    revenue_change_percent: number
    average_transaction_value: number
    average_daily_revenue: number
    transactions_per_day: number
    average_profit_margin_pct: number | null
    top_sellers: Array<{ product_name: string; times_sold: number; total_revenue: number; performance_label: string }>
    slow_movers: Array<{ product_name: string; times_sold: number; performance_label: string }>
    no_sales_products: Array<{ product_name: string }>
    recommendations: Array<{ type: string; priority: string; message: string }>
    alerts: Array<{ type: string; severity: string; message: string }>
  } | null
  insightsLoading: boolean

  // Transactions
  transactions: Transaction[]
  transactionsLoading: boolean
  transactionsMeta: { limit: number; offset: number; count: number } | null

  // Products
  products: Product[]
  productsLoading: boolean

  // Voice transaction (immediate result)
  lastVoiceTransaction: Transaction | null
  lastTranscription: string | null

  // Pending confirmation (Fix 3 — confirmation UI)
  pendingConfirmation: PendingConfirmation | null
  confirmationLoading: boolean

  // ── Actions ──────────────────────────────────────────────────────────

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

  /**
   * Send recorded audio to the backend.
   *
   * Returns either:
   *   { type: 'immediate', transaction, transcription } — transaction committed
   *   { type: 'pending', pending_confirmation }         — needs user confirmation
   */
  createVoiceTransaction: (audioBase64: string) => Promise<(VoiceTransactionResult & { success: boolean })>

  clearLastVoiceTransaction: () => void

  /** Confirm the active pending transaction and commit it to the database. */
  confirmPendingTransaction: () => Promise<boolean>

  /** Reject the active pending transaction and discard it. */
  rejectPendingTransaction: () => Promise<boolean>

  clearPendingConfirmation: () => void
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
  pendingConfirmation: null,
  confirmationLoading: false,

  // ── Analytics ────────────────────────────────────────────────────────

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

  fetchInsights: async () => {
    set({ insightsLoading: true })
    try {
      const response = await analyticsApi.getInsights()
      if (response.success && response.data) {
        set({
          insights: {
            summary: response.data.summary,
            health_score: response.data.health_score,
            health_label: response.data.health_label,
            revenue: response.data.revenue,
            revenue_change_percent: response.data.revenue_change_percent,
            average_transaction_value: response.data.average_transaction_value,
            average_daily_revenue: response.data.average_daily_revenue,
            transactions_per_day: response.data.transactions_per_day,
            average_profit_margin_pct: response.data.average_profit_margin_pct,
            top_sellers: response.data.top_sellers ?? [],
            slow_movers: response.data.slow_movers ?? [],
            no_sales_products: response.data.no_sales_products ?? [],
            recommendations: response.data.recommendations ?? [],
            alerts: response.data.alerts ?? [],
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

  // ── Transactions ──────────────────────────────────────────────────────

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

  // ── Products ──────────────────────────────────────────────────────────

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

  createTransaction: async (data) => {
    try {
      const response = await transactionsApi.create(data)
      if (response.success) {
        get().fetchTransactions()
        get().fetchAnalytics()
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to create transaction:', error)
      return false
    }
  },

  // ── Voice transaction ─────────────────────────────────────────────────

  createVoiceTransaction: async (audioBase64: string) => {
    try {
      const response = await transactionsApi.createVoice(audioBase64)

      if (!response.success || !response.data) {
        return { success: false, type: 'immediate' as const }
      }

      const result = response.data

      if (result.type === 'pending' && result.pending_confirmation) {
        // Backend needs user confirmation before committing
        set({ pendingConfirmation: result.pending_confirmation })
        return { success: true, ...result }
      }

      // Transaction committed immediately
      if (result.transaction) {
        set({
          lastVoiceTransaction: result.transaction,
          lastTranscription: result.transcription ?? null,
          pendingConfirmation: null,
        })
        get().fetchTransactions()
        get().fetchAnalytics()
      }

      return { success: true, ...result }
    } catch (error) {
      console.error('Failed to create voice transaction:', error)
      return { success: false, type: 'immediate' as const }
    }
  },

  clearLastVoiceTransaction: () => {
    set({ lastVoiceTransaction: null, lastTranscription: null })
  },

  // ── Pending confirmation ──────────────────────────────────────────────

  confirmPendingTransaction: async () => {
    const { pendingConfirmation } = get()
    if (!pendingConfirmation) return false

    set({ confirmationLoading: true })
    try {
      const response = await confirmationApi.confirm(pendingConfirmation.id)
      if (response.success && response.data) {
        set({
          lastVoiceTransaction: response.data,
          pendingConfirmation: null,
          confirmationLoading: false,
        })
        get().fetchTransactions()
        get().fetchAnalytics()
        return true
      }
      set({ confirmationLoading: false })
      return false
    } catch (error) {
      console.error('Failed to confirm transaction:', error)
      set({ confirmationLoading: false })
      return false
    }
  },

  rejectPendingTransaction: async () => {
    const { pendingConfirmation } = get()
    if (!pendingConfirmation) return false

    set({ confirmationLoading: true })
    try {
      const response = await confirmationApi.reject(pendingConfirmation.id)
      set({ pendingConfirmation: null, confirmationLoading: false })
      return response.success
    } catch (error) {
      console.error('Failed to reject transaction:', error)
      set({ confirmationLoading: false, pendingConfirmation: null })
      return false
    }
  },

  clearPendingConfirmation: () => {
    set({ pendingConfirmation: null })
  },
}))
