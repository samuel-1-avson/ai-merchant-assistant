export interface User {
  id: string
  email: string
  full_name?: string
  business_name?: string
  created_at: string
}

export interface Transaction {
  id: string
  user_id: string
  product_id?: string
  product_name?: string
  quantity: number
  unit: string
  price: number
  total: number
  notes?: string
  created_at: string
}

export interface Product {
  id: string
  user_id: string
  name: string
  description?: string
  sku?: string
  default_price?: number
  cost_price?: number
  unit: string
  stock_quantity: number
  low_stock_threshold: number
  is_active: boolean
  image_url?: string
  created_at: string
}

export interface AnalyticsSummary {
  total_revenue: number
  total_transactions: number
  total_items_sold: number
  average_transaction_value: number
  top_products: TopProduct[]
  daily_sales: DailySale[]
}

export interface TopProduct {
  product_id: string
  product_name: string
  total_quantity: number
  total_revenue: number
  times_sold: number
}

export interface DailySale {
  date: string
  revenue: number
  transaction_count: number
}

// ── AI voice / confirmation types ─────────────────────────────────────────

export interface PendingConfirmation {
  id: string
  user_id: string
  status: 'Pending' | 'Confirmed' | 'Rejected' | 'Expired'
  extracted_entities: {
    product?: string | null
    quantity?: number | null
    unit?: string | null
    price?: number | null
    currency?: string | null
  }
  proposed_product?: {
    id: string
    name: string
    match_score: number
  } | null
  confidence: number
  is_new_product: boolean
  created_at: string
  expires_at: string
  original_transcription: string
  display_name: string
  display_quantity: string
  display_price: string
  display_total: string
  remaining_seconds?: number
}

export interface VoiceTransactionResult {
  /** 'immediate' = transaction committed; 'pending' = awaiting user confirmation; 'awaiting_price' = price not heard */
  type: 'immediate' | 'pending' | 'awaiting_price'
  transaction?: Transaction
  transcription?: string
  extracted_entities?: PendingConfirmation['extracted_entities']
  pending_confirmation?: PendingConfirmation
  /** Present when type === 'awaiting_price' */
  transaction_id?: string
  product_name?: string
}
