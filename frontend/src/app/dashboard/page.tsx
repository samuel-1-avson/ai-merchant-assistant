'use client'

import { useState, useEffect, useCallback } from 'react'
import { VoiceRecorder } from '@/components/voice/VoiceRecorder'
import { RecentTransactions } from '@/components/dashboard/RecentTransactions'
import { Layout } from '@/components/layout/Layout'
import { NotificationBell } from '@/components/realtime/NotificationBell'
import { ToastContainer, useToast } from '@/components/ui/Toast'
import { useDashboardStore } from '@/stores/dashboardStore'
import { useTransactionUpdates, useWebSocket } from '@/hooks/useWebSocket'
import { formatCurrency } from '@/lib/utils'
import {
  TrendingUp,
  Calendar,
  Sparkles,
  ArrowUpRight,
  ArrowDownRight,
  Package,
  Wifi,
  WifiOff,
  RefreshCw,
  Activity,
  AlertTriangle,
  TrendingDown,
} from 'lucide-react'

interface StatData {
  title: string
  value: string
  change: string
  changeType: 'positive' | 'negative'
  icon: React.ElementType
  color: string
}

export default function Dashboard() {
  const [currentDate, setCurrentDate] = useState('')
  const { toasts, removeToast, success, error, info } = useToast()
  
  // Get data and actions from store
  const {
    analytics,
    insights,
    analyticsLoading,
    insightsLoading,
    fetchAnalytics,
    fetchInsights,
    fetchTransactions,
  } = useDashboardStore()

  // WebSocket connection for real-time updates
  const { isConnected: wsConnected, subscribe } = useWebSocket({
    autoConnect: true,
    onConnect: () => {
      console.log('[Dashboard] WebSocket connected')
    },
    onDisconnect: () => {
      console.log('[Dashboard] WebSocket disconnected')
    },
  })

  // Handle real-time transaction updates
  const handleTransactionUpdate = useCallback((message: any) => {
    console.log('[Dashboard] Transaction update received:', message)
    
    // Refresh transactions list
    fetchTransactions({ limit: 10, offset: 0 })
    
    // Refresh analytics
    fetchAnalytics(7)
    
    // Show notification
    info('New transaction recorded!')
  }, [fetchTransactions, fetchAnalytics, info])

  // Subscribe to transaction updates
  useEffect(() => {
    if (wsConnected) {
      const unsubscribe = subscribe('transaction_update', handleTransactionUpdate)
      return unsubscribe
    }
  }, [wsConnected, subscribe, handleTransactionUpdate])

  useEffect(() => {
    // Set current date
    setCurrentDate(new Date().toLocaleDateString('en-US', { 
      weekday: 'long', 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric' 
    }))

    // Fetch data on mount
    fetchAnalytics(7)
    fetchInsights()
    fetchTransactions({ limit: 10, offset: 0 })
  }, [fetchAnalytics, fetchInsights, fetchTransactions])

  // Prepare stats from real analytics + insights data
  const revChange = insights?.revenue_change_percent ?? 0
  const stats: StatData[] = analytics ? [
    {
      title: 'Total Revenue',
      value: `$${Number(analytics.total_revenue || 0).toFixed(2)}`,
      change: `${revChange >= 0 ? '+' : ''}${revChange.toFixed(1)}% vs last week`,
      changeType: revChange >= 0 ? 'positive' : 'negative',
      icon: TrendingUp,
      color: 'primary',
    },
    {
      title: 'Total Orders',
      value: Number(analytics.total_transactions || 0).toLocaleString(),
      change: insights ? `${(insights.transactions_per_day).toFixed(1)}/day` : '—',
      changeType: 'positive',
      icon: Package,
      color: 'secondary',
    },
    {
      title: 'Items Sold',
      value: Math.round(Number(analytics.total_items_sold || 0)).toLocaleString(),
      change: insights?.average_profit_margin_pct != null
        ? `${insights.average_profit_margin_pct.toFixed(0)}% avg margin`
        : 'No cost data',
      changeType: (insights?.average_profit_margin_pct ?? 0) > 0 ? 'positive' : 'negative',
      icon: Package,
      color: 'accent',
    },
    {
      title: 'Avg Transaction',
      value: `$${Number(analytics.average_transaction_value || 0).toFixed(2)}`,
      change: insights ? `$${insights.average_daily_revenue.toFixed(2)}/day` : '—',
      changeType: Number(analytics.average_transaction_value || 0) >= 20 ? 'positive' : 'negative',
      icon: TrendingUp,
      color: 'primary',
    },
  ] : []

  const isLoading = analyticsLoading || insightsLoading

  if (isLoading) {
    return (
      <Layout>
        <div className="space-y-6">
          {/* Skeleton Header */}
          <div className="flex justify-between items-center">
            <div className="h-8 w-48 bg-slate-200 rounded-lg animate-pulse" />
            <div className="h-6 w-32 bg-slate-200 rounded-lg animate-pulse" />
          </div>
          
          {/* Skeleton Stats */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="h-32 bg-slate-200 rounded-2xl animate-pulse" />
            ))}
          </div>
          
          {/* Skeleton Content */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="h-96 bg-slate-200 rounded-2xl animate-pulse" />
            <div className="h-96 bg-slate-200 rounded-2xl animate-pulse" />
          </div>
        </div>
      </Layout>
    )
  }

  return (
    <Layout>
      <div className="space-y-8">
        {/* Header Section */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl sm:text-3xl font-bold text-slate-900">
              Dashboard
            </h1>
            <p className="text-slate-500 mt-1 flex items-center gap-2">
              <Calendar className="w-4 h-4" />
              {currentDate}
            </p>
          </div>
          
          <div className="flex items-center gap-3">
            {/* Connection Status */}
            <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-sm ${
              wsConnected 
                ? 'bg-green-50 text-green-700 border border-green-200' 
                : 'bg-gray-50 text-gray-600 border border-gray-200'
            }`}>
              {wsConnected ? (
                <>
                  <Wifi className="w-4 h-4" />
                  <span className="hidden sm:inline">Live</span>
                </>
              ) : (
                <>
                  <WifiOff className="w-4 h-4" />
                  <span className="hidden sm:inline">Offline</span>
                </>
              )}
            </div>

            {/* Notification Bell */}
            <NotificationBell />

            <button 
              className="btn-ghost text-sm"
              onClick={() => {
                fetchAnalytics()
                fetchInsights()
                fetchTransactions()
                success('Dashboard refreshed')
              }}
            >
              <RefreshCw className="w-4 h-4" />
              <span className="hidden sm:inline">Refresh</span>
            </button>
            <button className="btn-primary text-sm">
              <ArrowUpRight className="w-4 h-4" />
              <span className="hidden sm:inline">Export</span>
            </button>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {stats.map((stat, index) => (
            <div 
              key={index}
              className="card-hover group"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div className="flex items-start justify-between">
                <div className={`w-12 h-12 rounded-xl bg-${stat.color}-100 flex items-center justify-center group-hover:scale-110 transition-transform`}>
                  <stat.icon className={`w-6 h-6 text-${stat.color}-600`} />
                </div>
                <div className={`flex items-center gap-1 text-sm font-medium ${
                  stat.changeType === 'positive' ? 'text-secondary-600' : 'text-red-600'
                }`}>
                  {stat.changeType === 'positive' ? (
                    <ArrowUpRight className="w-4 h-4" />
                  ) : (
                    <ArrowDownRight className="w-4 h-4" />
                  )}
                  {stat.change}
                </div>
              </div>
              <div className="mt-4">
                <p className="text-2xl font-bold text-slate-900">{stat.value}</p>
                <p className="text-sm text-slate-500 mt-1">{stat.title}</p>
              </div>
            </div>
          ))}
        </div>

        {/* Business Health + AI Insight Row */}
        {insights && (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Health Score Card */}
            <div className="card border border-slate-100 flex items-center gap-4">
              <div className={`w-16 h-16 rounded-2xl flex items-center justify-center flex-shrink-0 ${
                insights.health_score >= 80 ? 'bg-green-100' :
                insights.health_score >= 60 ? 'bg-blue-100' :
                insights.health_score >= 40 ? 'bg-yellow-100' : 'bg-red-100'
              }`}>
                <Activity className={`w-8 h-8 ${
                  insights.health_score >= 80 ? 'text-green-600' :
                  insights.health_score >= 60 ? 'text-blue-600' :
                  insights.health_score >= 40 ? 'text-yellow-600' : 'text-red-600'
                }`} />
              </div>
              <div>
                <p className="text-sm text-slate-500">Business Health</p>
                <p className="text-3xl font-bold text-slate-900">{insights.health_score}<span className="text-lg text-slate-400">/100</span></p>
                <span className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${
                  insights.health_score >= 80 ? 'bg-green-100 text-green-700' :
                  insights.health_score >= 60 ? 'bg-blue-100 text-blue-700' :
                  insights.health_score >= 40 ? 'bg-yellow-100 text-yellow-700' : 'bg-red-100 text-red-700'
                }`}>{insights.health_label.replace('_', ' ')}</span>
              </div>
            </div>

            {/* AI Insight Banner */}
            <div className="lg:col-span-2 bg-gradient-to-r from-accent-50 to-primary-50 border border-accent-200 rounded-2xl p-6 flex items-start gap-4">
              <div className="w-10 h-10 bg-accent-100 rounded-xl flex items-center justify-center flex-shrink-0">
                <Sparkles className="w-5 h-5 text-accent-600" />
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="font-semibold text-slate-900 mb-1">AI Insight</h3>
                <p className="text-slate-600 text-sm">{insights.summary}</p>
                {insights.recommendations.length > 0 && (
                  <ul className="mt-2 space-y-1">
                    {insights.recommendations.slice(0, 2).map((rec, i) => (
                      <li key={i} className="text-sm text-slate-500 flex items-start gap-2">
                        <span className={`mt-0.5 flex-shrink-0 ${rec.priority === 'high' ? 'text-red-500' : rec.priority === 'medium' ? 'text-yellow-500' : 'text-accent-500'}`}>•</span>
                        {rec.message}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Product Performance Row */}
        {insights && (insights.top_sellers.length > 0 || insights.slow_movers.length > 0) && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Top Sellers */}
            {insights.top_sellers.length > 0 && (
              <div className="card border border-slate-100">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-8 h-8 bg-green-100 rounded-lg flex items-center justify-center">
                    <TrendingUp className="w-4 h-4 text-green-600" />
                  </div>
                  <h3 className="font-semibold text-slate-900">Top Sellers (30 days)</h3>
                </div>
                <div className="space-y-3">
                  {insights.top_sellers.slice(0, 4).map((p, i) => (
                    <div key={i} className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-bold text-slate-400 w-4">{i + 1}</span>
                        <span className="text-sm text-slate-700 font-medium">{p.product_name}</span>
                      </div>
                      <div className="text-right">
                        <span className="text-sm font-semibold text-slate-900">${Number(p.total_revenue).toFixed(2)}</span>
                        <span className="text-xs text-slate-400 ml-2">{p.times_sold} sales</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Slow Movers / Alerts */}
            {insights.slow_movers.length > 0 && (
              <div className="card border border-slate-100">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-8 h-8 bg-yellow-100 rounded-lg flex items-center justify-center">
                    <TrendingDown className="w-4 h-4 text-yellow-600" />
                  </div>
                  <h3 className="font-semibold text-slate-900">Slow Movers (30 days)</h3>
                </div>
                <div className="space-y-3">
                  {insights.slow_movers.slice(0, 4).map((p, i) => (
                    <div key={i} className="flex items-center justify-between">
                      <span className="text-sm text-slate-700">{p.product_name}</span>
                      <span className="text-xs text-yellow-600 bg-yellow-50 px-2 py-0.5 rounded-full">{p.times_sold} sales</span>
                    </div>
                  ))}
                </div>
                {insights.no_sales_products.length > 0 && (
                  <div className="mt-3 pt-3 border-t border-slate-100 flex items-center gap-2 text-xs text-red-600">
                    <AlertTriangle className="w-3.5 h-3.5" />
                    {insights.no_sales_products.length} product{insights.no_sales_products.length > 1 ? 's' : ''} with zero sales
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Voice Recorder Section */}
          <div className="lg:col-span-1" data-section="quick-sale">
            <div className="card border border-slate-100 sticky top-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-primary-100 rounded-xl flex items-center justify-center">
                  <TrendingUp className="w-5 h-5 text-primary-600" />
                </div>
                <div>
                  <h2 className="text-lg font-bold text-slate-900">Quick Sale</h2>
                  <p className="text-sm text-slate-500">Record a new transaction</p>
                </div>
              </div>
              <div data-recorder="voice">
                <VoiceRecorder onSuccess={(msg) => success(msg)} onError={(msg) => error(msg)} />
              </div>
              
              {/* Quick Tips */}
              <div className="mt-6 pt-6 border-t border-slate-100">
                <p className="text-sm font-medium text-slate-700 mb-3">Try saying:</p>
                <div className="space-y-2">
                  {[
                    'Sold 3 shirts for $45 each',
                    'Customer bought 2 caps, total $50',
                    'New order: 5 hoodies at $80',
                  ].map((tip, i) => (
                    <p key={i} className="text-sm text-slate-500 bg-slate-50 rounded-lg px-3 py-2">
                      &quot;{tip}&quot;
                    </p>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Recent Transactions */}
          <div className="lg:col-span-2">
            <div className="card border border-slate-100">
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-secondary-100 rounded-xl flex items-center justify-center">
                    <Package className="w-5 h-5 text-secondary-600" />
                  </div>
                  <div>
                    <h2 className="text-lg font-bold text-slate-900">Recent Transactions</h2>
                    <p className="text-sm text-slate-500 flex items-center gap-2">
                      Last 10 recorded sales
                      {wsConnected && (
                        <span className="flex items-center gap-1 text-green-600">
                          <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                          Live updates
                        </span>
                      )}
                    </p>
                  </div>
                </div>
                <button 
                  className="btn-ghost text-sm"
                  onClick={() => fetchTransactions()}
                >
                  Refresh
                </button>
              </div>
              <RecentTransactions />
            </div>
          </div>
        </div>
      </div>

      {/* Toast Notifications */}
      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </Layout>
  )
}
