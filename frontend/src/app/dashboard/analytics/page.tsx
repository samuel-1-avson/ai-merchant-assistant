'use client'

import { useState, useEffect } from 'react'
import { Layout } from '@/components/layout/Layout'
import { TrendingUp, TrendingDown, DollarSign, ShoppingCart, AlertCircle } from 'lucide-react'
import { analyticsApi } from '@/lib/api/client'

interface AnalyticsData {
  total_revenue: number
  total_transactions: number
  total_items_sold: number
  average_transaction_value: number
  daily_sales: DailySale[]
  top_products: TopProduct[]
}

interface DailySale {
  date: string
  revenue: number
  transaction_count: number
}

interface TopProduct {
  product_id: string
  product_name: string
  total_quantity: number
  total_revenue: number
  times_sold: number
}

interface TrendData {
  direction: string
  slope: number
  r_squared: number
  forecast: { date: string; value: number }[]
}

export default function AnalyticsPage() {
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null)
  const [trends, setTrends] = useState<TrendData | null>(null)
  const [insights, setInsights] = useState<any>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [period, setPeriod] = useState('7')

  useEffect(() => {
    fetchAnalytics()
  }, [period])

  const fetchAnalytics = async () => {
    setIsLoading(true)
    try {
      const days = Number(period)
      const [summaryResult, trendsResult, insightsResult] = await Promise.all([
        analyticsApi.getSummary(days),
        analyticsApi.getTrends(days),
        analyticsApi.getInsights(),
      ])
      if (summaryResult.success && summaryResult.data) setAnalytics(summaryResult.data as any)
      if (trendsResult.success && trendsResult.data) setTrends(trendsResult.data as any)
      if (insightsResult.success && insightsResult.data) setInsights(insightsResult.data)
    } catch (error) {
      console.error('Error fetching analytics:', error)
    } finally {
      setIsLoading(false)
    }
  }

  if (isLoading) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-screen">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        </div>
      </Layout>
    )
  }

  return (
    <Layout>
      <div className="space-y-6">
        <div className="flex justify-between items-center">
          <h1 className="text-3xl font-bold">Analytics</h1>
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
            className="px-4 py-2 border rounded-lg"
          >
            <option value="7">Last 7 Days</option>
            <option value="30">Last 30 Days</option>
            <option value="90">Last 90 Days</option>
          </select>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <StatCard
            title="Total Revenue"
            value={`$${Number(analytics?.total_revenue || 0).toLocaleString()}`}
            trend={trends?.direction === 'increasing' ? 'up' : 'down'}
            icon={DollarSign}
          />
          <StatCard
            title="Transactions"
            value={Number(analytics?.total_transactions || 0).toLocaleString()}
            trend="up"
            icon={ShoppingCart}
          />
          <StatCard
            title="Items Sold"
            value={Number(analytics?.total_items_sold || 0).toLocaleString()}
            trend="up"
            icon={TrendingUp}
          />
          <StatCard
            title="Avg Transaction"
            value={`$${Number(analytics?.average_transaction_value || 0).toFixed(2)}`}
            trend="stable"
            icon={DollarSign}
          />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Sales Chart */}
          <div className="bg-white p-6 rounded-xl shadow-sm border">
            <h2 className="text-xl font-semibold mb-4">Sales Trend</h2>
            <div className="h-64 flex items-end space-x-2">
              {analytics?.daily_sales.map((day, index) => (
                <div key={index} className="flex-1 flex flex-col items-center">
                  <div
                    className="w-full bg-primary-500 rounded-t"
                    style={{
                      height: `${(Number(day.revenue || 0) / Math.max(...analytics.daily_sales.map(d => Number(d.revenue || 0)))) * 200}px`
                    }}
                  />
                  <span className="text-xs text-gray-500 mt-2">
                    {new Date(day.date).toLocaleDateString('en', { weekday: 'short' })}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Forecast */}
          <div className="bg-white p-6 rounded-xl shadow-sm border">
            <h2 className="text-xl font-semibold mb-4">7-Day Forecast</h2>
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <TrendingUp className="w-5 h-5 text-green-500" />
                <span className="text-sm text-gray-600">
                  Trend: {trends?.direction} (R² = {Number(trends?.r_squared || 0).toFixed(2)})
                </span>
              </div>
              <div className="h-48 flex items-end space-x-2">
                {trends?.forecast.map((point, index) => (
                  <div key={index} className="flex-1 flex flex-col items-center">
                    <div
                      className="w-full bg-green-400 rounded-t opacity-70"
                      style={{
                        height: `${Math.min((Number(point.value || 0) / 3000) * 150, 150)}px`
                      }}
                    />
                    <span className="text-xs text-gray-500 mt-2">
                      {new Date(point.date).toLocaleDateString('en', { month: 'short', day: 'numeric' })}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Top Products */}
        <div className="bg-white p-6 rounded-xl shadow-sm border">
          <h2 className="text-xl font-semibold mb-4">Top Products</h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4">Product</th>
                  <th className="text-right py-3 px-4">Quantity Sold</th>
                  <th className="text-right py-3 px-4">Revenue</th>
                  <th className="text-right py-3 px-4">Times Sold</th>
                </tr>
              </thead>
              <tbody>
                {analytics?.top_products.map((product) => (
                  <tr key={product.product_id} className="border-b">
                    <td className="py-3 px-4 font-medium">{product.product_name}</td>
                    <td className="text-right py-3 px-4">{Number(product.total_quantity || 0).toLocaleString()}</td>
                    <td className="text-right py-3 px-4">${Number(product.total_revenue || 0).toLocaleString()}</td>
                    <td className="text-right py-3 px-4">{Number(product.times_sold || 0).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* AI Insights */}
        {insights && (
          <div className="bg-gradient-to-r from-primary-50 to-blue-50 p-6 rounded-xl border border-primary-200">
            <div className="flex items-center gap-2 mb-4">
              <AlertCircle className="w-5 h-5 text-primary-600" />
              <h2 className="text-xl font-semibold">AI Insights</h2>
            </div>
            <p className="text-gray-700 mb-4">{insights.summary}</p>
            <div className="space-y-2">
              <h3 className="font-medium">Recommendations:</h3>
              <ul className="list-disc list-inside space-y-1 text-gray-600">
                {insights.recommendations.map((rec: string, index: number) => (
                  <li key={index}>{rec}</li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </div>
    </Layout>
  )
}

function StatCard({ title, value, trend, icon: Icon }: { title: string; value: string; trend: string; icon: any }) {
  const TrendIcon = trend === 'up' ? TrendingUp : trend === 'down' ? TrendingDown : TrendingUp
  const trendColor = trend === 'up' ? 'text-green-600' : trend === 'down' ? 'text-red-600' : 'text-gray-600'

  return (
    <div className="bg-white p-6 rounded-xl shadow-sm border">
      <div className="flex items-center justify-between">
        <div className="p-2 bg-primary-50 rounded-lg">
          <Icon className="w-5 h-5 text-primary-600" />
        </div>
        <div className={`flex items-center gap-1 text-sm ${trendColor}`}>
          <TrendIcon className="w-4 h-4" />
          {trend === 'up' ? '+12%' : trend === 'down' ? '-5%' : '0%'}
        </div>
      </div>
      <div className="mt-4">
        <p className="text-sm text-gray-500">{title}</p>
        <p className="text-2xl font-bold">{value}</p>
      </div>
    </div>
  )
}
