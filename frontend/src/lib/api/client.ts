/**
 * API Client for AI Merchant Assistant Backend
 * Base URL: http://localhost:3000/api/v1
 */

import { Transaction, Product, AnalyticsSummary, User, PendingConfirmation, VoiceTransactionResult } from '@/types'
import { useAuthStore } from '@/stores/authStore'

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8888/api/v1'

// Get auth token from the Zustand in-memory store (always fresh, no localStorage timing issues)
// Falls back to Supabase's own internal storage if the store token is null
function getAuthToken(): string | null {
  // 1. Zustand in-memory store — this is updated synchronously by onAuthStateChange
  const storeToken = useAuthStore.getState().token
  if (storeToken) return storeToken

  // 2. Supabase's own internal storage (key: sb-<projectRef>-auth-token)
  //    Used as fallback in case Zustand hasn't hydrated yet on first load
  if (typeof window !== 'undefined') {
    try {
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i)
        if (key && key.startsWith('sb-') && key.endsWith('-auth-token')) {
          const raw = localStorage.getItem(key)
          if (raw) {
            const parsed = JSON.parse(raw)
            const token = parsed?.access_token
            if (token) return token
          }
        }
      }
    } catch { /* ignore */ }
  }

  return null
}

// Generic fetch wrapper with error handling
async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<{ success: boolean; data?: T; error?: string }> {
  try {
    const token = getAuthToken()
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {}),
    }

    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    })

    // Some error responses (e.g. 401 from middleware) have an empty body
    const contentType = response.headers.get('content-type') || ''
    const hasJson = contentType.includes('application/json')
    const result = hasJson ? await response.json() : null

    if (!response.ok || (result && !result.success)) {
      if (response.status === 401) {
        return { success: false, error: 'Unauthorized — please log in again' }
      }
      return {
        success: false,
        error: result?.error || `HTTP ${response.status}: ${response.statusText}`,
      }
    }

    return { success: true, data: result?.data }
  } catch (error) {
    console.error('API Error:', error)
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Network error',
    }
  }
}

// Auth API
export const authApi = {
  register: async (data: {
    email: string
    password: string
    full_name?: string
    business_name?: string
  }): Promise<{ success: boolean; data?: { user: User; token: string }; error?: string }> => {
    return fetchApi('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  login: async (credentials: {
    email: string
    password: string
  }): Promise<{ success: boolean; data?: { user: User; token: string }; error?: string }> => {
    return fetchApi('/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    })
  },

  googleLogin: async (data: {
    token: string
  }): Promise<{ success: boolean; data?: { user: User; token: string }; error?: string }> => {
    return fetchApi('/auth/google', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },
}

// Transactions API
export const transactionsApi = {
  list: async (params?: { limit?: number; offset?: number }): Promise<{ success: boolean; data?: { transactions: Transaction[]; meta: { limit: number; offset: number; count: number } }; error?: string }> => {
    const queryParams = new URLSearchParams()
    if (params?.limit) queryParams.set('limit', params.limit.toString())
    if (params?.offset) queryParams.set('offset', params.offset.toString())
    
    return fetchApi(`/transactions?${queryParams.toString()}`)
  },

  create: async (data: {
    product_id?: string
    quantity: number
    unit?: string
    price: number
    notes?: string
  }): Promise<{ success: boolean; data?: Transaction; error?: string }> => {
    return fetchApi('/transactions', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  createVoice: async (audioBase64: string): Promise<{ success: boolean; data?: VoiceTransactionResult; error?: string }> => {
    return fetchApi('/transactions/voice', {
      method: 'POST',
      body: JSON.stringify({ audio_data: audioBase64 }),
    })
  },
}

// Confirmation API — for the pending transaction confirmation workflow
export const confirmationApi = {
  /** Fetch all pending confirmations for the current user */
  list: async (): Promise<{ success: boolean; data?: PendingConfirmation[]; error?: string }> => {
    return fetchApi('/transactions/confirmations')
  },

  /** Confirm a pending transaction — commits it to the database */
  confirm: async (confirmationId: string): Promise<{ success: boolean; data?: Transaction; error?: string }> => {
    return fetchApi(`/transactions/confirmations/${confirmationId}/confirm`, {
      method: 'POST',
    })
  },

  /** Reject a pending transaction — discards it */
  reject: async (confirmationId: string): Promise<{ success: boolean; error?: string }> => {
    return fetchApi(`/transactions/confirmations/${confirmationId}/reject`, {
      method: 'POST',
    })
  },
}

// Products API
export const productsApi = {
  list: async (search?: string): Promise<{ success: boolean; data?: { products: Product[]; meta: { count: number } }; error?: string }> => {
    const queryParams = new URLSearchParams()
    if (search) queryParams.set('search', search)
    
    return fetchApi(`/products?${queryParams.toString()}`)
  },

  create: async (data: {
    name: string
    description?: string
    sku?: string
    default_price?: number
    cost_price?: number
    unit?: string
    stock_quantity?: number
    low_stock_threshold?: number
  }): Promise<{ success: boolean; data?: Product; error?: string }> => {
    return fetchApi('/products', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  search: async (query: string): Promise<{ success: boolean; data?: Product[]; error?: string }> => {
    return fetchApi(`/products/search?q=${encodeURIComponent(query)}`)
  },
}

// Analytics API
export const analyticsApi = {
  getSummary: async (days?: number): Promise<{ success: boolean; data?: AnalyticsSummary; error?: string }> => {
    const queryParams = new URLSearchParams()
    if (days) queryParams.set('days', days.toString())
    
    return fetchApi(`/analytics/summary?${queryParams.toString()}`)
  },

  getTrends: async (days?: number): Promise<{ success: boolean; data?: { direction: string; slope: number; r_squared: number; forecast: unknown[] }; error?: string }> => {
    const queryParams = new URLSearchParams()
    if (days) queryParams.set('days', days.toString())
    
    return fetchApi(`/analytics/trends?${queryParams.toString()}`)
  },

  getForecast: async (days?: number): Promise<{ success: boolean; data?: { predicted_revenue: number; lower_bound: number; upper_bound: number; confidence: number; period: string; daily_forecast: unknown[] }; error?: string }> => {
    const queryParams = new URLSearchParams()
    if (days) queryParams.set('days', days.toString())
    
    return fetchApi(`/analytics/forecast?${queryParams.toString()}`)
  },

  getInsights: async (): Promise<{ success: boolean; data?: {
    period: string
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
  }; error?: string }> => {
    return fetchApi('/analytics/insights')
  },
}

// Voice API
export const voiceApi = {
  transcribe: async (audioBase64: string): Promise<{ success: boolean; data?: { text: string; confidence: number; language: string }; error?: string }> => {
    return fetchApi('/voice/transcribe', {
      method: 'POST',
      body: JSON.stringify({ audio_data: audioBase64 }),
    })
  },

  /**
   * Convert text to speech.
   * Returns base64-encoded WAV audio that can be decoded and played in the browser.
   */
  synthesize: async (text: string): Promise<{ success: boolean; data?: { audio: string; format: string }; error?: string }> => {
    return fetchApi('/voice/synthesize', {
      method: 'POST',
      body: JSON.stringify({ text }),
    })
  },

  /** Play a base64 WAV string in the browser using Web Audio API */
  playBase64Audio: async (base64Audio: string): Promise<void> => {
    const binary = atob(base64Audio)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    const blob = new Blob([bytes], { type: 'audio/wav' })
    const url = URL.createObjectURL(blob)
    const audio = new Audio(url)
    return new Promise((resolve) => {
      audio.onended = () => { URL.revokeObjectURL(url); resolve() }
      audio.onerror = () => { URL.revokeObjectURL(url); resolve() }
      audio.play().catch(() => resolve())
    })
  },
}

// AI Assistant chat
export const assistantApi = {
  chat: async (
    message: string,
    history: Array<{ role: string; text: string }> = [],
  ): Promise<{ success: boolean; data?: { reply: string }; error?: string }> => {
    return fetchApi('/assistant/chat', {
      method: 'POST',
      body: JSON.stringify({ message, history }),
    })
  },
}

// Alerts API
export const alertsApi = {
  list: async (unreadOnly?: boolean): Promise<{ success: boolean; data?: { alerts: unknown[]; counts: unknown }; error?: string }> => {
    const q = unreadOnly ? '?unread_only=true' : ''
    return fetchApi(`/alerts${q}`)
  },
  markRead: async (id: string): Promise<{ success: boolean; error?: string }> => {
    return fetchApi(`/alerts/${id}/read`, { method: 'POST' })
  },
  markAllRead: async (): Promise<{ success: boolean; error?: string }> => {
    return fetchApi('/alerts/read-all', { method: 'POST' })
  },
  checkNow: async (): Promise<{ success: boolean; error?: string }> => {
    return fetchApi('/alerts/check', { method: 'POST' })
  },
  counts: async (): Promise<{ success: boolean; data?: { total_unread: number; critical: number; warning: number; info: number }; error?: string }> => {
    return fetchApi('/alerts/counts')
  },
}

// Health Check
export const healthApi = {
  check: async (): Promise<{ success: boolean; data?: string }> => {
    try {
      const response = await fetch(`${API_BASE_URL.replace('/api/v1', '')}/health`)
      const text = await response.text()
      return { success: response.ok, data: text }
    } catch {
      return { success: false, data: 'unhealthy' }
    }
  },
}
