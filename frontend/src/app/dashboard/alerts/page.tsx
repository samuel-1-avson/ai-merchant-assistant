'use client'

import { useState, useEffect } from 'react'
import { Layout } from '@/components/layout/Layout'
import { Bell, Check, AlertTriangle, Info, AlertCircle, RefreshCw } from 'lucide-react'

interface Alert {
  id: string
  alert_type: string
  severity: string
  title: string
  message: string
  metadata: any
  is_read: boolean
  created_at: string
}

export default function AlertsPage() {
  const [alerts, setAlerts] = useState<Alert[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [filter, setFilter] = useState('all')

  useEffect(() => {
    fetchAlerts()
  }, [])

  const fetchAlerts = async () => {
    setIsLoading(true)
    try {
      const res = await fetch('http://localhost:8888/api/v1/alerts')
      const data = await res.json()
      if (data.success) {
        setAlerts(data.data)
      }
    } catch (error) {
      console.error('Error fetching alerts:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const markAsRead = async (id: string) => {
    try {
      await fetch(`http://localhost:8888/api/v1/alerts/${id}/read`, { method: 'POST' })
      setAlerts(alerts.map(a => a.id === id ? { ...a, is_read: true } : a))
    } catch (error) {
      console.error('Error marking alert as read:', error)
    }
  }

  const checkNow = async () => {
    setIsLoading(true)
    try {
      await fetch('http://localhost:8888/api/v1/alerts/check', { method: 'POST' })
      await fetchAlerts()
    } catch (error) {
      console.error('Error checking alerts:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const filteredAlerts = alerts.filter(alert => {
    if (filter === 'all') return true
    if (filter === 'unread') return !alert.is_read
    if (filter === 'critical') return alert.severity === 'Critical'
    return true
  })

  const unreadCount = alerts.filter(a => !a.is_read).length

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
          <div className="flex items-center gap-4">
            <h1 className="text-3xl font-bold">Alerts</h1>
            {unreadCount > 0 && (
              <span className="bg-red-500 text-white px-3 py-1 rounded-full text-sm font-medium">
                {unreadCount} new
              </span>
            )}
          </div>
          <button
            onClick={checkNow}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition"
          >
            <RefreshCw className="w-4 h-4" />
            Check Now
          </button>
        </div>

        {/* Filters */}
        <div className="flex gap-2">
          {['all', 'unread', 'critical'].map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`px-4 py-2 rounded-lg capitalize transition ${
                filter === f
                  ? 'bg-primary-600 text-white'
                  : 'bg-white border hover:bg-gray-50'
              }`}
            >
              {f}
            </button>
          ))}
        </div>

        {/* Alerts List */}
        <div className="space-y-4">
          {filteredAlerts.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-xl border">
              <Bell className="w-12 h-12 text-gray-300 mx-auto mb-4" />
              <p className="text-gray-500">No alerts found</p>
            </div>
          ) : (
            filteredAlerts.map((alert) => (
              <AlertCard key={alert.id} alert={alert} onMarkRead={markAsRead} />
            ))
          )}
        </div>
      </div>
    </Layout>
  )
}

function AlertCard({ alert, onMarkRead }: { alert: Alert; onMarkRead: (id: string) => void }) {
  const SeverityIcon = {
    'Critical': AlertCircle,
    'Warning': AlertTriangle,
    'Info': Info,
  }[alert.severity] || Info

  const severityColor = {
    'Critical': 'bg-red-50 border-red-200',
    'Warning': 'bg-yellow-50 border-yellow-200',
    'Info': 'bg-blue-50 border-blue-200',
  }[alert.severity] || 'bg-gray-50 border-gray-200'

  const iconColor = {
    'Critical': 'text-red-500',
    'Warning': 'text-yellow-500',
    'Info': 'text-blue-500',
  }[alert.severity] || 'text-gray-500'

  return (
    <div className={`p-6 rounded-xl border ${severityColor} ${!alert.is_read ? 'ring-2 ring-primary-200' : ''}`}>
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-4">
          <div className={`p-2 rounded-lg bg-white ${iconColor}`}>
            <SeverityIcon className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-lg">{alert.title}</h3>
              {!alert.is_read && (
                <span className="bg-primary-500 text-white text-xs px-2 py-0.5 rounded-full">
                  New
                </span>
              )}
            </div>
            <p className="text-gray-600 mt-1">{alert.message}</p>
            
            {/* Metadata */}
            {alert.metadata && (
              <div className="mt-3 flex flex-wrap gap-2">
                {alert.metadata.current_stock !== undefined && (
                  <span className="text-sm bg-white px-3 py-1 rounded-full border">
                    Stock: {alert.metadata.current_stock}
                  </span>
                )}
                {alert.metadata.suggested_quantity !== undefined && (
                  <span className="text-sm bg-white px-3 py-1 rounded-full border">
                    Suggested: {alert.metadata.suggested_quantity}
                  </span>
                )}
                {alert.metadata.deviation_percent !== undefined && (
                  <span className="text-sm bg-white px-3 py-1 rounded-full border">
                    Deviation: {Number(alert.metadata.deviation_percent || 0).toFixed(1)}%
                  </span>
                )}
              </div>
            )}

            <p className="text-sm text-gray-400 mt-3">
              {new Date(alert.created_at).toLocaleString()}
            </p>
          </div>
        </div>

        {!alert.is_read && (
          <button
            onClick={() => onMarkRead(alert.id)}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 hover:bg-white rounded-lg transition"
          >
            <Check className="w-4 h-4" />
            Mark as read
          </button>
        )}
      </div>
    </div>
  )
}
