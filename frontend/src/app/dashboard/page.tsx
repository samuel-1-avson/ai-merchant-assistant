'use client'

import { useState, useEffect } from 'react'
import { VoiceRecorder } from '@/components/voice/VoiceRecorder'
import { StatsCards } from '@/components/dashboard/StatsCards'
import { RecentTransactions } from '@/components/dashboard/RecentTransactions'
import { Layout } from '@/components/layout/Layout'

export default function Dashboard() {
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Simulate loading data
    setTimeout(() => setIsLoading(false), 1000)
  }, [])

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
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-gray-500">{new Date().toLocaleDateString()}</p>
        </div>

        <StatsCards />

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white p-6 rounded-xl shadow-sm border">
            <h2 className="text-xl font-semibold mb-4">Record a Sale</h2>
            <VoiceRecorder />
          </div>

          <div className="bg-white p-6 rounded-xl shadow-sm border">
            <h2 className="text-xl font-semibold mb-4">Recent Transactions</h2>
            <RecentTransactions />
          </div>
        </div>
      </div>
    </Layout>
  )
}
