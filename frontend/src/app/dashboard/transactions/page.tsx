'use client'

import { useState, useEffect, useCallback } from 'react'
import { Layout } from '@/components/layout/Layout'
import { transactionsApi } from '@/lib/api/client'
import { useToast } from '@/components/ui/Toast'
import { Transaction } from '@/types'
import { formatCurrency, formatDate } from '@/lib/utils'
import { 
  ShoppingCart, 
  Search, 
  Calendar, 
  Filter, 
  Download,
  ChevronLeft,
  ChevronRight,
  Package,
  Mic,
  Plus,
  Trash2,
  RefreshCw
} from 'lucide-react'
import Link from 'next/link'

interface TransactionFilters {
  search: string
  startDate: string
  endDate: string
}

export default function TransactionsPage() {
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [currentPage, setCurrentPage] = useState(0)
  const [totalCount, setTotalCount] = useState(0)
  const [filters, setFilters] = useState<TransactionFilters>({
    search: '',
    startDate: '',
    endDate: ''
  })
  const [showFilters, setShowFilters] = useState(false)
  const { success, error } = useToast()
  const limit = 20

  const fetchTransactions = useCallback(async () => {
    setIsLoading(true)
    try {
      const result = await transactionsApi.list({
        limit,
        offset: currentPage * limit
      })

      if (result.success && result.data?.transactions) {
        setTransactions(result.data.transactions)
        setTotalCount(result.data.meta?.count || result.data.transactions.length)
      } else {
        console.error('Failed to fetch transactions:', result.error)
      }
    } catch (err) {
      console.error('An error occurred while fetching transactions', err)
    } finally {
      setIsLoading(false)
    }
  }, [currentPage])

  useEffect(() => {
    fetchTransactions()
  }, [fetchTransactions])

  // Filter transactions client-side for now
  const filteredTransactions = (transactions || []).filter(t => {
    const matchesSearch = !filters.search || 
      t.product_name?.toLowerCase().includes(filters.search.toLowerCase()) ||
      t.notes?.toLowerCase().includes(filters.search.toLowerCase())
    
    const matchesStartDate = !filters.startDate || 
      new Date(t.created_at) >= new Date(filters.startDate)
    
    const matchesEndDate = !filters.endDate || 
      new Date(t.created_at) <= new Date(filters.endDate + 'T23:59:59')
    
    return matchesSearch && matchesStartDate && matchesEndDate
  })

  const totalPages = Math.ceil(totalCount / limit)

  const exportToCSV = () => {
    const headers = ['Date', 'Product', 'Quantity', 'Unit Price', 'Total', 'Notes']
    const rows = filteredTransactions.map(t => [
      new Date(t.created_at).toLocaleDateString(),
      t.product_name || 'Unknown Product',
      t.quantity,
      t.price,
      t.total,
      t.notes || ''
    ])
    
    const csv = [headers, ...rows].map(row => row.join(',')).join('\n')
    const blob = new Blob([csv], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `transactions_${new Date().toISOString().split('T')[0]}.csv`
    a.click()
    success('Transactions exported to CSV')
  }

  const clearFilters = () => {
    setFilters({ search: '', startDate: '', endDate: '' })
  }

  return (
    <Layout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-900">Transactions</h1>
            <p className="text-slate-500 mt-1">
              Manage and view all your sales transactions
            </p>
          </div>
          
          <div className="flex items-center gap-3">
            <button 
              onClick={() => setShowFilters(!showFilters)}
              className={`btn-ghost text-sm ${showFilters ? 'bg-primary-50 text-primary-700' : ''}`}
            >
              <Filter className="w-4 h-4" />
              Filters
            </button>
            <button 
              onClick={exportToCSV}
              className="btn-ghost text-sm"
            >
              <Download className="w-4 h-4" />
              Export
            </button>
            <Link 
              href="/dashboard"
              className="btn-primary text-sm"
            >
              <Mic className="w-4 h-4" />
              New Sale
            </Link>
          </div>
        </div>

        {/* Filters */}
        {showFilters && (
          <div className="bg-white p-4 rounded-xl border border-slate-200 space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">Search</label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                  <input
                    type="text"
                    value={filters.search}
                    onChange={(e) => setFilters({ ...filters, search: e.target.value })}
                    placeholder="Search products or notes..."
                    className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">Start Date</label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                  <input
                    type="date"
                    value={filters.startDate}
                    onChange={(e) => setFilters({ ...filters, startDate: e.target.value })}
                    className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">End Date</label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                  <input
                    type="date"
                    value={filters.endDate}
                    onChange={(e) => setFilters({ ...filters, endDate: e.target.value })}
                    className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  />
                </div>
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <button 
                onClick={clearFilters}
                className="btn-ghost text-sm"
              >
                Clear Filters
              </button>
            </div>
          </div>
        )}

        {/* Stats Summary */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <p className="text-sm text-slate-500">Total Transactions</p>
            <p className="text-2xl font-bold text-slate-900 mt-1">{totalCount}</p>
          </div>
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <p className="text-sm text-slate-500">Showing</p>
            <p className="text-2xl font-bold text-slate-900 mt-1">{filteredTransactions.length}</p>
          </div>
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <p className="text-sm text-slate-500">Total Value</p>
            <p className="text-2xl font-bold text-slate-900 mt-1">
              {formatCurrency(
                filteredTransactions.reduce((sum, t) => sum + (t.total || 0), 0),
                'USD'
              ).replace('US', '')}
            </p>
          </div>
        </div>

        {/* Transactions Table */}
        <div className="bg-white rounded-xl border border-slate-200 overflow-hidden">
          {isLoading ? (
            <div className="p-8 text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto" />
              <p className="mt-4 text-slate-500">Loading transactions...</p>
            </div>
          ) : filteredTransactions.length === 0 ? (
            <div className="p-12 text-center">
              <ShoppingCart className="w-12 h-12 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-slate-900 mb-2">No transactions found</h3>
              <p className="text-slate-500 mb-4">
                {transactions.length === 0 
                  ? "You haven't recorded any transactions yet." 
                  : "No transactions match your filters."}
              </p>
              {transactions.length === 0 ? (
                <Link 
                  href="/dashboard"
                  className="btn-primary"
                >
                  <Mic className="w-4 h-4" />
                  Record Your First Sale
                </Link>
              ) : (
                <button 
                  onClick={clearFilters}
                  className="btn-outline"
                >
                  Clear Filters
                </button>
              )}
            </div>
          ) : (
            <>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-slate-50 border-b border-slate-200">
                    <tr>
                      <th className="text-left py-3 px-4 text-sm font-semibold text-slate-700">Date</th>
                      <th className="text-left py-3 px-4 text-sm font-semibold text-slate-700">Product</th>
                      <th className="text-right py-3 px-4 text-sm font-semibold text-slate-700">Qty</th>
                      <th className="text-right py-3 px-4 text-sm font-semibold text-slate-700">Unit Price</th>
                      <th className="text-right py-3 px-4 text-sm font-semibold text-slate-700">Total</th>
                      <th className="text-left py-3 px-4 text-sm font-semibold text-slate-700">Notes</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {filteredTransactions.map((transaction) => (
                      <tr 
                        key={transaction.id}
                        className="hover:bg-slate-50 transition-colors"
                      >
                        <td className="py-3 px-4 text-sm text-slate-600">
                          {formatDate(transaction.created_at)}
                        </td>
                        <td className="py-3 px-4">
                          <div className="flex items-center gap-2">
                            <div className="w-8 h-8 bg-primary-100 rounded-lg flex items-center justify-center">
                              <Package className="w-4 h-4 text-primary-600" />
                            </div>
                            <span className="font-medium text-slate-900">
                              {transaction.product_name || 'Unknown Product'}
                            </span>
                          </div>
                        </td>
                        <td className="py-3 px-4 text-sm text-slate-600 text-right">
                          {transaction.quantity} {transaction.unit}
                        </td>
                        <td className="py-3 px-4 text-sm text-slate-600 text-right">
                          {formatCurrency(transaction.price || 0, 'USD').replace('US', '')}
                        </td>
                        <td className="py-3 px-4 text-sm font-semibold text-slate-900 text-right">
                          {formatCurrency(transaction.total || 0, 'USD').replace('US', '')}
                        </td>
                        <td className="py-3 px-4 text-sm text-slate-500 max-w-xs truncate">
                          {transaction.notes || '-'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Pagination */}
              {totalPages > 1 && (
                <div className="flex items-center justify-between px-4 py-3 border-t border-slate-200">
                  <p className="text-sm text-slate-500">
                    Page {currentPage + 1} of {totalPages}
                  </p>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => setCurrentPage(p => Math.max(0, p - 1))}
                      disabled={currentPage === 0}
                      className="p-2 rounded-lg hover:bg-slate-100 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <ChevronLeft className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => setCurrentPage(p => Math.min(totalPages - 1, p + 1))}
                      disabled={currentPage >= totalPages - 1}
                      className="p-2 rounded-lg hover:bg-slate-100 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <ChevronRight className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </Layout>
  )
}
