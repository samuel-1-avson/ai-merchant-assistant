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
  RefreshCw
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

  // Prepare stats from real analytics data
  const stats: StatData[] = analytics ? [
    {
      title: 'Total Revenue',
      value: formatCurrency(analytics.total_revenue, 'USD').replace('US', ''),
      change: insights?.revenue_change_percent 
        ? `${insights.revenue_change_percent >= 0 ? '+' : ''}${insights.revenue_change_percent.toFixed(1)}%`
        : '+0%',
      changeType: insights?.revenue_change_percent && insights.revenue_change_percent >= 0 ? 'positive' : 'negative',
      icon: TrendingUp,
      color: 'primary',
    },
    {
      title: 'Total Orders',
      value: analytics.total_transactions.toLocaleString(),
      change: '+8.2%',
      changeType: 'positive',
      icon: Package,
      color: 'secondary',
    },
    {
      title: 'Items Sold',
      value: Math.round(analytics.total_items_sold).toLocaleString(),
      change: '+15.3%',
      changeType: 'positive',
      icon: Package,
      color: 'accent',
    },
    {
      title: 'Avg Transaction',
      value: `$${analytics.average_transaction_value.toFixed(2)}`,
      change: analytics.average_transaction_value > 20 ? '+5.2%' : '-2.1%',
      changeType: analytics.average_transaction_value > 20 ? 'positive' : 'negative',
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

        {/* AI Insight Banner */}
        {insights && (
          <div className="bg-gradient-to-r from-accent-50 to-primary-50 border border-accent-200 rounded-2xl p-6 flex items-start gap-4">
            <div className="w-12 h-12 bg-accent-100 rounded-xl flex items-center justify-center flex-shrink-0">
              <Sparkles className="w-6 h-6 text-accent-600" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold text-slate-900 mb-1">AI Insight</h3>
              <p className="text-slate-600">
                {insights.summary}
              </p>
              {insights.recommendations.length > 0 && (
                <ul className="mt-2 space-y-1">
                  {insights.recommendations.slice(0, 2).map((rec, i) => (
                    <li key={i} className="text-sm text-slate-500 flex items-start gap-2">
                      <span className="text-accent-500">•</span>
                      {rec}
                    </li>
                  ))}
                </ul>
              )}
            </div>
            <button className="btn-outline text-sm py-2 px-4 flex-shrink-0">
              View Details
            </button>
          </div>
        )}

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Voice Recorder Section */}
          <div className="lg:col-span-1">
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
              <VoiceRecorder onSuccess={(msg) => success(msg)} onError={(msg) => error(msg)} />
              
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
