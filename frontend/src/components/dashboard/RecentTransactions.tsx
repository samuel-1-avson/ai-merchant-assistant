'use client'

import { motion, AnimatePresence } from 'framer-motion'
import { Package, ShoppingBag, Shirt, Coffee, MoreHorizontal, TrendingUp, Clock } from 'lucide-react'
import { useDashboardStore } from '@/stores/dashboardStore'
import { Transaction } from '@/types'

const getIconForCategory = (category?: string) => {
  switch (category) {
    case 'clothing':
      return Shirt
    case 'accessories':
      return ShoppingBag
    case 'food':
      return Coffee
    default:
      return Package
  }
}

/** Resolve a display name for a transaction, falling back to notes extraction. */
function getProductName(transaction: Transaction): string {
  if (transaction.product_name) return transaction.product_name
  if (transaction.notes) {
    for (const prefix of [
      'New product from voice: ',
      'Voice transaction: ',
      'Multi-item transaction: ',
    ]) {
      if (transaction.notes.startsWith(prefix)) {
        return transaction.notes.slice(prefix.length)
      }
    }
  }
  return 'Unknown Product'
}

const formatTimeAgo = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000)
  
  if (diffInSeconds < 60) return 'Just now'
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`
  return `${Math.floor(diffInSeconds / 86400)}d ago`
}

export function RecentTransactions() {
  const { transactions, transactionsLoading, fetchTransactions } = useDashboardStore()

  if (transactionsLoading) {
    return (
      <div className="space-y-4">
        {[...Array(4)].map((_, i) => (
          <div key={i} className="flex items-center gap-4 p-4">
            <div className="w-12 h-12 bg-slate-100 rounded-xl animate-pulse" />
            <div className="flex-1 space-y-2">
              <div className="h-4 bg-slate-100 rounded w-1/3 animate-pulse" />
              <div className="h-3 bg-slate-100 rounded w-1/4 animate-pulse" />
            </div>
            <div className="h-5 bg-slate-100 rounded w-16 animate-pulse" />
          </div>
        ))}
      </div>
    )
  }

  if (!transactions || transactions.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="w-16 h-16 bg-slate-100 rounded-2xl flex items-center justify-center mx-auto mb-4">
          <TrendingUp className="w-8 h-8 text-slate-400" />
        </div>
        <p className="text-slate-900 font-semibold mb-1">No transactions yet</p>
        <p className="text-sm text-slate-500 mb-4">Record your first sale using the voice recorder!</p>
        <button 
          onClick={() => fetchTransactions()}
          className="text-sm text-primary-600 hover:text-primary-700 font-medium"
        >
          Refresh
        </button>
      </div>
    )
  }

  return (
    <div className="space-y-1">
      <AnimatePresence>
        {transactions.map((transaction, index) => {
          const Icon = getIconForCategory(transaction.product_name)
          
          return (
            <motion.div
              key={transaction.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ delay: index * 0.05 }}
              className="group flex items-center gap-4 p-4 rounded-2xl hover:bg-slate-50 transition-colors cursor-pointer"
            >
              {/* Icon */}
              <div className="w-12 h-12 bg-primary-50 group-hover:bg-primary-100 rounded-xl flex items-center justify-center transition-colors">
                <Icon className="w-5 h-5 text-primary-600" />
              </div>

              {/* Details */}
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-slate-900 truncate">
                  {getProductName(transaction)}
                </p>
                <div className="flex items-center gap-2 text-sm text-slate-500">
                  <span>{transaction.quantity} {transaction.unit}</span>
                  <span>×</span>
                  <span>${Number(transaction.price || 0).toFixed(2)}</span>
                </div>
              </div>

              {/* Time & Total */}
              <div className="text-right">
                <p className="font-bold text-slate-900">
                  ${Number(transaction.total || 0).toFixed(2)}
                </p>
                <p className="flex items-center gap-1 text-xs text-slate-400">
                  <Clock className="w-3 h-3" />
                  {formatTimeAgo(transaction.created_at)}
                </p>
              </div>

              {/* Actions */}
              <button className="opacity-0 group-hover:opacity-100 p-2 hover:bg-slate-200 rounded-lg transition-all">
                <MoreHorizontal className="w-4 h-4 text-slate-400" />
              </button>
            </motion.div>
          )
        })}
      </AnimatePresence>

      {/* View All Link */}
      <div className="pt-4 border-t border-slate-100">
        <button 
          onClick={() => fetchTransactions()}
          className="w-full py-3 text-sm font-medium text-primary-600 hover:text-primary-700 hover:bg-primary-50 rounded-xl transition-colors"
        >
          Refresh Transactions
        </button>
      </div>
    </div>
  )
}
